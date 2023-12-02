use log::error;
use serde::{Deserialize, Serialize};
use sha256::digest;
use std::cell::RefCell;
use std::collections::{HashMap, VecDeque};
use std::rc::Rc;

#[derive(Debug)]
pub struct MerkleNode {
    level: usize,
    index: usize,
    value: String,
}

impl MerkleNode {
    pub fn new(level: usize, index: usize, value: String) -> Self {
        Self {
            level,
            index,
            value,
        }
    }
}

pub struct MerkleTree {
    height: usize,
    data: Vec<Rc<RefCell<MerkleNode>>>,
    root: Rc<RefCell<MerkleNode>>,
    // TODO(production): can store pointer data in the node themselves, a hashmap is suboptimal
    store: HashMap<(usize, usize), Rc<RefCell<MerkleNode>>>,
}

impl MerkleTree {
    pub fn new() -> Self {
        Self {
            height: 0,
            data: Vec::new(),
            root: Rc::new(RefCell::new(MerkleNode::new(0, 0, String::new()))),
            store: Default::default(),
        }
    }

    /// compute_root builds the merkle tree level by level using a queue
    fn compute_root(&mut self) {
        if self.data.len() == 1 {
            self.root = Rc::clone(&self.data[0]);
            return;
        }

        let mut queue: VecDeque<Rc<RefCell<MerkleNode>>> = VecDeque::new();
        let mut next_level_queue: VecDeque<Rc<RefCell<MerkleNode>>> = VecDeque::new();

        self.data
            .iter()
            .for_each(|node| queue.push_back(Rc::clone(node)));

        while !queue.is_empty() {
            let left = queue.pop_front().unwrap();
            let left_ref = left.borrow();
            let parent_index = left_ref.index / 2;
            let parent_level = left_ref.level - 1;

            // if there is a right node, compute the hash of the parent node using its value
            // else just compute the hash of the left node's value with itself
            let parent_value = match queue.pop_front() {
                Some(right) => digest(format!("{}{}", left_ref.value, right.borrow().value)),
                None => digest(format!("{}{}", left_ref.value, left_ref.value)),
            };

            let parent = Rc::new(RefCell::new(MerkleNode::new(
                parent_level,
                parent_index,
                parent_value,
            )));
            self.store
                .insert((parent_level, parent_index), Rc::clone(&parent));

            if parent_level == 0 {
                self.root = parent;
                return;
            }

            next_level_queue.push_back(Rc::clone(&parent));

            if queue.is_empty() {
                queue = next_level_queue;
                next_level_queue = VecDeque::new();
            }
        }
    }

    /// get_sibling_from_node_level_and_index gets the sibling node of a node given its id
    fn get_sibling_from_node_level_and_index(
        &self,
        level: usize,
        index: usize,
    ) -> (usize, usize, String) {
        if level <= 0 || level > self.height {
            panic!("Invalid level to get sibling node for");
        }

        if index < 0 || index > self.data.len() {
            panic!("Invalid index to get sibling node for");
        }

        // the sibling index is either the right or left node to the current index
        // if the current index is the left node and also the last node in the nodes list
        // return the current index. This means it is duplicated in the merkle tree because
        // the length of the input data is odd
        let sibling_index = if index % 2 == 0 && index == self.data.len() - 1 {
            index
        } else if index % 2 == 0 {
            index + 1
        } else {
            index - 1
        };

        let node = self
            .store
            .get(&(level, sibling_index))
            .expect("sibling node should be in the merkle root store");

        return (level, sibling_index, node.borrow().value.clone());
    }

    /// get_merkle_path_from_node_index gets all ancestors of a leaf node in a path
    /// given its id. The root is not included since it is part of every valid path
    fn get_merkle_path_from_node_index(&self, mut index: usize) -> Vec<(usize, usize)> {
        if index < 0 || index >= self.data.len() {
            panic!("node index is invalid");
        }
        let mut path = vec![(0, 0); self.height];
        for lvl in (1..self.height + 1).rev() {
            path[lvl - 1] = (lvl, index);
            index /= 2;
        }
        path
    }

