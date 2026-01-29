use clap::Parser;
use std::sync::Arc;
use std::{error::Error, process};
use tiko::download_from_url;
use tokio::sync::Semaphore;
use tokio::task::JoinSet;

#[tokio::main]
async fn main() {
    let args = Args::parse();

    if let Err(e) = run(args).await {
        eprintln!("Application error: {e}");
        process::exit(1);
    }
}

async fn run(args: Args) -> Result<(), Box<dyn Error>> {
    let mut set = JoinSet::new();

    let semaphore = Arc::new(Semaphore::new(1));

    for url in args.links {
        let semaphore = Arc::clone(&semaphore);
        set.spawn(async move { download_from_url(&url, semaphore).await });
    }

    while let Some(res) = set.join_next().await {
        if let Err(e) = res? {
            eprintln!("Error downloading: {}", e);
        }
    }

    Ok(())
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    pub links: Vec<String>,
}
