use std::collections::HashMap;
use std::sync::Mutex;
use std::time::Instant;

use super::config::ButtonConfig;

struct CachedArt {
    url: Option<String>,
    fetched_at: Instant,
}

struct SpotifyToken {
    access_token: String,
    expires_at: Instant,
}

static ALBUM_ART_CACHE: std::sync::LazyLock<Mutex<HashMap<String, CachedArt>>> =
    std::sync::LazyLock::new(|| Mutex::new(HashMap::new()));

static SPOTIFY_TOKEN: std::sync::LazyLock<Mutex<Option<SpotifyToken>>> =
    std::sync::LazyLock::new(|| Mutex::new(None));

const CACHE_TTL_SECS: u64 = 3600; // 1 hour
const NEGATIVE_CACHE_TTL_SECS: u64 = 300; // 5 minutes for "not found" (retry sooner)

const SPOTIFY_CLIENT_ID: &str = "78a2ba90e40f48ea8d22a2a9ec8e3802";
const SPOTIFY_CLIENT_SECRET: &str = "7fd823b5305e4f2aa0c299c3a16e3515";

/// Clean up a media title for better search results
fn clean_title(title: &str) -> String {
    let mut s = title.to_string();

    let noise_parens = [
        "official video", "official music video", "official audio",
        "official lyric video", "lyric video", "lyrics video", "lyrics",
        "music video", "audio", "visualizer", "video officiel",
        "clip officiel", "clip official", "hd", "hq", "4k",
        "live", "en vivo", "en direct",
    ];
    for pattern in &["(", "["] {
        let close = if *pattern == "(" { ")" } else { "]" };
        while let Some(start) = s.find(pattern) {
            if let Some(end) = s[start..].find(close) {
                let inner = s[start + 1..start + end].to_lowercase();
                if noise_parens.iter().any(|n| inner.contains(n)) {
                    s = format!("{}{}", &s[..start], &s[start + end + 1..]);
                    continue;
                }
            }
            break;
        }
    }

    let feat_markers = ["feat.", "ft.", "featuring", "feat ", "ft "];
    let lower = s.to_lowercase();
    for marker in &feat_markers {
        if let Some(pos) = lower.find(marker) {
            if pos > 0 {
                s = s[..pos].to_string();
            }
        }
    }

    if let Some(pos) = s.rfind(" - ") {
        let after = s[pos + 3..].to_lowercase();
        let noise_suffixes = ["topic", "official", "vevo", "records", "music"];
        if noise_suffixes.iter().any(|n| after.contains(n)) {
            s = s[..pos].to_string();
        }
    }

    s.trim().trim_matches(|c: char| c == '-' || c == '|' || c == '·').trim().to_string()
}

fn clean_artist(artist: &str) -> String {
    let s = artist.replace(" - Topic", "").replace(" - Sujet", "");
    s.trim().to_string()
}

/// Fetch album art URL — tries Spotify first (most precise), then Deezer, then iTunes
pub async fn fetch_album_art_url(artist: &str, title: &str) -> Option<String> {
    let clean_a = clean_artist(artist);
    let clean_t = clean_title(title);
    let cache_key = format!("{}|{}", clean_a.to_lowercase(), clean_t.to_lowercase());

    // Check cache
    {
        if let Ok(cache) = ALBUM_ART_CACHE.lock() {
            if let Some(entry) = cache.get(&cache_key) {
                let elapsed = entry.fetched_at.elapsed().as_secs();
                let ttl = if entry.url.is_some() { CACHE_TTL_SECS } else { NEGATIVE_CACHE_TTL_SECS };
                if elapsed < ttl {
                    return entry.url.clone();
                }
            }
        }
    }

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(8))
        .build()
        .ok();

    let client = match client {
        Some(c) => c,
        None => {
            cache_negative(&cache_key);
            return None;
        }
    };

    // === Strategy 1: Spotify (most precise) ===
    {
        let result = try_spotify_search(&client, &clean_a, &clean_t, SPOTIFY_CLIENT_ID, SPOTIFY_CLIENT_SECRET).await;
        if result.is_some() {
            cache_result(&cache_key, &result);
            return result;
        }
    }

    // === Strategy 2: Deezer (best coverage for French/international music) ===
    let result = try_deezer_search(&client, &clean_a, &clean_t).await;
    if result.is_some() {
        cache_result(&cache_key, &result);
        return result;
    }

    // === Strategy 3: iTunes (good for English/US music) ===
    let result = try_itunes_search(&client, &clean_a, &clean_t).await;
    if result.is_some() {
        cache_result(&cache_key, &result);
        return result;
    }

    // === Strategy 4: Simplified title on Deezer ===
    let simplified = simplify_title(&clean_t);
    if simplified != clean_t {
        let result = try_deezer_search(&client, &clean_a, &simplified).await;
        if result.is_some() {
            cache_result(&cache_key, &result);
            return result;
        }
    }

    // === Strategy 5: Title only (no artist) on Deezer ===
    if !clean_a.is_empty() {
        let result = try_deezer_search(&client, "", &clean_t).await;
        if result.is_some() {
            cache_result(&cache_key, &result);
            return result;
        }
    }

    // === Strategy 6: Artist only on Deezer (at least get an artist image) ===
    if !clean_a.is_empty() && clean_a.len() >= 2 {
        let result = try_deezer_artist_image(&client, &clean_a).await;
        if result.is_some() {
            cache_result(&cache_key, &result);
            return result;
        }
    }

    cache_negative(&cache_key);
    None
}

