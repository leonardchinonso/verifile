use std::io::{Read, Write};
use std::net::{TcpStream};
use std::fs::File;
use log::{error, info};
use common::SERVER_ADDRESS;
use serde::{Serialize, Deserialize};

const FILES_DATA_NAME: &str = "files/file_data.json";

#[derive(Serialize, Deserialize)]
struct FilesData {
    merkle_root: String,
    files_count: usize,
}

pub struct Client {
    file_names: Vec<String>,
    files_count: usize,
    merkle_root: String,
    curr_file_data: Vec<u8>
}

impl Client {
    pub fn new() -> Self {
        Self{
            file_names: Vec::new(),
            files_count: 0,
            merkle_root: String::new(),
            curr_file_data: Vec::new()
        }
    }

    pub fn add_files(&mut self, file_names: Vec<String>) {
        self.files_count = file_names.len();
        self.file_names = file_names;
    }

    pub fn process_files(&self) {
        let data = FilesData {
            merkle_root: "e4fa1555ad877bf0ec455483371867200eee89550a93eff2f95a6198".to_string(),
            files_count: self.files_count,
        };

        let json = serde_json::to_string(&data).expect("Failed to serialize data");
        let mut file = File::create(FILES_DATA_NAME).expect("Failed to create file");
        file.write_all(json.as_bytes()).expect("Failed to write data");
    }

    pub fn send_files(&mut self) {
        // open a stream to send multiple files
        let mut stream = TcpStream::connect(SERVER_ADDRESS).expect("Failed to connect to server.");

        for i in 0..self.file_names.len() {
            // open the file
            let mut file = File::open(&self.file_names[i]).unwrap();

            // create a buffer to read the file into
            let mut file_buf = Vec::new();
            file.read_to_end(&mut file_buf).unwrap();

            // Send the size of the file
            let size = file_buf.len() as u64;
            stream.write_all(&size.to_be_bytes()).expect("Failed to send file size to server");

            // write the file contents to the server over the stream
            stream.write_all(&file_buf).expect("Failed to send file content to server");

            // clear the buffer for the next file
            file_buf.clear();
        }

        // Send a file size of 0 to indicate that there are no more files
        let size = 0u64;
        stream.write_all(&size.to_be_bytes()).expect("Failed to send file size of 0 to server");

        // delete the files from storage
        for file_name in self.file_names.iter() {
            std::fs::remove_file(file_name).expect("Failed to remove file");
        }

        // delete all files from the client object
        self.file_names.clear();

        info!("Files sent successfully");
    }

    fn verify_download_request(&mut self, index: usize) -> Result<(), String> {
        let mut file = File::open(FILES_DATA_NAME).expect("Failed to open file");

        let mut json = String::new();
        file.read_to_string(&mut json).expect("Failed to read file");

        let data: FilesData = serde_json::from_str(&json).expect("Failed to deserialize data");

        if index >= data.files_count {
            return Err(String::from("file index to download is not available"));
        }

        self.files_count = data.files_count;
        self.merkle_root = data.merkle_root;

        Ok(())
    }

    fn get_file_from_server(&mut self, index: usize) -> Result<(), String> {
        // open a stream to send multiple files
        let mut stream = TcpStream::connect(SERVER_ADDRESS).expect("Failed to connect to server.");

        // send the file index to download
        stream.write_all(&index.to_be_bytes()).expect("Failed to send file index to server");

        // Create a new file
        let mut file = File::create(format!("files/{}.txt", index)).expect("Cannot create file to download to");

        // Buffer to hold file data
        let mut file_buf = Vec::new();

        // Read the file data
        if let Err(e) = stream.read_to_end(&mut file_buf) {
            error!("Error reading downloaded content: {}", e);
            return Err(format!("Error reading downloaded content: {}", e));
        }

        // Write the file data
        file.write_all(&file_buf).unwrap();

        // save the contents as the current file
        self.curr_file_data = file_buf;

        Ok(())
    }

    pub fn download_file(&mut self, index: usize) -> Result<(), String> {
        self.verify_download_request(index)?;

        self.get_file_from_server(index)?;

        Ok(())
    }
}