    /// get_siblings_of_merkle_path_nodes gets all the siblings of the nodes in the
    /// current node's merkle path, given the node id
    fn get_siblings_of_merkle_path_nodes(&self, index: usize) -> Vec<(usize, usize, String)> {
        self.get_merkle_path_from_node_index(index)
            .into_iter()
            .map(|(lvl, idx)| self.get_sibling_from_node_level_and_index(lvl, idx))
            .collect::<Vec<(usize, usize, String)>>()
    }

    pub fn root_hash(&self) -> String {
        return self.root.borrow().value.clone();
    }
}

impl From<Vec<Vec<u8>>> for MerkleTree {
    fn from(data: Vec<Vec<u8>>) -> Self {
        if data.is_empty() {
            error!("data cannot be empty");
            panic!("data cannot be empty");
        }

        let mut tree = MerkleTree::new();

        // if N is the number of leaf nodes in the tree, then N = 2^H; H = log2(N)
        tree.height = (data.len() as f64).log2().ceil() as usize;
        tree.data = data
            .into_iter()
            .enumerate()
            .map(|(i, d)| {
                let (level, index, value) = (tree.height, i, digest(d));
                let node = Rc::new(RefCell::new(MerkleNode::new(level, index, value)));
                tree.store.insert((tree.height, i), Rc::clone(&node));
                node
            })
            .collect::<Vec<Rc<RefCell<MerkleNode>>>>();
        tree.compute_root();
        tree
    }
}

/// MerkleProof represents the proof for a file index
#[derive(Debug, Serialize, Deserialize)]
pub struct MerkleProof {
    siblings: Vec<(usize, usize, String)>,
    file_buffer: Vec<u8>,
}

impl MerkleProof {
    pub fn build(tree: &MerkleTree, index: usize, file_buffer: Vec<u8>) -> Self {
        let siblings = tree.get_siblings_of_merkle_path_nodes(index);
        Self {
            siblings,
            file_buffer,
        }
    }

    pub fn siblings(&self) -> Vec<(usize, usize, String)> {
        self.siblings.clone()
    }

    pub fn file_buffer(&self) -> Vec<u8> {
        self.file_buffer.clone()
    }
}

impl ToString for MerkleProof {
    fn to_string(&self) -> String {
        serde_json::to_string(self).expect("merkle proof deserialization should not fail")
    }
}

#[cfg(test)]
mod test {
    use sha256::digest;

    fn input_data() -> Vec<Vec<u8>> {
        vec![
            String::from("Hello").into_bytes(),
            String::from("Lorem").into_bytes(),
            String::from("Arbitrary").into_bytes(),
            String::from("Ronaldo").into_bytes(),
            String::from("Rust").into_bytes(),
            String::from("Keyboard").into_bytes(),
            String::from("Golang").into_bytes(),
        ]
    }

    fn build_merkle_vector(data: &Vec<Vec<u8>>) -> Vec<Vec<String>> {
        let height = (data.len() as f64).log2().ceil() as usize;

        let mut curr_vector = data.iter().map(|d| digest(d)).collect::<Vec<String>>();
        let mut vector = Vec::new();
        vector.push(curr_vector.clone());
        while vector.len() != height + 1 {
            let mut next_vector = Vec::new();

            let mut i = 0usize;
            while i < curr_vector.len() {
                // if first node is at last position, duplicate the value
                let data = if i == curr_vector.len() - 1 {
                    digest(format!("{}{}", curr_vector[i], curr_vector[i]))
                } else {
                    digest(format!("{}{}", curr_vector[i], curr_vector[i + 1]))
                };

                next_vector.push(data);
                i += 2
            }

            vector.push(next_vector.clone());
            curr_vector = next_vector;
        }

        vector.reverse();

        vector
    }