fn simplify_title(title: &str) -> String {
    let separators = [" - ", " | ", " · ", " // ", " — "];
    let mut result = title.to_string();
    for sep in &separators {
        if let Some(pos) = result.find(sep) {
            if pos > 3 {
                result = result[..pos].to_string();
            }
        }
    }
    result.trim().to_string()
}

// ══════════════════════════ SPOTIFY API ══════════════════════════

/// Get or refresh Spotify access token using Client Credentials flow
async fn get_spotify_token(client: &reqwest::Client, client_id: &str, client_secret: &str) -> Option<String> {
    // Check cached token
    {
        if let Ok(guard) = SPOTIFY_TOKEN.lock() {
            if let Some(ref token) = *guard {
                if token.expires_at > Instant::now() {
                    return Some(token.access_token.clone());
                }
            }
        }
    }

    // Request new token
    use base64::Engine;
    let credentials = format!("{}:{}", client_id, client_secret);
    let b64 = base64::engine::general_purpose::STANDARD.encode(credentials.as_bytes());

    let resp = client
        .post("https://accounts.spotify.com/api/token")
        .header("Authorization", format!("Basic {}", b64))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body("grant_type=client_credentials")
        .send()
        .await
        .ok()?;

    let json: serde_json::Value = resp.json().await.ok()?;
    let access_token = json.get("access_token")?.as_str()?.to_string();
    let expires_in = json.get("expires_in").and_then(|v| v.as_u64()).unwrap_or(3600);

    // Cache token (expire 60s early to be safe)
    if let Ok(mut guard) = SPOTIFY_TOKEN.lock() {
        *guard = Some(SpotifyToken {
            access_token: access_token.clone(),
            expires_at: Instant::now() + std::time::Duration::from_secs(expires_in.saturating_sub(60)),
        });
    }

    Some(access_token)
}

async fn try_spotify_search(client: &reqwest::Client, artist: &str, title: &str, client_id: &str, client_secret: &str) -> Option<String> {
    let token = get_spotify_token(client, client_id, client_secret).await?;

    let query = if artist.is_empty() {
        format!("track:{}", title)
    } else if title.is_empty() {
        format!("artist:{}", artist)
    } else {
        format!("artist:{} track:{}", artist, title)
    };

    let url = format!(
        "https://api.spotify.com/v1/search?q={}&type=track&limit=5",
        urlencoded(&query)
    );

    let resp = client
        .get(&url)
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .ok()?;

    let json: serde_json::Value = resp.json().await.ok()?;
    let items = json.get("tracks")?.get("items")?.as_array()?;

    if items.is_empty() {
        return None;
    }

    let artist_lower = artist.to_lowercase();
    let title_lower = title.to_lowercase();

    let mut best: Option<&serde_json::Value> = None;
    let mut best_score = 0i32;

    for item in items {
        let mut score = 0i32;

        // Check artists
        if let Some(artists) = item.get("artists").and_then(|v| v.as_array()) {
            for a in artists {
                if let Some(name) = a.get("name").and_then(|v| v.as_str()) {
                    let a_lower = name.to_lowercase();
                    if !artist_lower.is_empty() {
                        if a_lower == artist_lower {
                            score += 5;
                        } else if a_lower.contains(&artist_lower) || artist_lower.contains(&a_lower) {
                            score += 3;
                        }
                    }
                }
            }
        }

        // Check track name
        if let Some(name) = item.get("name").and_then(|v| v.as_str()) {
            let t_lower = name.to_lowercase();
            if !title_lower.is_empty() {
                if t_lower == title_lower {
                    score += 5;
                } else if t_lower.contains(&title_lower) || title_lower.contains(&t_lower) {
                    score += 3;
                }
            }
        }

        if score > best_score {
            best_score = score;
            best = Some(item);
        }
    }

    let chosen = if best_score >= 2 { best } else { items.first() };
    let chosen = chosen?;

    // Get album art — largest image first
    let images = chosen.get("album")?.get("images")?.as_array()?;
    if images.is_empty() {
        return None;
    }

    // Spotify returns images sorted by size desc (640, 300, 64)
    images.first()?.get("url").and_then(|v| v.as_str()).map(|s| s.to_string())
}

