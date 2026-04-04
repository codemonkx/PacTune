use serde_json::Value;

pub async fn fetch_lrc(artist: &str, title: &str) -> Option<String> {
    let client = reqwest::Client::builder()
        .user_agent("PacTune Player (https://github.com/codemonkx/VINLY)")
        .build()
        .unwrap();
    
    let response = client
        .get("https://lrclib.net/api/get")
        .query(&[("artist_name", artist), ("track_name", title)])
        .send()
        .await
        .ok()?;

    if response.status().is_success() {
        let json: Value = response.json().await.ok()?;
        
        if let Some(synced_lyrics) = json.get("syncedLyrics").and_then(|v| v.as_str()) {
            return Some(synced_lyrics.to_string());
        }
    }

    None
}
