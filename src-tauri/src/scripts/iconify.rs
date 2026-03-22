use serde::{Deserialize, Serialize};
use tauri::command;

const API_URL: &str = "https://api.iconify.design";

/// Colorful icon sets (priority order)
const COLORFUL_SETS: &str = "logos,devicon,vscode-icons,fluent-emoji,fluent-emoji-flat,noto";
const COLORFUL_SETS_2: &str = "flat-color-icons,mdi,twemoji,openmoji";

/// Monochrome sets to avoid
const MONOCHROME_SETS: &[&str] = &[
    "simple-icons", "carbon", "tabler", "lucide", "heroicons", "feather", "bytesize",
    "octicon", "codicon", "radix-icons", "ph", "ri", "uil",
];

/// Sets where the base name usually contains text/wordmark
/// e.g. logos:discord = text+logo, logos:discord-icon = icon only
const ICON_SUFFIX_SETS: &[&str] = &["logos"];

/// Name parts that indicate text-heavy variants
const TEXT_INDICATORS: &[&str] = &[
    "wordmark", "-word", "logotype", "-text", "-horizontal", "-vertical",
];

#[derive(Deserialize, Debug)]
struct SearchResponse {
    icons: Vec<String>,
}

/// Search Iconify for an icon matching the app name, return a proxied PNG URL.
/// Uses smart search: tries full name, then individual keywords, then simplified name.
/// Prioritizes COLORFUL icon sets, ICON-ONLY variants (no text), avoids monochrome.
/// Known music/media player icons (hardcoded PNG URLs for accuracy)
fn known_player_icon(name: &str) -> Option<String> {
    let lower = name.to_lowercase();
    // Use weserv proxy to convert SVGs to PNGs that Discord can display
    let url = match lower.as_str() {
        "apple music" | "itunes" | "music" =>
            "https://images.weserv.nl/?url=https%3A%2F%2Fapi.iconify.design%2Fsimple-icons%2Fapplemusic.svg%3Fcolor%3D%2523fa243c%26width%3D512&w=512&h=512&output=png",
        "spotify" | "spotify music" =>
            "https://images.weserv.nl/?url=https%3A%2F%2Fapi.iconify.design%2Fsimple-icons%2Fspotify.svg%3Fcolor%3D%25231db954%26width%3D512&w=512&h=512&output=png",
        "youtube" =>
            "https://images.weserv.nl/?url=https%3A%2F%2Fapi.iconify.design%2Flogos%2Fyoutube-icon.svg%3Fwidth%3D512%26height%3D512&w=512&h=512&output=png",
        "youtube music" =>
            "https://images.weserv.nl/?url=https%3A%2F%2Fapi.iconify.design%2Fsimple-icons%2Fyoutubemusic.svg%3Fcolor%3D%2523ff0000%26width%3D512&w=512&h=512&output=png",
        "twitch" =>
            "https://images.weserv.nl/?url=https%3A%2F%2Fapi.iconify.design%2Flogos%2Ftwitch.svg%3Fwidth%3D512%26height%3D512&w=512&h=512&output=png",
        "soundcloud" =>
            "https://images.weserv.nl/?url=https%3A%2F%2Fapi.iconify.design%2Fsimple-icons%2Fsoundcloud.svg%3Fcolor%3D%2523ff5500%26width%3D512&w=512&h=512&output=png",
        "deezer" =>
            "https://images.weserv.nl/?url=https%3A%2F%2Fapi.iconify.design%2Fsimple-icons%2Fdeezer.svg%3Fcolor%3D%2523feaa2d%26width%3D512&w=512&h=512&output=png",
        "tidal" =>
            "https://images.weserv.nl/?url=https%3A%2F%2Fapi.iconify.design%2Fsimple-icons%2Ftidal.svg%3Fcolor%3D%252300ffff%26width%3D512&w=512&h=512&output=png",
        "vlc" | "vlc media player" =>
            "https://images.weserv.nl/?url=https%3A%2F%2Fapi.iconify.design%2Fsimple-icons%2Fvlcmediaplayer.svg%3Fcolor%3D%2523ff8800%26width%3D512&w=512&h=512&output=png",
        "firefox" | "mozilla firefox" =>
            "https://images.weserv.nl/?url=https%3A%2F%2Fapi.iconify.design%2Flogos%2Ffirefox.svg%3Fwidth%3D512%26height%3D512&w=512&h=512&output=png",
        "chrome" | "google chrome" =>
            "https://images.weserv.nl/?url=https%3A%2F%2Fapi.iconify.design%2Flogos%2Fchrome.svg%3Fwidth%3D512%26height%3D512&w=512&h=512&output=png",
        "edge" | "microsoft edge" =>
            "https://images.weserv.nl/?url=https%3A%2F%2Fapi.iconify.design%2Flogos%2Fmicrosoft-edge.svg%3Fwidth%3D512%26height%3D512&w=512&h=512&output=png",
        "brave" | "brave browser" =>
            "https://images.weserv.nl/?url=https%3A%2F%2Fapi.iconify.design%2Flogos%2Fbrave.svg%3Fwidth%3D512%26height%3D512&w=512&h=512&output=png",
        "opera" | "opera gx" =>
            "https://images.weserv.nl/?url=https%3A%2F%2Fapi.iconify.design%2Flogos%2Fopera.svg%3Fwidth%3D512%26height%3D512&w=512&h=512&output=png",
        _ => return None,
    };
    Some(url.to_string())
}

