use common::model::file_info::FileInfo;
use common::model::merkle::{MerkleProof, MerkleTree};
use common::SERVER_ADDRESS;
use log::{error, info};
use serde::{Deserialize, Serialize};
use sha256::digest;
use std::fs::File;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::str::FromStr;

const FILES_DATA_NAME: &str = "merkle.json";

#[derive(Serialize, Deserialize)]
struct DiskData {
    merkle_root: String,
    files_count: usize,
}

impl DiskData {
    fn build(merkle_root: String, files_count: usize) -> Self {
        Self {
            merkle_root,
            files_count,
        }
    }
}

impl FromStr for DiskData {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        serde_json::from_str(s)
            .map_err(|_| String::from("disk data deserialization should not fail"))
    }
}

impl ToString for DiskData {
    fn to_string(&self) -> String {
        serde_json::to_string(self).expect("disk data deserialization should not fail")
    }
}

pub struct Client {
    files: Vec<FileInfo>,
    file_names: Vec<String>,
    files_data: Vec<Vec<u8>>,
    files_count: usize,
    merkle_root: String,
    downloaded_file: Vec<u8>,
}

impl Client {
    pub fn new() -> Self {
        Self {
            files: Vec::new(),
            file_names: Vec::new(),
            files_data: Vec::new(),
            files_count: 0,
            merkle_root: String::new(),
            downloaded_file: Vec::new(),
        }
    }
}

/// this implementation has methods concerned with sending files to the server
impl Client {
    /// load_files_into_memory loads the contents of the specified files into memory
    pub fn load_files_into_memory(&mut self, file_names: Vec<String>) {
        file_names
            .into_iter()
            .enumerate()
            .for_each(|(index, file_name)| {
                let mut file = File::open(&file_name).expect("file should be present");
                let mut file_buf = Vec::new();
                file.read_to_end(&mut file_buf).unwrap();
                self.files.push(FileInfo::new(index, file_name, file_buf));
            });

        self.files_count = self.files.len();
    }

    /// build_merkle_tree_and_save_to_disk builds a merkle tree from the files
    /// and saves the merkle root and the number of files to disk
    pub fn build_merkle_tree_and_save_to_disk(&mut self) {
        let bufs = self
            .files
            .iter()
            .map(|file| file.content())
            .collect::<Vec<Vec<u8>>>();
        let merkle_tree = MerkleTree::from(bufs);
        self.merkle_root = merkle_tree.root_hash();
        let disk_json = DiskData::build(merkle_tree.root_hash(), self.files.len()).to_string();
        let mut file = File::create(FILES_DATA_NAME).expect("json file creation should not fail");
        file.write_all(disk_json.as_bytes())
            .expect("writing data to the stream should not fail");
    }

    /// send_files_and_clear_file_data sends the files stored in memory to the server
    /// over a TCP connection and consumes up the files in memory
    pub fn send_files_and_clear_file_data(&mut self) {
        let mut stream =
            TcpStream::connect(SERVER_ADDRESS).expect("should connect to tcp server stream");

        let files_json = serde_json::to_string(&self.files)
            .expect("serializing the list of files should not fail");
        stream
            .write_all(files_json.as_bytes())
            .expect("sending files over tcp stream should not fail");

        self.files.iter().for_each(|file| {
            std::fs::remove_file(file.name())
                .expect("removing file from the directory should not fail")
        });

        info!("Files sent successfully");
    }

    /// prepare_and_send_files validates the files, computes the merkle root,
    /// sends the files to the server and deletes the files from the client
    pub fn prepare_and_send_files(&mut self, file_names: Vec<String>) {
        self.load_files_into_memory(file_names);
        self.build_merkle_tree_and_save_to_disk();
        self.send_files_and_clear_file_data();
    }
}

/// this implementation has methods concerned with receiving and verifying files from the server
impl Client {
    /// compute_merkle_root_from_proof computes the root of the merkle tree given the siblings
    /// by walking up the merkle tree until the root.
    /// Each node can either be a left or right node, compute the node hash with that information
    fn compute_merkle_root_from_proof(&self, proof: &MerkleProof, index: usize) -> String {
        let mut curr_hash = digest(proof.file_content());
        let mut siblings = proof.siblings();
        let mut curr_index = index;

        siblings.sort_by(|(lvl1, _, _), (lvl2, _, _)| lvl2.cmp(&lvl1));

        for (_, _, sibling_hash) in siblings {
            curr_hash = if curr_index % 2 == 0 {
                digest(format!("{}{}", curr_hash, sibling_hash))
            } else {
                digest(format!("{}{}", sibling_hash, curr_hash))
            };
            curr_index /= 2;
        }

        curr_hash
    }

