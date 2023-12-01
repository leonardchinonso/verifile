use common::model::merkle::{MerkleProof, MerkleTree};
use common::SERVER_ADDRESS;
use log::{error, info};
use serde::{Deserialize, Serialize};
use sha256::digest;
use std::fs::File;
use std::io::{Read, Write};
use std::net::TcpStream;

const FILES_DATA_NAME: &str = "files/file_data.json";

#[derive(Serialize, Deserialize)]
struct DiskData {
    merkle_root: String,
    files_count: usize,
}

impl DiskData {
    fn new(merkle_root: String, files_count: usize) -> Self {
        Self {
            merkle_root,
            files_count,
        }
    }
}

pub struct Client {
    file_names: Vec<String>,
    files_data: Vec<Vec<u8>>,
    files_count: usize,
    merkle_root: String,
}

impl Client {
    pub fn new() -> Self {
        Self {
            file_names: Vec::new(),
            files_data: Vec::new(),
            files_count: 0,
            merkle_root: String::new(),
        }
    }

    pub fn add_files(&mut self, file_names: Vec<String>) {
        self.files_count = file_names.len();
        self.file_names = file_names;

        for file_name in self.file_names.iter() {
            // open the file
            let mut file = File::open(file_name).unwrap();

            // create a buffer to read the file into
            let mut file_buf = Vec::new();
            file.read_to_end(&mut file_buf).unwrap();

            // push the current file as bytes to the client
            self.files_data.push(file_buf);
        }
    }

    /// process_files builds a merkle tree from the files and saves the merkle root to disk
    pub fn process_files(&self) {
        let merkle_tree = MerkleTree::from(self.files_data.clone());

        let disk_data = DiskData::new(merkle_tree.root_hash(), self.files_count);

        let json = serde_json::to_string(&disk_data).expect("Failed to serialize data");
        let mut file = File::create(FILES_DATA_NAME).expect("Failed to create file");
        file.write_all(json.as_bytes())
            .expect("Failed to write data");
    }

    pub fn send_files(&mut self) {
        // open a stream to send multiple files
        let mut stream = TcpStream::connect(SERVER_ADDRESS).expect("Failed to connect to server.");

        for file_buf in self.files_data.iter() {
            // Send the size of the file
            let size = file_buf.len() as u64;
            stream
                .write_all(&size.to_be_bytes())
                .expect("Failed to send file size to server");

            // write the file contents to the server over the stream
            stream
                .write_all(file_buf)
                .expect("Failed to send file content to server");
        }

        // Send a file size of 0 to indicate that there are no more files
        let size = 0u64;
        stream
            .write_all(&size.to_be_bytes())
            .expect("Failed to send file size of 0 to server");

        // delete the files from storage
        for file_name in self.file_names.iter() {
            std::fs::remove_file(file_name).expect("Failed to remove file");
        }

        // delete all files from the client object
        self.file_names.clear();
        self.files_data.clear();

        info!("Files sent successfully");
    }

    fn verify_download_request(&mut self, index: usize) -> Result<(), String> {
        let mut file = File::open(FILES_DATA_NAME).expect("Failed to open file");

        let mut json = String::new();
        file.read_to_string(&mut json).expect("Failed to read file");

        let data: DiskData = serde_json::from_str(&json).expect("Failed to deserialize data");

        if index >= data.files_count {
            return Err(String::from("file index to download is not available"));
        }

        self.files_count = data.files_count;
        self.merkle_root = data.merkle_root;

        Ok(())
    }

    fn get_merkle_proof_from_server(&mut self, index: usize) -> Result<Vec<u8>, String> {
        // open a stream to send multiple files
        let mut stream = TcpStream::connect(SERVER_ADDRESS).expect("Failed to connect to server.");

        // send the file index to download
        stream
            .write_all(&index.to_be_bytes())
            .expect("Failed to send file index to server");

        // Create a new file
        let mut json_proof_file = File::create(format!("files/merkle_proof_{}.json", index))
            .expect("Cannot create file to download to");

        // Buffer to hold file data
        let mut json_proof_buf = Vec::new();

        // Read the file data
        if let Err(e) = stream.read_to_end(&mut json_proof_buf) {
            error!("Error reading downloaded content: {}", e);
            return Err(format!("Error reading downloaded content: {}", e));
        }

        // Write the file data
        json_proof_file.write_all(&json_proof_buf).unwrap();

        // return the contents
        Ok(json_proof_buf)
    }

    /// download_file sends a download request to the server with the index
    /// gets the file and computes the merkle root using the proof from the server
    pub fn download_file(&mut self, index: usize) -> Result<(), String> {
        self.verify_download_request(index)?;

        let json_proof_buf = self.get_merkle_proof_from_server(index)?;
        let mp: MerkleProof =
            serde_json::from_slice(&json_proof_buf).expect("Failed to deserialize proof");

        let generated_root = compute_merkle_root_from_proof(mp, index);

        // TODO(development): Compare the generated root to the merkle root stored on disk

        Ok(())
    }
}

/// compute_merkle_root_from_proof computes the root of the merkle tree given the siblings
/// by walking up the merkle tree until the root
fn compute_merkle_root_from_proof(proof: MerkleProof, index: usize) -> String {
    let mut curr_hash = digest(proof.file_buffer());
    let mut siblings = proof.siblings();
    let mut curr_index = index;

    // sort the siblings by levels, starting from the sibling of the leaf node
    siblings.sort_by(|(lvl1, _, _), (lvl2, _, _)| lvl2.cmp(&lvl1));

    for (_, _, sibling_hash) in siblings {
        curr_hash = if curr_index % 2 == 0 {
            // if current node is a left node, sibling is right
            digest(format!("{}{}", curr_hash, sibling_hash))
        } else {
            // if curr node is a right node, then sibling is left
            digest(format!("{}{}", sibling_hash, curr_hash))
        };

        // set index to parent index
        curr_index /= 2;
    }

    curr_hash
}