pub async fn fetch_icon(app_name: &str) -> Option<String> {
    if app_name.is_empty() {
        return None;
    }

    // Check known players first
    if let Some(url) = known_player_icon(app_name) {
        return Some(url);
    }

    let client = reqwest::Client::new();

    // Build a list of search queries to try (most specific → least specific)
    let mut queries: Vec<String> = Vec::new();

    // 1. Full name as-is
    queries.push(app_name.to_string());

    // 2. Remove common prefixes that pollute search
    let cleaned = app_name
        .replace("Microsoft ", "")
        .replace("Adobe ", "")
        .replace("JetBrains ", "")
        .replace("Google ", "")
        .trim()
        .to_string();
    if cleaned != app_name && !cleaned.is_empty() {
        queries.push(cleaned);
    }

    // 3. Individual significant words (skip short/common ones)
    let skip_words = [
        "the", "and", "for", "pro", "app", "desktop", "client",
        "launcher", "using", "on", "studio", "code",
    ];
    let words: Vec<&str> = app_name
        .split_whitespace()
        .filter(|w| w.len() > 2 && !skip_words.contains(&w.to_lowercase().as_str()))
        .collect();
    for word in &words {
        let w = word.to_string();
        if !queries.contains(&w) {
            queries.push(w);
        }
    }

    // Try each query with our priority search strategy
    for query in &queries {
        if let Some(url) = search_with_priorities(&client, query).await {
            return Some(url);
        }
    }

    None
}

/// Try searching Iconify with multiple icon set priorities for a given query
async fn search_with_priorities(client: &reqwest::Client, query: &str) -> Option<String> {
    // Priority 1: Colorful brand icon sets
    if let Some(url) = search_iconify(client, query, COLORFUL_SETS).await {
        return Some(url);
    }

    // Priority 2: More colorful sets (mdi, flat-color-icons, etc.)
    if let Some(url) = search_iconify(client, query, COLORFUL_SETS_2).await {
        return Some(url);
    }

    // Priority 3: Any set, but filter out monochrome
    let fallback_url = format!(
        "{}/search?query={}&limit=10",
        API_URL,
        urlencoding(query)
    );
    let resp = match client.get(&fallback_url).send().await {
        Ok(r) if r.status().is_success() => r,
        _ => return None,
    };
    let data: SearchResponse = resp.json().await.ok()?;

    // Prefer non-monochrome, icon-only
    let best = pick_best_icon(&data.icons);
    if let Some(icon_id) = best {
        return icon_to_png_url(icon_id);
    }

    // Last resort: even monochrome is better than nothing
    data.icons.first().and_then(|id| icon_to_png_url(id))
}

/// Search a specific set of Iconify prefixes for a query
async fn search_iconify(client: &reqwest::Client, query: &str, prefixes: &str) -> Option<String> {
    let url = format!(
        "{}/search?query={}&limit=15&prefixes={}",
        API_URL,
        urlencoding(query),
        prefixes
    );

    let resp = client.get(&url).send().await.ok()?;
    if !resp.status().is_success() {
        return None;
    }

    let data: SearchResponse = resp.json().await.ok()?;
    if data.icons.is_empty() {
        return None;
    }

    let best = pick_best_icon(&data.icons);
    best.and_then(|id| icon_to_png_url(id))
}

