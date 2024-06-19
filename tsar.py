#!/usr/bin/env python3
import os
import sys
import json
import spotipy
import spotipy.util as util
import eyed3
import click
import urllib.request
import re
import time
import subprocess
import shutil
from json.decoder import JSONDecodeError

def remove_file(filename):
    try:
        os.remove(filename)
    except FileNotFoundError:
        pass

def sanitize_filename(filename):
    """Takes only a filename, not a full path"""
    return re.sub('/', ' ', filename).strip()

def start_recorder(output_filename, device_name, username, password, binary_arg):
    # setup recorder
    generic_args = ["-n", device_name,
                    "-b", "320",
                    "--device-type", "computer",
                    "--initial-volume", "100",
                    "--disable-credential-cache",
                    "--disable-audio-cache",
                    "--disable-gapless",
                    "--backend", "pipe",
                    "--passthrough" ]

    output_arg = ["--device", output_filename]

    command = [binary_arg, "-u", username, "-p", password] + generic_args + output_arg
    print("starting recorder with command: ")
    print(command)
    recorder = subprocess.Popen(command, shell=False)

    # let recorder warm up
    time.sleep(3)

    return recorder

def start_api(username):
    """
    the following must be set:
    SPOTIPY_CLIENT_ID
    SPOTIPY_CLIENT_SECRET
    SPOTIPY_REDIRECT_URI
    """
    scope = 'user-read-private user-read-playback-state user-modify-playback-state'

    try:
        token = util.prompt_for_user_token(username, scope)
    except (AttributeError, JSONDecodeError):
        os.remove(f".cache-{username}")
        token = util.prompt_for_user_token(username, scope)

    spotify_api = spotipy.Spotify(auth=token, retries=10, status_retries=10, backoff_factor=1.5)

    return spotify_api


def find_device_id(spotify_api, device_name):
    device_id = None
    retry_count = 0
    while(device_id is None):
        devices = spotify_api.devices()
        for dev in devices['devices']:
            print(dev["name"])
            if dev["name"] == device_name:
                print("using device:")
                print(dev)
                device_id = dev["id"]

        if device_id is None and retry_count >= 5:
            raise ValueError(f"could not find device after 5 retries, available devices are: {json.dumps(devices, sort_keys=True, indent=4)}")
        elif device_id is None and retry_count < 5:
            print(f"could not find device on retry {retry_count}, available devices are: {json.dumps(devices, sort_keys=True, indent=4)}")
            print("sleeping before trying again")
            time.sleep(30)
        else:
            return device_id

    return device_id

def find_playlist_tracks(spotify_api, playlist_uri):
    tracks = []
    max_track_limit = 100
    playlist = spotify_api.playlist_items(playlist_uri, limit=max_track_limit, additional_types=('track', ))
    playlist_size = playlist.get("total")
    tracks += playlist.get("items")
    print(f"playlist size is: {playlist_size}")

    # since the api limits us to ~100 tracks at a time, concatonate our requests
    offset = max_track_limit
    end = playlist_size
    while(offset < playlist_size):
        playlist = spotify_api.playlist_items(playlist_uri, limit=max_track_limit, offset=offset, additional_types=('track', ))
        tracks += playlist.get("items")
        offset += max_track_limit

    if(playlist_size != len(tracks)):
        raise ValueError(f"playlist has {playlist_size} songs but only got {len(tracks)}")

    # remove the playlist metadata
    unwrapped_tracks = []
    for track in tracks:
        unwrapped_tracks.append(track.get("track"))

    return unwrapped_tracks

def find_album_tracks(spotify_api, album_uri):
    tracks = []
    max_track_limit = 50
    album = spotify_api.album_items(album_uri)
    album_size = album.get("total")
    tracks += album.get("items")
    print(f"album size is: {album_size}")

    # since the api limits us to ~100 tracks at a time, concatonate our requests
    offset = max_track_limit
    end = album_size
    while(offset < album_size):
        playlist = spotify_api.album_tracks(album_uri, limit=max_track_limit, offset=offset)
        tracks += playlist.get("items")
        offset += max_track_limit

    if(album_size != len(tracks)):
        raise ValueError(f"album has {album_size} songs but only got {len(tracks)}")

    return tracks

def play_song(spotify_api, device_id, track_uri):
    trackSelectionList = []
    trackSelectionList.append(track_uri)
    spotify_api.start_playback(device_id, None, trackSelectionList)

    while(spotify_api.current_playback() is None):
        print("waiting to get status...")

    while(spotify_api.current_playback().get("is_playing")):
        print("song is playing...")
        time.sleep(2)

    # let the recorders decoder finish up
    time.sleep(2)
    print("song is done!")

def convert_song(input_filename, output_filename):
    print("converting song...")
    ffmpeg = subprocess.run(["ffmpeg", "-hide_banner" ,"-i", input_filename, "-b:a", "320k", output_filename])

