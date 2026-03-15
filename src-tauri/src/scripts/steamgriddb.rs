use serde::Deserialize;

const BASE_URL: &str = "https://www.steamgriddb.com/api/v2";

#[derive(Deserialize, Debug)]
struct SearchResponse {
    success: bool,
    data: Vec<SearchResult>,
}

#[derive(Deserialize, Debug)]
struct SearchResult {
    id: u64,
    #[allow(dead_code)]
    name: String,
}

#[derive(Deserialize, Debug)]
struct IconResponse {
    success: bool,
    data: Vec<IconResult>,
}

#[derive(Deserialize, Debug)]
struct IconResult {
    url: String,
    #[allow(dead_code)]
    thumb: String,
    width: u32,
    height: u32,
}

/// Search SteamGridDB for an app/game by name and return the best icon URL
pub async fn fetch_icon(app_name: &str, api_key: &str) -> Option<String> {
    if api_key.is_empty() || app_name.is_empty() {
        return None;
    }

    let client = reqwest::Client::new();

    // Step 1: Search for the app
    let search_url = format!("{}/search/autocomplete/{}", BASE_URL, urlencoding(app_name));
    let search_resp = client
        .get(&search_url)
        .header("Authorization", format!("Bearer {}", api_key))
        .send()
        .await
        .ok()?;

    if !search_resp.status().is_success() {
        return None;
    }

    let search_data: SearchResponse = search_resp.json().await.ok()?;
    if !search_data.success || search_data.data.is_empty() {
        return None;
    }

    let game_id = search_data.data[0].id;

    // Step 2: Get icons for this game
    let icons_url = format!("{}/icons/game/{}", BASE_URL, game_id);
    let icons_resp = client
        .get(&icons_url)
        .header("Authorization", format!("Bearer {}", api_key))
        .send()
        .await
        .ok()?;

    if !icons_resp.status().is_success() {
        return None;
    }

    let icons_data: IconResponse = icons_resp.json().await.ok()?;
    if !icons_data.success || icons_data.data.is_empty() {
        return None;
    }

    // Pick the best icon: prefer PNG, reasonable size
    let best = icons_data
        .data
        .iter()
        .filter(|i| i.width >= 64 && i.height >= 64)
        .min_by_key(|i| {
            // Prefer icons closest to 256x256
            let diff = (i.width as i64 - 256).abs() + (i.height as i64 - 256).abs();
            diff
        })
        .or_else(|| icons_data.data.first());

    best.map(|i| i.url.clone())
}

/// Simple URL encoding for the search term
fn urlencoding(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '~' => c.to_string(),
            ' ' => "%20".to_string(),
            _ => format!("%{:02X}", c as u32),
        })
        .collect()
}
