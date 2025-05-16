// TODO import librespot (playback)
// TODO import rspotify (control)
// TODO import id3 (mp3 id3 tags)
// TODO import clap cli lib
use clap::Parser;
use std::path::PathBuf;

// TODO just shell out to ffmpeg
// std::process::Command("ffmpeg")

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {

    /// Remove all songs from the playlist when complete
    #[arg{long, action=clap::ArgAction::Count}]
    empty_playlist: u8,

    /// location to save the songs to
    #[arg(long)]
    output_dir: PathBuf,

    /// playlist or album uri to record, of the form spotify:playlist:<rand> or spotify:album:<rand>
    #[arg(long)]
    uri: String,

    /// directory to store cached credentials.json
    #[arg(long)]
    cache_dir: PathBuf,

    /// username of the user to login as
    #[arg{long}]
    username: String,

}

fn main() {
    let args = Args::parse();

    println!("{}", args.uri);
    println!("{}", args.cache_dir.display());
    println!("{}", args.output_dir.display());
    println!("{}", args.username);
    println!("{}", args.empty_playlist);
}