def set_song_metadata(track, input_filename):
    def artists(artists_list):
        """Takes a list of artists and formats them for tagging"""
        artists_str = artists_list[0].get("name")
        additional_artists = artists_list[1:]
        for artist in additional_artists:
            artists_str += "; "
            artists_str += artist.get("name")
        return artists_str

    def album_art_url(track):
        images = track.get("album").get("images")
        for image in images:
            if image.get("height") == 640:
                return image.get("url")

        print("could not find large album art, trying smaller size")
        for image in images:
            if image.get("height") == 300:
                return image.get("url")
        raise ValueError(f"could not find suitable album art image in images: {images}")

    def canonical_artist(track):
        track_artist = track.get("artists")[0].get("name", "Unknown Artist")
        album_artist = track.get("album").get("artists")[0].get("name", "Unknown Artist")

        if track_artist != album_artist:
            # if the album artist is generic, just use the track artist
            if "Various Artists" in album_artist:
                return track_artist
            raise ValueError(f"could not determine canonical artist, track_artist = {track_artist}, album_artist = {album_artist}")

        return track_artist

    if not track.get("uri"):
        raise ValueError("track should be unwrapped first")

    audiofile = eyed3.load(input_filename)
    audiofile.tag.artist = artists(track.get("artists"))
    audiofile.tag.album = track.get("album").get("name")
    audiofile.tag.album_artist = artists(track.get("album").get("artists"))
    audiofile.tag.title = track.get("name")
    audiofile.tag.track_num = track.get("track_number")

    album_art = None
    url = album_art_url(track)
    with urllib.request.urlopen(url) as response:
        album_art = response.read()

    if album_art is None:
        raise ValueError(f"unable to get album art from url {url}")

    audiofile.tag.images.set(3, img_data=album_art, mime_type="image/jpeg")
    audiofile.tag.save()

    artist = sanitize_filename(canonical_artist(track))
    title = sanitize_filename(track.get("name"))
    return f"{artist} - {title}.mp3"


def run(output_dir, uri, username, password, empty_playlist, librespot_binary):
    ogg_filename = "/tmp/raw_file.ogg"
    mp3_filename = "/tmp/untagged_song.mp3"
    device_name = "_comp_"

    def cleanup_files():
        remove_file(ogg_filename)
        remove_file(mp3_filename)

    def finish(recorder):
        cleanup_files()
        recorder.kill()

    cleanup_files()
    if not os.path.isdir(output_dir):
        os.makedirs(output_dir, exist_ok=True)

    # setup our apis
    spotify_api = start_api(username)
    recorder = start_recorder(ogg_filename, device_name, username, password, librespot_binary)
    recorder_device_id = find_device_id(spotify_api, device_name)

    if "playlist" in uri:
        # get tracklist from known playlist
        tracks = find_playlist_tracks(spotify_api, uri)
    if "album" in uri:
        # get tracklist from known album
        tracks = find_album_tracks(spotify_api, uri)

    print(f"number of tracks = {len(tracks)}")
    if len(tracks) == 0:
        print("no tracks to process, quitting tsar...")
        return

    for track in tracks:
        play_song(spotify_api, recorder_device_id, track.get("uri"))
        # process the song
        # recorder outputs to ogg_filename
        convert_song(ogg_filename, mp3_filename)
        song_name = set_song_metadata(track, mp3_filename)
        out = f"{output_dir}/{song_name}"
        print(f"moving song to {out}")
        shutil.move(mp3_filename, out)
        cleanup_files()

    # cleanup
    finish(recorder)
    print(f"recorder return code = {recorder.returncode}")
    if recorder.returncode:
        raise ValueError(f"Error processing song using tsar")

    # validate that all tracks were properly downloaded
    filenames = next(os.walk(output_dir), (None, None, []))[2]  # [] if no file
    if len(tracks) != len(filenames):
        raise ValueError(f"""Expected {len(tracks)} songs, but found {len(filenames)}.
                             expected list: {tracks}
                             found list: {filenames}""")

    if empty_playlist:
        if "album" in uri:
            print(f"ignoring empty_playlist flag as we are working with an album")
        else:
            print(f"removing {len(tracks)} songs from playlist {uri}")
            uris = []
            for track in tracks:
                uris.append(track.get("uri"))
            spotify_api.playlist_remove_all_occurrences_of_items(uri, uris)


    print(f"tsar finished. {len(tracks)} songs from playlist {uri}")

@click.command()
@click.option("--output_dir", type=str, required=True, help="location to save the songs to")
@click.option("--uri", type=str, required=True, help="playlist or album uri to record, of the form spotify:playlist:<rand> or spotify:album:<rand>")
@click.option("--username", type=str, required=True, help="username of the user to login as")
@click.option("--password", type=str, required=True, help="password of the user to login as")
@click.option("--empty_playlist", is_flag=True, default=False, help="remove all songs from the playlist when complete")
@click.option("--librespot_binary", type=str, default="librespot", help="path to the librespot binary")
def main(output_dir, uri, username, password, empty_playlist, librespot_binary):
    run(output_dir=output_dir,
        uri=uri,
        username=username,
        password=password,
        empty_playlist=empty_playlist,
        librespot_binary=librespot_binary)

if __name__ == "__main__":
    main()

