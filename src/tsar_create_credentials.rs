use std::path::PathBuf;
mod controlclient;


#[tokio::main]
async fn main() {
    let cache_dir = PathBuf::from("");
    let spotify = controlclient::create_playback_client(&cache_dir).await;
    println!("spotify token cache file should now be available in {cache_path}", cache_path=spotify.config.cache_path.display())
}
