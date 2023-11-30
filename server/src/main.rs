use env_logger::Builder;
use log::LevelFilter;

mod server;

fn main() {
    // initialize the logger
    Builder::new().filter(None, LevelFilter::Info).init();

    let mut server = server::Server::new();
    server.start();
}
