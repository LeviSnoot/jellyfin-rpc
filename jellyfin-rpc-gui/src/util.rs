use jellyfin_rpc::{config::get_urls_path, Client, ClientBuilder};

use crate::form::Form;

/// Runs blocking code (the jellyfin client is blocking) off the UI thread.
pub async fn blocking<T: Send + 'static>(f: impl FnOnce() -> T + Send + 'static) -> T {
    tokio::task::spawn_blocking(f)
        .await
        .expect("blocking task panicked")
}

/// Builds a client from the (possibly unsaved) form, only used for talking to jellyfin.
pub fn client_for(form: &Form) -> Result<Client, String> {
    let mut form = form.clone();

    if form.url.trim().is_empty() || form.api_key.trim().is_empty() {
        return Err("Set the Jellyfin URL and API key first".to_string());
    }

    // Usernames are only needed for live sessions
    if form.usernames.is_empty() {
        form.usernames.push("_".to_string());
    }

    let urls_path = get_urls_path().unwrap_or_default();

    ClientBuilder::from_config(form.to_builder().build(), &urls_path)
        .build()
        .map_err(|e| e.to_string())
}

/// Url of a jellyfin item's primary image, resized to fit a thumbnail.
pub fn thumbnail_url(base: &str, id: &str) -> String {
    let base = base.trim();
    let sep = if base.ends_with('/') { "" } else { "/" };
    format!("{base}{sep}Items/{id}/Images/Primary?maxHeight=96")
}