/// Fetch album tracks via Deezer API (free, no auth required)
pub async fn fetch_album_tracks(artist: &str, title: &str) -> Option<AlbumInfo> {
    let clean_a = clean_artist(artist);
    let clean_t = clean_title(title);

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(8))
        .build()
        .ok()?;

    // Search for the track on Deezer
    let queries = vec![
        format!("artist:\"{}\" track:\"{}\"", clean_a, clean_t),
        format!("{} {}", clean_a, clean_t),
        clean_t.clone(),
    ];

    let mut album_id: Option<u64> = None;

    for query in &queries {
        let url = format!("https://api.deezer.com/search?q={}&limit=5", urlencoded(query));
        if let Ok(resp) = client.get(&url).send().await {
            if let Ok(json) = resp.json::<serde_json::Value>().await {
                if let Some(data) = json.get("data").and_then(|d| d.as_array()) {
                    // Find best match
                    let artist_lower = clean_a.to_lowercase();
                    let title_lower = clean_t.to_lowercase();

                    for item in data {
                        let t_name = item.get("title").and_then(|v| v.as_str()).unwrap_or("");
                        let a_name = item.get("artist").and_then(|a| a.get("name")).and_then(|v| v.as_str()).unwrap_or("");

                        let t_match = t_name.to_lowercase().contains(&title_lower) || title_lower.contains(&t_name.to_lowercase().as_str());
                        let a_match = a_name.to_lowercase().contains(&artist_lower) || artist_lower.contains(&a_name.to_lowercase().as_str());

                        if t_match || a_match {
                            if let Some(id) = item.get("album").and_then(|a| a.get("id")).and_then(|v| v.as_u64()) {
                                album_id = Some(id);
                                break;
                            }
                        }
                    }
                    // Fallback: take first result
                    if album_id.is_none() && !data.is_empty() {
                        album_id = data[0].get("album").and_then(|a| a.get("id")).and_then(|v| v.as_u64());
                    }
                    if album_id.is_some() { break; }
                }
            }
        }
    }

    let album_id = album_id?;

    // Fetch album info
    let album_url = format!("https://api.deezer.com/album/{}", album_id);
    let album_resp = client.get(&album_url).send().await.ok()?;
    let album_json: serde_json::Value = album_resp.json().await.ok()?;

    let album_name = album_json.get("title").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let artist_name = album_json.get("artist")
        .and_then(|a| a.get("name"))
        .and_then(|v| v.as_str())
        .unwrap_or(artist)
        .to_string();
    let album_art = album_json.get("cover_xl")
        .or_else(|| album_json.get("cover_big"))
        .or_else(|| album_json.get("cover_medium"))
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let release_date = album_json.get("release_date").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let year = if release_date.len() >= 4 { release_date[..4].to_string() } else { release_date.clone() };

    // Get tracks from album
    let track_items = album_json.get("tracks")
        .and_then(|t| t.get("data"))
        .and_then(|d| d.as_array())?;

    let tracks: Vec<TrackInfo> = track_items.iter().enumerate().filter_map(|(i, t)| {
        let name = t.get("title").and_then(|v| v.as_str())?.to_string();
        let duration = t.get("duration").and_then(|v| v.as_u64()).unwrap_or(0);
        let is_explicit = t.get("explicit_lyrics").and_then(|v| v.as_bool()).unwrap_or(false);
        Some(TrackInfo {
            name,
            duration_secs: duration as f64,
            track_number: (i + 1) as u32,
            is_explicit,
        })
    }).collect();

    let total_duration: f64 = tracks.iter().map(|t| t.duration_secs).sum();

    let deezer_link = album_json.get("link")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    Some(AlbumInfo {
        album_name,
        artist_name,
        album_art_url: album_art,
        year,
        total_tracks: tracks.len() as u32,
        total_duration_secs: total_duration,
        tracks,
        deezer_url: deezer_link,
    })
}

