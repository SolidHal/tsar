use clap::Parser;
use std::path::PathBuf;
// use id3;
//
use tsar;

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

    /// path to the librespot binary
    #[arg(long)]
    librespot_binary_path: PathBuf,

}








#[tokio::main]
async fn main() {
    let args = Args::parse();

    println!("{}", args.uri);
    println!("{}", args.cache_dir.display());
    println!("{}", args.output_dir.display());
    println!("{}", args.librespot_binary_path.display());
    println!("{}", args.empty_playlist);

    tsar::tsar_run(&args.output_dir, &args.uri, &args.cache_dir, &args.librespot_binary_path, args.empty_playlist).await;
}
