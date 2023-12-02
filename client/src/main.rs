use crate::args::Action;
use clap::Parser;
use env_logger::Builder;
use log::{info, LevelFilter};
use std::error::Error;

mod args;
mod client;

fn main() -> Result<(), Box<dyn Error>> {
    Builder::new().filter(None, LevelFilter::Info).init();

    let mut args = args::Argument::parse();
    info!("{:?}", args);

    let mut client = client::Client::new();

    match args.action() {
        Action::Send => {
            args.validate()?;
            client.prepare_and_send_files(args.file_names());
        }
        Action::Download(n) => {
            client.download_verify_and_write_file(n)?;
        }
    }

    Ok(())
}