#[derive(serde::Serialize, Clone)]
pub struct TrackInfo {
    pub name: String,
    pub duration_secs: f64,
    pub track_number: u32,
    pub is_explicit: bool,
}

#[derive(serde::Serialize, Clone)]
pub struct AlbumInfo {
    pub album_name: String,
    pub artist_name: String,
    pub album_art_url: String,
    pub year: String,
    pub total_tracks: u32,
    pub total_duration_secs: f64,
    pub tracks: Vec<TrackInfo>,
    pub deezer_url: String,
}

/// Validate Spotify credentials by attempting to get a token
pub async fn validate_spotify_credentials(client_id: &str, client_secret: &str) -> bool {
    let client = match reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .ok()
    {
        Some(c) => c,
        None => return false,
    };
    get_spotify_token(&client, client_id, client_secret).await.is_some()
}

// ══════════════════════════ DEEZER API ══════════════════════════

async fn try_deezer_search(client: &reqwest::Client, artist: &str, title: &str) -> Option<String> {
    let query = if artist.is_empty() {
        format!("track:\"{}\"", title)
    } else if title.is_empty() {
        format!("artist:\"{}\"", artist)
    } else {
        format!("artist:\"{}\" track:\"{}\"", artist, title)
    };

    if query.trim().is_empty() || query.trim().len() < 3 {
        return None;
    }

    let url = format!(
        "https://api.deezer.com/search?q={}&limit=5",
        urlencoded(&query)
    );

    let resp = client.get(&url).send().await.ok()?;
    let json: serde_json::Value = resp.json().await.ok()?;
    let results = json.get("data")?.as_array()?;

    if results.is_empty() {
        // Deezer strict search failed, try loose search
        let loose_query = if artist.is_empty() {
            title.to_string()
        } else if title.is_empty() {
            artist.to_string()
        } else {
            format!("{} {}", artist, title)
        };

        let loose_url = format!(
            "https://api.deezer.com/search?q={}&limit=5",
            urlencoded(&loose_query)
        );

        let resp2 = client.get(&loose_url).send().await.ok()?;
        let json2: serde_json::Value = resp2.json().await.ok()?;
        let results2 = json2.get("data")?.as_array()?;

        if results2.is_empty() {
            return None;
        }

        return pick_best_deezer(results2, artist, title);
    }

    pick_best_deezer(results, artist, title)
}

fn pick_best_deezer(results: &[serde_json::Value], artist: &str, title: &str) -> Option<String> {
    let artist_lower = artist.to_lowercase();
    let title_lower = title.to_lowercase();

    let mut best: Option<&serde_json::Value> = None;
    let mut best_score = 0i32;

    for result in results {
        let mut score = 0i32;

        // Check artist
        if let Some(a) = result.get("artist").and_then(|v| v.get("name")).and_then(|v| v.as_str()) {
            let a_lower = a.to_lowercase();
            if !artist_lower.is_empty() {
                if a_lower == artist_lower {
                    score += 5;
                } else if a_lower.contains(&artist_lower) || artist_lower.contains(&a_lower) {
                    score += 3;
                }
            }
        }

        // Check title
        if let Some(t) = result.get("title").and_then(|v| v.as_str()) {
            let t_lower = t.to_lowercase();
            if !title_lower.is_empty() {
                if t_lower == title_lower {
                    score += 5;
                } else if t_lower.contains(&title_lower) || title_lower.contains(&t_lower) {
                    score += 3;
                }
                // Word-level matching
                let title_words: Vec<&str> = title_lower.split_whitespace()
                    .filter(|w| w.len() > 2)
                    .collect();
                let match_count = title_words.iter().filter(|w| t_lower.contains(*w)).count();
                score += match_count as i32;
            }
        }

        // Prefer results with album art
        if result.get("album").and_then(|v| v.get("cover_big")).and_then(|v| v.as_str()).is_some() {
            score += 1;
        }

        if score > best_score {
            best_score = score;
            best = Some(result);
        }
    }

    let chosen = if best_score >= 2 { best } else { results.first() };
    let chosen = chosen?;

    // Get album cover — prefer cover_xl (1000x1000) > cover_big (500x500)
    let album = chosen.get("album")?;
    if let Some(url) = album.get("cover_xl").and_then(|v| v.as_str()) {
        if !url.is_empty() {
            return Some(url.to_string());
        }
    }
    if let Some(url) = album.get("cover_big").and_then(|v| v.as_str()) {
        if !url.is_empty() {
            return Some(url.to_string());
        }
    }
    album.get("cover_medium").and_then(|v| v.as_str()).map(|s| s.to_string())
}

