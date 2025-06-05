# tsar - the spotify audio recorder

## requires:
```
librespot
 - build with `cargo build --features passthrough-decoder`
spotipy
eyed3
click
ffmpeg
```

## Build
```
# start dev env
nix develop
# build
cargo build
```

## usage:

normal usage:
```
./tsar.py --output_dir="out" --playlist_id="spotify:playlist:<rand>" --username="<username>" --cache_dir="cache_dir"
```

empty the playlist once complete:
```
./tsar.py --output_dir="out" --playlist_id="spotify:playlist:<rand>" --username="<username>" --cache_dir="cache_dir" --empty_playlist
```

use a custom build of librespot:
```
./tsar.py --output_dir="out" --playlist_id="spotify:playlist:<rand>" --username="<username>" --cache_dir="cache_dir" --librespot_binary="<path/to/binary>"
```



## Create cached credentials

for librespot (the player portion)
https://github.com/librespot-org/librespot/wiki/Options#headless-oauth
```
librespot --cache cache_dir --enable-oauth --oauth-port 0
```

for rspotify (the controller portion)

```
./tsar_create_credentials
```
this will create `.spotify_token_cache.json`

put it in the cache_dir privided to tsar
