use env_logger::Builder;
use log::LevelFilter;

mod server;

fn main() {
    // initialize the logger
    Builder::new().filter(None, LevelFilter::Info).init();

    let server = server::Server::new();
    server.process();
}