/// Get artist image from Deezer as last resort
async fn try_deezer_artist_image(client: &reqwest::Client, artist: &str) -> Option<String> {
    let url = format!(
        "https://api.deezer.com/search/artist?q={}&limit=1",
        urlencoded(artist)
    );

    let resp = client.get(&url).send().await.ok()?;
    let json: serde_json::Value = resp.json().await.ok()?;
    let results = json.get("data")?.as_array()?;
    let first = results.first()?;

    if let Some(img) = first.get("picture_xl").and_then(|v| v.as_str()) {
        if !img.is_empty() {
            return Some(img.to_string());
        }
    }
    if let Some(img) = first.get("picture_big").and_then(|v| v.as_str()) {
        if !img.is_empty() {
            return Some(img.to_string());
        }
    }
    first.get("picture_medium").and_then(|v| v.as_str()).map(|s| s.to_string())
}

// ══════════════════════════ ITUNES API ══════════════════════════

async fn try_itunes_search(client: &reqwest::Client, artist: &str, title: &str) -> Option<String> {
    let query = if artist.is_empty() {
        title.to_string()
    } else if title.is_empty() {
        artist.to_string()
    } else {
        format!("{} {}", artist, title)
    };

    if query.trim().is_empty() {
        return None;
    }

    let url = format!(
        "https://itunes.apple.com/search?term={}&media=music&limit=10&entity=song",
        urlencoded(&query)
    );

    let resp = client.get(&url).send().await.ok()?;
    let json: serde_json::Value = resp.json().await.ok()?;
    let results = json.get("results")?.as_array()?;

    if results.is_empty() {
        return None;
    }

    let artist_lower = artist.to_lowercase();
    let title_lower = title.to_lowercase();

    let mut best: Option<&serde_json::Value> = None;
    let mut best_score = 0i32;

    for result in results {
        let mut score = 0i32;

        if let Some(a) = result.get("artistName").and_then(|v| v.as_str()) {
            let a_lower = a.to_lowercase();
            if !artist_lower.is_empty() {
                if a_lower == artist_lower {
                    score += 5;
                } else if a_lower.contains(&artist_lower) || artist_lower.contains(&a_lower) {
                    score += 3;
                }
                let artist_words: Vec<&str> = artist_lower.split_whitespace().collect();
                let match_count = artist_words.iter().filter(|w| a_lower.contains(*w)).count();
                score += match_count as i32;
            }
        }

        if let Some(t) = result.get("trackName").and_then(|v| v.as_str()) {
            let t_lower = t.to_lowercase();
            if !title_lower.is_empty() {
                if t_lower == title_lower {
                    score += 5;
                } else if t_lower.contains(&title_lower) || title_lower.contains(&t_lower) {
                    score += 3;
                }
                let title_words: Vec<&str> = title_lower.split_whitespace()
                    .filter(|w| w.len() > 2)
                    .collect();
                let match_count = title_words.iter().filter(|w| t_lower.contains(*w)).count();
                score += match_count as i32;
            }
        }

        if result.get("artworkUrl100").and_then(|v| v.as_str()).is_some() {
            score += 1;
        }

        if score > best_score {
            best_score = score;
            best = Some(result);
        }
    }

    let chosen = if best_score >= 2 { best } else { results.first() };
    let chosen = chosen?;

    let artwork_url = chosen.get("artworkUrl100")?.as_str()?;
    let hd_url = artwork_url.replace("100x100bb", "600x600bb");

    Some(hd_url)
}

