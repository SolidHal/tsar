use std::{path::PathBuf, time::Duration};
use futures::stream::TryStreamExt;

use rspotify::{model::*, prelude::{BaseClient, OAuthClient}, AuthCodeSpotify, ClientError};
use std::process::{Command, Child};
use tempfile;
mod controlclient;


#[derive(Debug)]
pub struct TsarError {
    kind: String,
    message: String,
}

impl From<ClientError> for TsarError {
    fn from(error: ClientError) -> Self {
        TsarError {
            kind: String::from("rspotify"),
            message: error.to_string(),
        }
    }
}

impl From<IdError> for TsarError {
    fn from(error: IdError) -> Self {
        TsarError {
            kind: String::from("uri/id Error"),
            message: error.to_string(),
        }
    }
}

impl From<&str> for TsarError {
    fn from(msg: &str) -> Self {
        TsarError {
            kind: String::from("tsar"),
            message: msg.to_string(),
        }
    }
}

impl From<String> for TsarError {
    fn from(msg: String) -> Self {
        TsarError {
            kind: String::from("tsar"),
            message: msg,
        }
    }
}

// TODO handle multiple uris at once
/// Main entrypoint
/// Inputs:
/// output_dir - Where to write mp3s to
/// uri -- spotify uri to a playlist, album, or track
/// cache_dr - Where to look for and store api cache files
/// recorder_binary_path - full filsystem path to the librespot binary
/// empty_playlist - Whether to remove all tracks from the provided playlist when complete. Ignored
///     when the uri is an album or track
pub async fn tsar_run(output_dir: &PathBuf, uri: &String, cache_dir: &PathBuf, recorder_binary_path: &PathBuf, empty_playlist: u8) -> Result<(), TsarError> {
    let spotify_api = controlclient::create_playback_client(&cache_dir).await;

    let tracks: Vec<FullTrack>;

    if uri_is_playlist(uri){
        tracks  = find_playlist_tracks(&spotify_api, uri).await?;
        print_tracks(&tracks);
    }
    else if uri_is_album(uri) {
        tracks  = find_album_tracks(&spotify_api, uri).await?;
        print_tracks(&tracks);
    }
    else if uri_is_track(uri) {
        let track = find_track_from_uri(&spotify_api, uri).await?;
        tracks = vec![track];
        print_tracks(&tracks);
    }
    else {
        return Err(format!("Unable to handle uri {uri}. uri should be for an album <spotify:album:blah> or a playlist <spotify:playlist:blah>").into());
    }

    println!("number of tracks = {num}", num = tracks.len());
    if tracks.len() <= 0 {
        println!("no tracks to process, quitting...");
        return Ok(());
    }


    let workdir = tempfile::tempdir().expect("failed to create temp workdir");
    let ogg_filename = workdir.path().join("raw_file.ogg");
    let device_name = "_comp_";
    let mut recorder = start_recorder(&ogg_filename, device_name, &cache_dir, recorder_binary_path).await;
    let recorder_device_id = find_device_id(&spotify_api, device_name).await?;

    for track in tracks {
        // TODO
        // play the song
        play_song(&spotify_api, &recorder_device_id, &track).await?;
        // process the song from ogg to mp3 format
        // move track to out
        // cleanup tmps
    }

    //TODO remove
    tokio::time::sleep(Duration::from_secs(10)).await;


    let mp3_filename = workdir.path().join("untagged_song.mp3");
    let mut completed_tracks: Vec<FullTrack> = Vec::<FullTrack>::new();

    // clean up recorder
    let _ = recorder.kill();



    if uri_is_playlist(uri) && empty_playlist != 0 {
        // empty the playlist
        let playlist_uri = PlaylistId::from_id_or_uri(uri)?;
        let mut playlist_uris = Vec::<PlayableId>::new();
        for track in completed_tracks {
            let id = track.id.expect("failed to get id");
            let playable = PlayableId::from(id);
            playlist_uris.push(playable);
        }
        spotify_api.playlist_remove_all_occurrences_of_items(playlist_uri, playlist_uris, None).await?;
    }

    return Ok(());
}

fn uri_is_playlist(uri: &String) -> bool {
    if uri.contains("spotify:playlist"){
        return true;
    }
    return false;
}

fn uri_is_album(uri: &String) -> bool {
    if uri.contains("spotify:album"){
        return true;
    }
    return false;
}

fn uri_is_track(uri: &String) -> bool {
    if uri.contains("spotify:track"){
        return true;
    }
    return false
}

fn get_track_from_playable(item: &PlayableItem) -> &track::FullTrack {
    match item {
        PlayableItem::Track(t) => t,
        _ => panic!("Unable to handle episodes yet"),
    }
}

fn print_tracks(tracks: &Vec<FullTrack>){
    for track in tracks{
        let name = &track.name;
        let uri = track.id.as_ref().expect("failed to get track uri").id();
        println!("track {name} {uri}");
    }
}

/// Get all the tracks in the playlist provided via uri
async fn find_playlist_tracks(spotify_api: &AuthCodeSpotify, uri: &String) -> Result<Vec<FullTrack>, TsarError> {
    let playlist_uri = PlaylistId::from_id_or_uri(uri)?;

    let playlist_paginator = spotify_api.playlist_items(playlist_uri, None, Some(Market::FromToken));
    let playlist_items = playlist_paginator.try_collect::<Vec<_>>().await?;

    let mut tracks: Vec<FullTrack> = Vec::<FullTrack>::new();

    for item in playlist_items {
        let Some(tmp_playable) = item.track else {
            println!("SKIPPING TRACK which does not contain playable item");
            continue;
        };
        let track = get_track_from_playable(&tmp_playable);

        tracks.push(track.clone());
    }

    return Ok(tracks);
}

