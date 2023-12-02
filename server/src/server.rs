use common::model::file_info::FileInfo;
use common::model::merkle::{MerkleProof, MerkleTree};
use common::SERVER_ADDRESS;
use log::{error, info};
use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};

#[derive(Debug, Clone)]
enum ServerState {
    Receive,
    Send,
}

pub struct Server {
    store: HashMap<usize, FileInfo>,
    state: ServerState,
    merkle_tree: MerkleTree,
}

impl Server {
    pub fn new() -> Self {
        Self {
            store: HashMap::new(),
            state: ServerState::Receive,
            merkle_tree: MerkleTree::new(),
        }
    }

    // /// handle_receive_and_store_files receives files from a TCP stream,
    // /// stores them, and updates the server's state and Merkle tree
    // fn handle_receive_and_store_files(&mut self, mut stream: TcpStream) {
    //     let mut buffer = [0; 8];
    //     let mut count = 0usize;
    //
    //     loop {
    //         match stream.read_exact(&mut buffer) {
    //             Ok(_) => {
    //                 let size = u64::from_be_bytes(buffer);
    //                 if size == 0 {
    //                     break;
    //                 }
    //
    //                 let mut file = File::create(format!("{count}.txt")).unwrap();
    //                 let mut file_buf = vec![0; size as usize];
    //                 stream.read_exact(&mut file_buf).unwrap();
    //                 file.write_all(&file_buf).unwrap();
    //                 self.files_data.push(file_buf);
    //                 count += 1;
    //             }
    //             Err(e) => {
    //                 error!("Error reading file contents: {e}");
    //                 panic!("{}", e); // broken pipe or connection timeout.. should not happen
    //             }
    //         }
    //     }
    //
    //     self.merkle_tree = MerkleTree::from(self.files_data.clone());
    //     self.state = ServerState::Send;
    // }

    /// handle_receive_and_store_files receives files from a TCP stream,
    /// stores them, and updates the server's state and Merkle tree
    fn handle_receive_and_store_files(&mut self, mut stream: TcpStream) {
        let mut json_buf = Vec::new();
        stream
            .read_to_end(&mut json_buf)
            .expect("should read downloaded files");

        let files_info: Vec<FileInfo> =
            serde_json::from_slice(&json_buf).expect("should deserialize downloaded files");

        let files_data = files_info
            .iter()
            .map(|file_info| file_info.content())
            .collect::<Vec<Vec<u8>>>();
        self.merkle_tree = MerkleTree::from(files_data.clone());

        self.store = files_info
            .into_iter()
            .fold(HashMap::new(), |mut h, file_info| {
                h.insert(file_info.index(), file_info);
                h
            });

        self.state = ServerState::Send;
    }

    /// handle_send_file_with_merkle_proof Reads a file index from a TCP stream,
    /// builds a Merkle proof for the file, and sends the proof over the stream
    fn handle_send_file_with_merkle_proof(&self, mut stream: TcpStream) {
        let mut buffer = [0; 8];

        match stream.read_exact(&mut buffer) {
            Ok(_) => {
                let index = u64::from_be_bytes(buffer);
                let mut file =
                    File::open(format!("{}.txt", index)).expect("should open file for reading");
                let mut file_buf = Vec::new();
                file.read_to_end(&mut file_buf).unwrap();

                let mp = MerkleProof::build(&self.merkle_tree, index as usize, file_buf.clone());
                let json_proof = mp.to_string();
                stream
                    .write_all(json_proof.as_bytes())
                    .expect("sending file content to client should not fail");
            }
            Err(e) => {
                error!("Error reading file index: {e}");
                panic!("{}", e); // broken pipe or connection timeout.. should not happen
            }
        }
    }

    pub fn start(&mut self) {
        let listener = TcpListener::bind(SERVER_ADDRESS).unwrap();
        info!("Server listening at: {}", SERVER_ADDRESS);

        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    match self.state.clone() {
                        ServerState::Receive => self.handle_receive_and_store_files(stream),
                        ServerState::Send => self.handle_send_file_with_merkle_proof(stream),
                    };
                }
                Err(e) => {
                    error!("Failed to accept connection: {}", e);
                }
            }
        }
    }
}
