// creates rspotify clients for controling playback and retreiving metadata

use std::path::PathBuf;

use chrono::offset::Utc;
use chrono::Duration;
use rspotify::{
    prelude::*, scopes, AuthCodeSpotify,
    Config, Credentials, OAuth, DEFAULT_CACHE_PATH
};

pub async fn create_playback_client(token_cache_dir: &PathBuf) -> AuthCodeSpotify {

    let token_cache_path = token_cache_dir.join(DEFAULT_CACHE_PATH);

    // Enable token refreshing and caching so we can pass the saved token to the main tsar client
    let config = Config {
        token_cached: true,
        token_refreshing: true,
        cache_path: token_cache_path,
        ..Default::default()
    };
    // May require the `env-file` feature enabled if the environment variables
    // aren't configured manually.
    let creds = Credentials::from_env().unwrap();
    let oauth = OAuth::from_env(scopes!("user-read-private", "user-read-playback-state", "user-modify-playback-state")).unwrap();


    println!(">>> Session one, obtaining refresh token and running some requests:");
    let spotify = AuthCodeSpotify::with_config(creds.clone(), oauth, config.clone());
    let url = spotify.get_authorize_url(false).unwrap();

    // either read the local token file and referesh it, or prompt the user for a new token
    spotify
        .prompt_for_token(&url)
        .await
        .expect("couldn't authenticate successfully");

    with_auth(&spotify).await;

    return spotify;
}



// Sample request that will follow some artists, print the user's
// followed artists, and then unfollow the artists.
async fn auth_code_do_things(spotify: &AuthCodeSpotify) {
    let _ret = spotify.current_user_playing_item().await.expect("unable to get current playing song");
    // println!("currently playing {ret.unwrap():?}");
    println!("Successfully read current playing status")
}


async fn expire_token<S: BaseClient>(spotify: &S) {
    let token_mutex = spotify.get_token();
    let mut token = token_mutex.lock().await.unwrap();
    let token = token.as_mut().expect("Token can't be empty as this point");
    // In a regular case, the token would expire with time. Here we just do
    // it manually.
    let now = Utc::now().checked_sub_signed(Duration::try_seconds(10).unwrap());
    token.expires_at = now;
    token.expires_in = Duration::try_seconds(0).unwrap();
    // We also use a garbage access token to make sure it's actually
    // refreshed.
    token.access_token = "garbage".to_owned();
}

async fn with_auth(spotify: &AuthCodeSpotify) {
    // In the first session of the application we authenticate and obtain the
    // refresh token.

    // We can now perform requests
    auth_code_do_things(spotify).await;

    // Manually expiring the token.
    expire_token(spotify).await;

    // Without automatically refreshing tokens, this would cause an
    // authentication error when making a request, because the auth token is
    // invalid. However, since it will be refreshed automatically, this will
    // work.
    println!(">>> Session two, the token should expire, then re-auth automatically");
    auth_code_do_things(spotify).await;
}
