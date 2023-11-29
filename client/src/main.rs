use std::error::Error;
use env_logger::Builder;
use log::{info, LevelFilter};
use clap::Parser;

mod args;
mod client;

fn main() -> Result<(), Box<dyn Error>> {
    // initialize the logger
    Builder::new().filter(None, LevelFilter::Info).init();

    let mut args = args::Argument::parse();
    args.validate_file_name()?;

    info!("{:?}", args);

    let client = client::Client::new(Vec::from([args.file_name()]));
    client.process();

    Ok(())
}