// ══════════════════════════ UTILS ══════════════════════════

fn cache_result(key: &str, result: &Option<String>) {
    if let Ok(mut cache) = ALBUM_ART_CACHE.lock() {
        cache.insert(key.to_string(), CachedArt {
            url: result.clone(),
            fetched_at: Instant::now(),
        });
    }
}

fn cache_negative(key: &str) {
    if let Ok(mut cache) = ALBUM_ART_CACHE.lock() {
        cache.insert(key.to_string(), CachedArt {
            url: None,
            fetched_at: Instant::now(),
        });
    }
}

/// Verify if artist+title correspond to a REAL known song on Deezer.
/// Uses strict matching: artist name AND track title must closely match.
/// Returns true only for actual music, not random YouTube videos.
pub async fn is_known_music(artist: &str, title: &str) -> bool {
    let clean_a = clean_artist(artist);
    let clean_t = clean_title(title);

    // Need both artist and title for strict verification
    if clean_a.len() < 2 || clean_t.len() < 2 {
        return false;
    }

    let client = match reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .ok()
    {
        Some(c) => c,
        None => return false,
    };

    // Strict Deezer search: artist + track
    let query = format!("artist:\"{}\" track:\"{}\"", clean_a, clean_t);
    let url = format!(
        "https://api.deezer.com/search?q={}&limit=3&strict=on",
        urlencoded(&query)
    );

    let resp = match client.get(&url).send().await {
        Ok(r) => r,
        Err(_) => return false,
    };
    let json: serde_json::Value = match resp.json().await {
        Ok(j) => j,
        Err(_) => return false,
    };

    let results = match json.get("data").and_then(|v| v.as_array()) {
        Some(r) => r,
        None => return false,
    };

    if results.is_empty() {
        return false;
    }

    let artist_lower = clean_a.to_lowercase();
    let title_lower = clean_t.to_lowercase();

    // Check if ANY result has a close artist match
    for result in results {
        let mut artist_ok = false;
        let mut title_ok = false;

        if let Some(a) = result.get("artist").and_then(|v| v.get("name")).and_then(|v| v.as_str()) {
            let a_lower = a.to_lowercase();
            // Artist must be a substring match (either direction)
            if a_lower.contains(&artist_lower) || artist_lower.contains(&a_lower) {
                artist_ok = true;
            }
            // Also check word-level: if most artist words match
            let artist_words: Vec<&str> = artist_lower.split_whitespace().filter(|w| w.len() > 1).collect();
            if !artist_words.is_empty() {
                let match_count = artist_words.iter().filter(|w| a_lower.contains(*w)).count();
                if match_count * 2 >= artist_words.len() {
                    artist_ok = true;
                }
            }
        }

        if let Some(t) = result.get("title").and_then(|v| v.as_str()) {
            let t_lower = t.to_lowercase();
            if t_lower.contains(&title_lower) || title_lower.contains(&t_lower) {
                title_ok = true;
            }
            // Word-level check for title
            let title_words: Vec<&str> = title_lower.split_whitespace().filter(|w| w.len() > 2).collect();
            if !title_words.is_empty() {
                let match_count = title_words.iter().filter(|w| t_lower.contains(*w)).count();
                if match_count * 2 >= title_words.len() {
                    title_ok = true;
                }
            }
        }

        if artist_ok && title_ok {
            return true;
        }
    }

    false
}

