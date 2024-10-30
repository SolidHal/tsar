# tsar - the spotify audio recorder

## requires:
```
librespot
 - built with --features passthrough-decoder
spotipy
eyed3
click
ffmpeg
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

