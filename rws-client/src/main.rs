mod app;
mod cli;
mod client;
mod ui;

use anyhow::Result;
use clap::Parser;

#[tokio::main]
async fn main() -> Result<()> {
    let args = cli::Args::parse();
    
    let mut app = app::App::new(args.username, args.server)?;
    ui::run(&mut app).await?;
    
    Ok(())
}
