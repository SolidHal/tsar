mod controlclient;


#[tokio::main]
async fn main() {
    let spotify = controlclient::create_playback_client().await;
    println!("spotify token cache file should now be available in {cache_path}", cache_path=spotify.config.cache_path.display())
}