    #[test]
    fn compute_root_works() {
        let data = input_data();
        let vector = build_merkle_vector(&data);
        let merkle_tree = super::MerkleTree::from(data);

        for ((lvl, idx), node_from_tree) in merkle_tree.store.iter() {
            let data_from_vector = vector[*lvl][*idx].clone();
            let data_from_tree = node_from_tree.borrow().value.clone();
            assert_eq!(data_from_tree, data_from_vector);
        }
    }

    #[test]
    fn get_sibling_hash_from_node_level_and_index_works() {
        let data = input_data();
        let vector = build_merkle_vector(&data);
        let merkle_tree = super::MerkleTree::from(data);

        let x = merkle_tree.get_sibling_from_node_level_and_index(1, 1);
        assert_eq!(x, (1, 0, vector[1][0].clone()));
        let x = merkle_tree.get_sibling_from_node_level_and_index(2, 0);
        assert_eq!(x, (2, 1, vector[2][1].clone()));
        let x = merkle_tree.get_sibling_from_node_level_and_index(2, 2);
        assert_eq!(x, (2, 3, vector[2][3].clone()));
        let x = merkle_tree.get_sibling_from_node_level_and_index(3, 6);
        assert_eq!(x, (3, 6, vector[3][6].clone()));
    }

    #[test]
    fn get_merkle_path_from_node_index_works() {
        let data = input_data();
        let vector = build_merkle_vector(&data);
        let merkle_tree = super::MerkleTree::from(data);

        let path = merkle_tree.get_merkle_path_from_node_index(0);
        assert_eq!(path, vec![(1, 0), (2, 0), (3, 0)]);
        let path = merkle_tree.get_merkle_path_from_node_index(1);
        assert_eq!(path, vec![(1, 0), (2, 0), (3, 1)]);
        let path = merkle_tree.get_merkle_path_from_node_index(5);
        assert_eq!(path, vec![(1, 1), (2, 2), (3, 5)]);
        let path = merkle_tree.get_merkle_path_from_node_index(6);
        assert_eq!(path, vec![(1, 1), (2, 3), (3, 6)]);
    }

    #[test]
    fn get_sibling_hashes_of_merkle_path_nodes_works() {
        let data = input_data();
        let vector = build_merkle_vector(&data);
        let merkle_tree = super::MerkleTree::from(data);

        let sibling_hashes = merkle_tree.get_siblings_of_merkle_path_nodes(0);
        assert_eq!(
            sibling_hashes,
            vec![
                (
                    1,
                    1,
                    String::from(
                        "a9e0c7220c010bec1b51938763dfb993c46732536e6bffc0a0c5534ac9e1417e"
                    )
                ),
                (
                    2,
                    1,
                    String::from(
                        "48131aa1d56237692add996ad586682c6942d92d81b4cfd5d89ec65d6d334f99"
                    )
                ),
                (
                    3,
                    1,
                    String::from(
                        "1b7f8466f087c27f24e1c90017b829cd8208969018a0bbe7d9c452fa224bc6cc"
                    )
                ),
            ]
        );
        let sibling_hashes = merkle_tree.get_siblings_of_merkle_path_nodes(6);
        assert_eq!(
            sibling_hashes,
            vec![
                (
                    1,
                    0,
                    String::from(
                        "ae62c1ce3d5f7158dc865e55d1d80fd3efc3070501042d87a77b400118ef2ff2"
                    )
                ),
                (
                    2,
                    2,
                    String::from(
                        "433548486c0b2157d6ac754c4526d586275468c2269a681fe369eaba4dc51bf3"
                    )
                ),
                (
                    3,
                    6,
                    String::from(
                        "50e56e797c4ac89f7994a37480fce29a8a0f0f123a695e2dc32d5632197e2318"
                    )
                ),
            ]
        );
    }
}
