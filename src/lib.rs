use std::path::PathBuf;
use futures::stream::TryStreamExt;

use rspotify::{prelude::BaseClient, AuthCodeSpotify, model::*,};
mod controlclient;

pub async fn tsar_run(_output_dir: PathBuf, uri: &String, cache_dir: PathBuf, _username: &String, _empty_playlist: u8) {
    let spotify_api = controlclient::create_playback_client(cache_dir).await;

    let _tracks: Vec<FullTrack>  = find_playlist_tracks(&spotify_api, uri).await;


}

fn get_track_from_playable(item: &PlayableItem) -> &track::FullTrack {
    match item {
        PlayableItem::Track(t) => t,
        _ => panic!("Unable to handle episodes yet"),
    }
}

async fn find_playlist_tracks(spotify_api: &AuthCodeSpotify, uri: &String) -> Vec<FullTrack> {

    println!("getting all tracks from playlist {uri}");
    // Get all the tracks from the playlist
    let playlist_uri = PlaylistId::from_id_or_uri(uri).expect("unable to create playlist object from uri, is uri valid?");

    let playlist_paginator = spotify_api.playlist_items(playlist_uri, Option::from(None), Option::from(Market::FromToken));
    let playlist_items = playlist_paginator.try_collect::<Vec<_>>().await.expect("failed to get all tracks from playlist");

    let mut tracks: Vec<FullTrack> = Vec::<FullTrack>::new();

    for item in playlist_items {
        let tmp_playable = item.track.unwrap();
        let track = get_track_from_playable(&tmp_playable);

        let name = &track.name;
        let uri = tmp_playable.id().expect("failed to get tracks id/uri");
        println!("found track {name} {uri:?}");

        tracks.push(track.clone());
    }

    return tracks
}
