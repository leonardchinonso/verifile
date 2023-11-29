use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use common::SERVER_ADDRESS;
use log::{error, info};

pub struct Server {

}

impl Server {
    pub fn new() -> Self {
        Self{}
    }

    fn handle_client(&self, mut stream: TcpStream) {
        let mut buffer = [0; 1024];
        let size = stream.read(&mut buffer).unwrap();
        let buf_content = &buffer[..size];
        let str_content = String::from_utf8_lossy(buf_content);
        println!("The contents of the file are: {}", str_content);

        let response = b"File received";
        stream.write_all(response).unwrap();
    }

    pub fn process(&self) {
        info!("Starting Server...");
        let listener = TcpListener::bind(SERVER_ADDRESS).unwrap();
        info!("Now Listening at: {}", SERVER_ADDRESS);
        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    info!("Client Connected");
                    self.handle_client(stream);
                }
                Err(e) => {
                    error!("Failed To Accept Connection: {}", e);
                }
            }
        }
    }
}