/// Generate a "Listen/Watch on X" button based on the player/service
pub fn get_listen_url(player_name: &str, artist: &str, title: &str) -> Option<ButtonConfig> {
    let clean_a = clean_artist(artist);
    let clean_t = clean_title(title);
    let query = urlencoded(&format!("{} {}", clean_a, clean_t));
    let title_query = urlencoded(&clean_t);
    let player = player_name.to_lowercase();

    // ── Music services ──
    if player.contains("spotify") {
        return Some(ButtonConfig {
            label: "Listen on Spotify".to_string(),
            url: format!("https://open.spotify.com/search/{}", query),
        });
    }
    if player.contains("apple music") || player.contains("itunes") {
        return Some(ButtonConfig {
            label: "Listen on Apple Music".to_string(),
            url: format!("https://music.apple.com/search?term={}", query),
        });
    }
    if player == "youtube music" {
        return Some(ButtonConfig {
            label: "Listen on YouTube Music".to_string(),
            url: format!("https://music.youtube.com/search?q={}", query),
        });
    }
    if player.contains("tidal") {
        return Some(ButtonConfig {
            label: "Listen on TIDAL".to_string(),
            url: format!("https://listen.tidal.com/search?q={}", query),
        });
    }
    if player.contains("deezer") {
        return Some(ButtonConfig {
            label: "Listen on Deezer".to_string(),
            url: format!("https://www.deezer.com/search/{}", query),
        });
    }
    if player.contains("soundcloud") {
        return Some(ButtonConfig {
            label: "Listen on SoundCloud".to_string(),
            url: format!("https://soundcloud.com/search?q={}", query),
        });
    }
    if player.contains("amazon music") {
        return Some(ButtonConfig {
            label: "Listen on Amazon Music".to_string(),
            url: format!("https://music.amazon.com/search/{}", query),
        });
    }

    // ── Video/streaming services ──
    if player == "youtube" {
        return Some(ButtonConfig {
            label: "Watch on YouTube".to_string(),
            url: format!("https://www.youtube.com/results?search_query={}", title_query),
        });
    }
    if player.contains("twitch") {
        // For Twitch, artist = streamer name
        let streamer = urlencoded(&clean_a);
        return Some(ButtonConfig {
            label: "Watch on Twitch".to_string(),
            url: if !clean_a.is_empty() {
                format!("https://www.twitch.tv/{}", streamer)
            } else {
                "https://www.twitch.tv".to_string()
            },
        });
    }
    if player.contains("netflix") {
        return Some(ButtonConfig {
            label: "Watch on Netflix".to_string(),
            url: "https://www.netflix.com".to_string(),
        });
    }
    if player.contains("disney") {
        return Some(ButtonConfig {
            label: "Watch on Disney+".to_string(),
            url: "https://www.disneyplus.com".to_string(),
        });
    }
    if player.contains("prime video") {
        return Some(ButtonConfig {
            label: "Watch on Prime Video".to_string(),
            url: "https://www.primevideo.com".to_string(),
        });
    }
    if player.contains("crunchyroll") {
        return Some(ButtonConfig {
            label: "Watch on Crunchyroll".to_string(),
            url: format!("https://www.crunchyroll.com/search?q={}", title_query),
        });
    }
    if player.contains("kick") {
        return Some(ButtonConfig {
            label: "Watch on Kick".to_string(),
            url: if !clean_a.is_empty() {
                format!("https://kick.com/{}", urlencoded(&clean_a))
            } else {
                "https://kick.com".to_string()
            },
        });
    }
    if player.contains("dailymotion") {
        return Some(ButtonConfig {
            label: "Watch on Dailymotion".to_string(),
            url: format!("https://www.dailymotion.com/search/{}", title_query),
        });
    }

    // ── Fallback for browsers ──
    if player.contains("chrome") || player.contains("firefox") || player.contains("edge")
        || player.contains("brave") || player.contains("browser") || player.contains("opera") {
        return Some(ButtonConfig {
            label: "Watch on YouTube".to_string(),
            url: format!("https://www.youtube.com/results?search_query={}", title_query),
        });
    }

    // ── Default: search on Apple Music ──
    Some(ButtonConfig {
        label: "Search Song".to_string(),
        url: format!("https://music.apple.com/search?term={}", query),
    })
}

/// Tauri command to validate Spotify credentials from the frontend
#[tauri::command]
pub async fn validate_spotify(client_id: String, client_secret: String) -> Result<bool, String> {
    Ok(validate_spotify_credentials(&client_id, &client_secret).await)
}

#[tauri::command]
pub async fn get_album_tracks_cmd(artist: String, title: String) -> Result<Option<AlbumInfo>, String> {
    Ok(fetch_album_tracks(&artist, &title).await)
}

/// URL encoding that properly handles Unicode (accents, CJK, etc.)
fn urlencoded(s: &str) -> String {
    let mut result = String::new();
    for byte in s.as_bytes() {
        match *byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                result.push(*byte as char);
            }
            b' ' => result.push('+'),
            _ => {
                result.push_str(&format!("%{:02X}", byte));
            }
        }
    }
    result
}
