use std::{error::Error, process};

use clap::Parser;
use tiko::download_from_url;

#[tokio::main]
async fn main() {
    let args = Args::parse();

    if let Err(e) = run(args).await {
        eprintln!("Application error: {e}");
        process::exit(1);
    }
}

async fn run(args: Args) -> Result<(), Box<dyn Error>> {
    download_from_url(&args.link).await?;

    Ok(())
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    pub link: String,
}