/// Pick the best icon from a list, preferring icon-only (no text) variants.
///
/// Priority order:
/// 1. Entries ending with "-icon" (pure symbol, guaranteed no text)
/// 2. Entries from emoji sets (fluent-emoji, noto, twemoji — always image-only)
/// 3. Entries from devicon/vscode-icons without wordmark (usually icon-only)
/// 4. Entries from "logos" set ONLY if name ends with "-icon" (otherwise has text)
/// 5. Any non-monochrome, non-text entry
fn pick_best_icon(icons: &[String]) -> Option<&String> {
    // Pass 1: "-icon" suffix = guaranteed icon-only (works for logos, devicon, etc.)
    for icon_id in icons {
        let name_part = icon_id.split(':').nth(1).unwrap_or("");
        if name_part.ends_with("-icon") {
            return Some(icon_id);
        }
    }

    // Pass 2: emoji sets are always pure images (no text ever)
    let emoji_sets = ["fluent-emoji", "fluent-emoji-flat", "noto", "twemoji", "openmoji", "emojione"];
    for icon_id in icons {
        let prefix = icon_id.split(':').next().unwrap_or("");
        if emoji_sets.contains(&prefix) {
            return Some(icon_id);
        }
    }

    // Pass 3: devicon, vscode-icons, mdi, flat-color-icons — usually icon-only
    // But skip wordmark variants
    let safe_icon_sets = ["devicon", "vscode-icons", "mdi", "flat-color-icons"];
    for icon_id in icons {
        let prefix = icon_id.split(':').next().unwrap_or("");
        let name_part = icon_id.split(':').nth(1).unwrap_or("");
        let is_safe_set = safe_icon_sets.contains(&prefix);
        let has_text = TEXT_INDICATORS.iter().any(|s| name_part.contains(s));
        if is_safe_set && !has_text {
            return Some(icon_id);
        }
    }

    // Pass 4: logos set — ONLY accept "-icon" variants (already handled in pass 1)
    // Skip "logos:" entries without "-icon" as they typically contain brand text

    // Pass 5: any non-monochrome entry that doesn't have text indicators
    for icon_id in icons {
        let prefix = icon_id.split(':').next().unwrap_or("");
        let name_part = icon_id.split(':').nth(1).unwrap_or("");
        let is_monochrome = MONOCHROME_SETS.contains(&prefix);
        let has_text = TEXT_INDICATORS.iter().any(|s| name_part.contains(s));
        let is_logos_without_icon = ICON_SUFFIX_SETS.contains(&prefix) && !name_part.ends_with("-icon");
        if !is_monochrome && !has_text && !is_logos_without_icon {
            return Some(icon_id);
        }
    }

    // Pass 6: accept logos without -icon (has text, but better than nothing)
    for icon_id in icons {
        let prefix = icon_id.split(':').next().unwrap_or("");
        let name_part = icon_id.split(':').nth(1).unwrap_or("");
        let is_monochrome = MONOCHROME_SETS.contains(&prefix);
        let has_text = TEXT_INDICATORS.iter().any(|s| name_part.contains(s));
        if !is_monochrome && !has_text {
            return Some(icon_id);
        }
    }

    None
}

/// Convert an Iconify icon ID (e.g. "logos:firefox") to a weserv-proxied PNG URL
fn icon_to_png_url(icon_id: &str) -> Option<String> {
    let parts: Vec<&str> = icon_id.splitn(2, ':').collect();
    if parts.len() != 2 {
        return None;
    }
    let prefix = parts[0];
    let name = parts[1];

    let svg_url = format!(
        "{}/{}/{}.svg?width=512&height=512",
        API_URL, prefix, name
    );
    let png_url = format!(
        "https://images.weserv.nl/?url={}&w=512&h=512&output=png",
        urlencoding_full(&svg_url)
    );

    Some(png_url)
}