/// Get all the tracks in the album provided via uri
async fn find_album_tracks(spotify_api: &AuthCodeSpotify, uri: &String) -> Result<Vec<FullTrack>, TsarError> {
    // Get all the tracks from the album
    let album_uri = AlbumId::from_id_or_uri(uri)?;

    // album_track gives us SimplifiedTrack objects, but playlist_items gives us FullTrack objects
    // convert all SimplifiedTrack objects by asking for each track in the album by uri
    let album_paginator = spotify_api.album_track(album_uri, Some(Market::FromToken));
    let album_items = album_paginator.try_collect::<Vec<_>>().await?;

    let mut album_uris: Vec<TrackId> = Vec::<TrackId>::new();
    for item in album_items {
        let Some(uri) = item.id else {
            panic!("album track does not have an id");
        };
        album_uris.push(uri);
    }

    let album_tracks = spotify_api.tracks(album_uris, Some(Market::FromToken)).await?;

    return Ok(album_tracks);
}

/// Lookup a FullTrack object provided a track uri
async fn find_track_from_uri(spotify_api: &AuthCodeSpotify, uri: &String) -> Result<FullTrack, TsarError> {
    let id = TrackId::from_id_or_uri(uri)?;
    return Ok(spotify_api.track(id, Some(Market::FromToken)).await?);
}


/// Provided a device_name, returns the associated device_id
/// Returns error if no device with the provided device_name was found
async fn find_device_id(spotify_api: &AuthCodeSpotify, device_name: &str) -> Result<String, TsarError>{

    let mut device_id = None;
    let retry_count = 0;

    let mut devices: Vec<Device>;
    while device_id.is_none() {
        devices = spotify_api.device().await?;
        for dev in &devices {
            if dev.name == *device_name {
                device_id = dev.id.clone();
                println!("found device {device_name} {device_id:?}");
                let id = device_id.expect("device matching name was found, but it does not contain a device id");
                return Ok(id);
            }
        }
        if retry_count < 5 {
            // failed to find the device, lets wait and try again in a moment
            println!("didn't find {device_name} in {devices:?}, trying again in a moment...");
            tokio::time::sleep(Duration::from_secs(10)).await;
        }
        else {
            return Err(format!("Failed to find device with name {device_name}. Found devices {devices:?}").into());
        }
    }
    return Err(format!("Failed to find device with name {device_name}.").into());
}

/// Spawns librespot configured to output played tracks to a file
async fn start_recorder(output_filename: &PathBuf, device_name: &str, cache_dir: &PathBuf, recorder_binary_path: &PathBuf) -> Child {
    let mut cmd = Command::new(recorder_binary_path);
    cmd.args(["--name", device_name,
        "--bitrate", "320",
        "--system-cache", cache_dir.to_str().expect("failed to convert cache_dir to string"),
        "--device-type", "computer",
        "--initial-volume", "100",
        "--disable-audio-cache",
        "--disable-gapless",
        "--backend", "pipe",
        "--passthrough",
        "--autoplay", "off",
        "--format", "S24",
        "--dither", "none",
        "--device", output_filename.to_str().expect("failed to convert output_filename to string")]);
    println!("starting recorder with command {prog:?} {args:?}", prog = cmd.get_program(), args = cmd.get_args());

    let recorder = cmd.spawn().expect("failed to start librespot");
    // let recorder warm up
    tokio::time::sleep(Duration::from_secs(20)).await;


    return recorder;
}


/// Plays the Requested Track
/// sleeps while the track is playing
/// returns once playback is complete
async fn play_song(spotify_api: &AuthCodeSpotify, device_id: &str, track: &FullTrack) -> Result<(), TsarError> {
    let uri = track.id.clone().expect("Cannot play track without id");
    let uri_list = vec!(PlayableId::from(uri));
    spotify_api.start_uris_playback(uri_list, Some(device_id), None, None).await?;

    while spotify_api.current_playing(Some(Market::FromToken),  None::<&[_]>).await
        .expect("Unable to get current play status from spotify").is_none() {
        println!("wating for playback to start");
        tokio::time::sleep(Duration::from_secs(1)).await;
    }

    while let Some(cur_playing) = spotify_api.current_playing(Some(Market::FromToken), None::<&[_]>).await.expect("Unable to get current play status from spotify") {
        let cur_playable = cur_playing.item.expect("Something is playing, but we are unsure what!");
        let cur_song = get_track_from_playable(&cur_playable);
        if track.id.as_ref().expect("Unable to get id from passed in track")
                .eq(cur_song.id.as_ref().expect("Current playing track doesn't have an id")) {
            return Err(format!("Music is playing, but it isn't playing our requested song").into());
        }
        println!("song is playing...");
        tokio::time::sleep(Duration::from_secs(1)).await;
    }

    // let the recorders decoder finish up
    tokio::time::sleep(Duration::from_secs(2)).await;
    println!("song is done!");

    return Ok(());

}
