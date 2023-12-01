use common::model::merkle::{MerkleProof, MerkleTree};
use common::SERVER_ADDRESS;
use log::{error, info};
use std::fs::File;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};

#[derive(Debug, Clone)]
enum ServerState {
    Receive,
    Send,
}

pub struct Server {
    files_data: Vec<Vec<u8>>,
    state: ServerState,
    merkle_tree: MerkleTree,
}

impl Server {
    pub fn new() -> Self {
        Self {
            files_data: Vec::new(),
            state: ServerState::Receive,
            merkle_tree: MerkleTree::new(),
        }
    }

    fn handle_receive_files(&mut self, mut stream: TcpStream) {
        let mut buffer = [0; 8]; // buffer to hold the size of the file
        let mut count = 0usize; // count to hold the amount of files received

        loop {
            match stream.read_exact(&mut buffer) {
                Ok(_) => {
                    let size = u64::from_be_bytes(buffer);
                    if size == 0 {
                        break; // no more files to read
                    }

                    // Create a new file
                    let mut file = File::create(format!("{count}.txt")).unwrap();
                    count += 1;

                    // Buffer to hold file data
                    let mut file_buf = vec![0; size as usize];

                    // Read the file data
                    stream.read_exact(&mut file_buf).unwrap();

                    // Write the file data
                    file.write_all(&file_buf).unwrap();

                    // add the file to the list of files
                    self.files_data.push(file_buf);
                }
                Err(e) => {
                    error!("Error reading file contents: {e}");
                    panic!("{}", e); // broken pipe or connection timeout.. should not happen
                }
            }
        }

        // build a merkle tree from the list of file data
        self.merkle_tree = MerkleTree::from(self.files_data.clone());

        self.state = ServerState::Send;
    }

    fn handle_send_file(&self, mut stream: TcpStream) {
        let mut buffer = [0; 8]; // buffer to hold the index of the file to send

        match stream.read_exact(&mut buffer) {
            Ok(_) => {
                let index = u64::from_be_bytes(buffer);

                // open the file
                let mut file = File::open(format!("{}.txt", index)).expect("File is invalid");

                // create a buffer to read the file into
                let mut file_buf = Vec::new();
                file.read_to_end(&mut file_buf).unwrap();

                let mp = MerkleProof::build(&self.merkle_tree, index as usize, file_buf.clone());
                let json_proof = mp.to_string();

                // write the file contents to the server over the stream
                stream
                    .write_all(json_proof.as_bytes())
                    .expect("Failed to send file content to client");
            }
            Err(e) => {
                error!("Error reading file index: {e}");
                panic!("{}", e); // broken pipe or connection timeout.. should not happen
            }
        }
    }

    pub fn start(&mut self) {
        info!("Starting Server...");
        let listener = TcpListener::bind(SERVER_ADDRESS).unwrap();
        info!("Now Listening at: {}", SERVER_ADDRESS);

        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    match self.state.clone() {
                        ServerState::Receive => self.handle_receive_files(stream),
                        ServerState::Send => self.handle_send_file(stream),
                    };
                }
                Err(e) => {
                    error!("Failed to accept connection: {}", e);
                }
            }
        }
    }
}
