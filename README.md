# tsar - the spotify audio recorder

## requires:
```
librespot
 - built from https://github.com/SolidHal/librespot with fix for the pipe backend
spotipy
eyed3
click
```

## usage:

normal usage:
```
./tsar.py --output_dir="out" --playlist_id="spotify:playlist:<rand>" --username="<username>" --password="<password>"
```

empty the playlist once complete:
```
./tsar.py --output_dir="out" --playlist_id="spotify:playlist:<rand>" --username="<username>" --password="<password>" --empty_playlist
```

use a custom build of librespot:
```
./tsar.py --output_dir="out" --playlist_id="spotify:playlist:<rand>" --username="<username>" --password="<password>" --librespot_binary="<path/to/binary>"
```