/// Fetch multiple icon candidates for the user to choose from.
/// Returns a list of (icon_id, png_url) pairs — up to `limit` results.
/// Only returns icon-only variants (no text/wordmarks).
pub async fn fetch_icon_candidates(app_name: &str, limit: usize) -> Vec<IconCandidate> {
    if app_name.is_empty() {
        return Vec::new();
    }

    let client = reqwest::Client::new();
    let mut candidates: Vec<IconCandidate> = Vec::new();
    let mut seen_urls: std::collections::HashSet<String> = std::collections::HashSet::new();

    // Build search queries (same logic as fetch_icon)
    let mut queries: Vec<String> = Vec::new();
    queries.push(app_name.to_string());

    let cleaned = app_name
        .replace("Microsoft ", "")
        .replace("Adobe ", "")
        .replace("JetBrains ", "")
        .replace("Google ", "")
        .trim()
        .to_string();
    if cleaned != app_name && !cleaned.is_empty() {
        queries.push(cleaned);
    }

    let skip_words = [
        "the", "and", "for", "pro", "app", "desktop", "client",
        "launcher", "using", "on", "studio", "code",
    ];
    for word in app_name.split_whitespace() {
        let w = word.to_string();
        if w.len() > 2 && !skip_words.contains(&w.to_lowercase().as_str()) && !queries.contains(&w) {
            queries.push(w);
        }
    }

    // Search across all queries and all sets
    let all_prefixes = [COLORFUL_SETS, COLORFUL_SETS_2, ""];
    for query in &queries {
        for prefixes in &all_prefixes {
            if candidates.len() >= limit {
                break;
            }

            let url = if prefixes.is_empty() {
                format!("{}/search?query={}&limit=15", API_URL, urlencoding(query))
            } else {
                format!("{}/search?query={}&limit=15&prefixes={}", API_URL, urlencoding(query), prefixes)
            };

            let resp = match client.get(&url).send().await {
                Ok(r) if r.status().is_success() => r,
                _ => continue,
            };
            let data: SearchResponse = match resp.json().await {
                Ok(d) => d,
                Err(_) => continue,
            };

            for icon_id in &data.icons {
                if candidates.len() >= limit {
                    break;
                }

                let prefix = icon_id.split(':').next().unwrap_or("");
                let name_part = icon_id.split(':').nth(1).unwrap_or("");

                // Skip wordmarks and text-heavy variants
                let has_text = TEXT_INDICATORS.iter().any(|s| name_part.contains(s));
                if has_text {
                    continue;
                }

                if let Some(png_url) = icon_to_png_url(icon_id) {
                    if seen_urls.insert(png_url.clone()) {
                        candidates.push(IconCandidate {
                            icon_id: icon_id.clone(),
                            url: png_url,
                            set_name: prefix.to_string(),
                        });
                    }
                }
            }
        }
    }

    candidates
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct IconCandidate {
    pub icon_id: String,
    pub url: String,
    pub set_name: String,
}

/// URL-encode for query parameters (keeps spaces as %20)
fn urlencoding(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '~' => c.to_string(),
            ' ' => "%20".to_string(),
            _ => format!("%{:02X}", c as u32),
        })
        .collect()
}

/// Full URL encoding (encodes everything except unreserved chars)
fn urlencoding_full(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '~' => c.to_string(),
            _ => format!("%{:02X}", c as u32),
        })
        .collect()
}

// ═══ Tauri Commands ═══

/// Search for icon candidates — called from frontend icon picker popup
#[command]
pub async fn search_icons(query: String) -> Result<Vec<IconCandidate>, String> {
    Ok(fetch_icon_candidates(&query, 12).await)
}

/// Set a specific icon for an app — called when user picks from popup
#[command]
pub fn set_app_icon(exe_name: String, icon_url: String) -> Result<(), String> {
    let mut config = super::config::load_config_internal();
    let builtin = super::config::get_builtin_apps();

    if let Some(app) = config.apps.get_mut(&exe_name) {
        // App already in user config
        app.large_image_key = Some(icon_url);
    } else if let Some(builtin_app) = builtin.get(&exe_name) {
        // App is from builtin DB — clone to user config with chosen icon
        let mut app_cfg = builtin_app.clone();
        app_cfg.large_image_key = Some(icon_url);
        config.apps.insert(exe_name, app_cfg);
    } else {
        return Err("App not found".to_string());
    }

    super::config::save_config(config)?;
    Ok(())
}
