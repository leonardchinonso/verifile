use crate::args::Action;
use clap::Parser;
use env_logger::Builder;
use log::{info, LevelFilter};
use std::error::Error;

mod args;
mod client;

fn main() -> Result<(), Box<dyn Error>> {
    // initialize the logger
    Builder::new().filter(None, LevelFilter::Info).init();

    // get arguments from CLI
    let mut args = args::Argument::parse();
    info!("{:?}", args);

    // create a client
    let mut client = client::Client::new();

    match args.action() {
        Action::Send => {
            args.validate_file_names()?;
            client.add_files(args.file_names()); // add the files to the client
            client.process_files(); // compute the merkle root and prep the files
            client.send_files(); // send the files to the server
        }
        Action::Download(n) => {
            client.download_file(n)?;
        }
    }

    Ok(())
}