    /// fetch_merkle_proof fetches the Merkle proof for a given file index from the server,
    fn fetch_merkle_proof(&mut self, index: usize) -> Result<MerkleProof, String> {
        let mut stream =
            TcpStream::connect(SERVER_ADDRESS).expect("client should connect to the server stream");
        stream
            .write_all(&index.to_be_bytes())
            .expect("file index should be sent to the server");

        let mut json_proof_buf = Vec::new();
        if let Err(e) = stream.read_to_end(&mut json_proof_buf) {
            error!("Error reading merkle proof from server: {}", e);
            return Err(format!("Error reading merkle proof from server: {}", e));
        }

        let proof = serde_json::from_slice(&json_proof_buf)
            .expect("merkle proof deserialization should not fail");

        Ok(proof)
    }

    /// validate_file_index_and_update_root validates the requested file index and updates
    /// the files_count and merkle_root fields if valid. Returns an error if the index is out of range.
    fn validate_file_index_and_update_root(&mut self, index: usize) -> Result<(), String> {
        let mut file = File::open(FILES_DATA_NAME)
            .expect("json file holding merkle root should not fail to open");

        let mut json_str = String::new();
        file.read_to_string(&mut json_str)
            .expect("merkle root file conversion to string should not fail");

        let data = DiskData::from_str(&json_str)?;
        if index >= data.files_count {
            return Err(String::from("file index to download is not available"));
        }
        self.files_count = data.files_count;
        self.merkle_root = data.merkle_root;

        Ok(())
    }

    /// download_file sends a download request to the server with the index, gets the
    /// file and computes and compares the merkle root using the proof from the server
    pub fn download_verify_and_write_file(&mut self, index: usize) -> Result<(), String> {
        self.validate_file_index_and_update_root(index)?;
        let mp = self.fetch_merkle_proof(index)?;
        let generated_root = self.compute_merkle_root_from_proof(&mp, index);
        assert_eq!(
            self.merkle_root, generated_root,
            "The file downloaded at index: {} is corrupt. \
        Expected merkle root: {}, Actual merkle root: {}",
            index, self.merkle_root, generated_root
        );

        let download_buf = mp.file_content();
        let mut download =
            File::create(mp.file_name()).expect("downloaded file creation should not fail");
        download.write_all(&download_buf).unwrap();

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::client::Client;
    use common::model::file_info::FileInfo;
    use common::model::merkle::MerkleProof;
    use sha256::digest;
    use std::fs::File;
    use std::io::Read;

    fn parse_files() -> (Vec<String>, Vec<FileInfo>) {
        let file_names: Vec<String> = vec![
            String::from("../files/cv.txt"),
            String::from("../files/food.json"),
        ];
        let mut files = Vec::new();
        for (i, f) in file_names.iter().enumerate() {
            let mut file = File::open(&f).expect("file should be present");
            let mut file_buf = Vec::new();
            file.read_to_end(&mut file_buf).unwrap();
            let file_info = FileInfo::new(i, f.clone(), file_buf);
            files.push(file_info);
        }
        (file_names, files)
    }

    fn get_merkle_root() -> String {
        let (_, expected_files) = parse_files();
        let first = digest(expected_files[0].content());
        let second = digest(expected_files[1].content());
        digest(format!("{}{}", first, second))
    }

    #[test]
    fn load_files_into_memory_works() {
        let (file_names, expected_files) = parse_files();
        let mut client = Client::new();
        client.load_files_into_memory(file_names);
        assert_eq!(client.files_count, 2);
        for (expected_file, actual_file) in client.files.iter().zip(expected_files.iter()) {
            assert_eq!(actual_file, expected_file)
        }
    }

    #[test]
    fn build_merkle_tree_and_save_to_disk_works() {
        let (file_names, _) = parse_files();
        let root_hash = get_merkle_root();
        let mut client = Client::new();
        client.load_files_into_memory(file_names);
        client.build_merkle_tree_and_save_to_disk();
        assert_eq!(client.merkle_root, root_hash);
    }

    #[test]
    fn mock_server_has_correct_files() {
        let (file_names, expected_files) = parse_files();
        let mp = MerkleProof::new(
            file_names[0].clone(),
            expected_files[0].content(),
            vec![(1, 1, digest(expected_files[1].content()))],
        );
        let mut client = Client::new();
        let hashed = client.compute_merkle_root_from_proof(&mp, 0);
        assert_eq!(hashed, get_merkle_root())
    }

    #[test]
    fn mock_server_does_not_have_correct_files() {
        let (file_names, mut expected_files) = parse_files();

        let mut altered_content = expected_files[0].content();
        altered_content[0] = 32u8;

        let mp = MerkleProof::new(
            file_names[0].clone(),
            altered_content.clone(),
            vec![(1, 1, digest(expected_files[1].content()))],
        );
        let mut client = Client::new();
        let hashed = client.compute_merkle_root_from_proof(&mp, 0);
        assert_ne!(hashed, get_merkle_root());
        assert_eq!(
            hashed,
            digest(format!(
                "{}{}",
                digest(altered_content),
                digest(expected_files[1].content())
            ))
        );
    }
}
