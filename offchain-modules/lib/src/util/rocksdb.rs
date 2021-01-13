use core::marker::PhantomData;
use force_eth_types::hasher::Blake2bHasher;
use rocksdb::ops::{Delete, Get, Open, Put};
use rocksdb::DB;
use serde::export::{Clone, Into};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use sparse_merkle_tree::error::Error;
use sparse_merkle_tree::traits::{Store, Value};
use sparse_merkle_tree::tree::{BranchNode, LeafNode};
use sparse_merkle_tree::H256;
use std::path::{Path, PathBuf};
use std::sync::Arc;

pub type SMT =
    sparse_merkle_tree::SparseMerkleTree<Blake2bHasher, RocksDBValue, RocksDBStore<RocksDBValue>>;

#[derive(Clone)]
pub struct RocksDBStore<V> {
    pub branch_db: Arc<DB>,
    pub leaves_db: Arc<DB>,
    pub phantom: PhantomData<V>,
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Clone)]
pub struct DBBranchNode {
    pub fork_height: u8,
    pub key: [u8; 32],
    pub node: [u8; 32],
    pub sibling: [u8; 32],
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Clone)]
pub struct DBLeafNode<V> {
    pub key: [u8; 32],
    pub value: V,
}

impl<V> RocksDBStore<V> {
    pub fn open(path: String) -> Self {
        let db_dir = shellexpand::tilde(path.as_str()).into_owned();
        let branch_db_path = Path::new(db_dir.as_str()).join("branch");
        let leaves_db_path = Path::new(db_dir.as_str()).join("leaves");

        fn create_db(db_path: &PathBuf) -> Arc<DB> {
            if !db_path.exists() {
                panic!("rocksdb path should exist when opening db");
            }
            let db = DB::open_default(db_path).expect("open rocksdb");
            Arc::new(db)
        }
        let branch_db = create_db(&branch_db_path);
        let leaves_db = create_db(&leaves_db_path);

        RocksDBStore {
            branch_db,
            leaves_db,
            phantom: PhantomData::default(),
        }
    }
    pub fn new(path: String) -> Self {
        let db_dir = shellexpand::tilde(path.as_str()).into_owned();
        let branch_db_path = Path::new(db_dir.as_str()).join("branch");
        let leaves_db_path = Path::new(db_dir.as_str()).join("leaves");

        fn create_db(db_path: &PathBuf) -> Arc<DB> {
            if !db_path.exists() {
                std::fs::create_dir_all(&db_path).expect("create db path dir");
            } else {
                panic!("rocksdb path should not exist when creating db");
            }
            let db = DB::open_default(db_path).expect("open rocksdb");
            Arc::new(db)
        }
        let branch_db = create_db(&branch_db_path);
        let leaves_db = create_db(&leaves_db_path);

        RocksDBStore {
            branch_db,
            leaves_db,
            phantom: PhantomData::default(),
        }
    }
}

impl<V: Clone + Serialize + DeserializeOwned> Store<V> for RocksDBStore<V> {
    fn get_branch(&self, node: &H256) -> Result<Option<BranchNode>, Error> {
        let value = self.branch_db.get(node.as_slice()).unwrap();
        match value {
            Some(v) => {
                let n: DBBranchNode = serde_json::from_slice(v.as_ref()).unwrap();
                let branch_node = BranchNode {
                    fork_height: n.fork_height,
                    key: n.key.into(),
                    node: n.node.into(),
                    sibling: n.sibling.into(),
                };
                Ok(Some(branch_node))
            }
            None => Ok(None),
        }
    }
    fn get_leaf(&self, leaf_hash: &H256) -> Result<Option<LeafNode<V>>, Error> {
        let value = self.leaves_db.get(leaf_hash.as_slice()).unwrap();
        match value {
            Some(v) => {
                let n: DBLeafNode<V> = serde_json::from_slice(v.as_ref()).unwrap();
                let node = LeafNode {
                    key: n.key.clone().into(),
                    value: n.value,
                };
                Ok(Some(node))
            }
            None => Ok(None),
        }
    }
    fn insert_branch(&mut self, node: H256, branch: BranchNode) -> Result<(), Error> {
        let db_branch_node = DBBranchNode {
            fork_height: branch.fork_height,
            key: branch.key.into(),
            node: branch.node.into(),
            sibling: branch.sibling.into(),
        };
        let db_branch_node_raw = serde_json::to_vec(&db_branch_node).unwrap();
        self.branch_db
            .put(node.as_slice(), db_branch_node_raw)
            .unwrap();
        Ok(())
    }
    fn insert_leaf(&mut self, leaf_hash: H256, leaf: LeafNode<V>) -> Result<(), Error> {
        let db_leaf_node = DBLeafNode {
            key: leaf.key.into(),
            value: leaf.value,
        };
        let db_leaf_node_raw = serde_json::to_vec(&db_leaf_node).unwrap();
        self.leaves_db
            .put(leaf_hash.as_slice(), db_leaf_node_raw)
            .unwrap();
        Ok(())
    }
    fn remove_branch(&mut self, node: &H256) -> Result<(), Error> {
        self.branch_db.delete(node.as_slice()).unwrap();
        Ok(())
    }
    fn remove_leaf(&mut self, leaf_hash: &H256) -> Result<(), Error> {
        self.leaves_db.delete(leaf_hash.as_slice()).unwrap();
        Ok(())
    }
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Default, Hash, Clone, Copy)]
pub struct RocksDBValue([u8; 32]);

impl Value for RocksDBValue {
    fn to_h256(&self) -> H256 {
        self.0.into()
    }
    fn zero() -> Self {
        RocksDBValue([0u8; 32])
    }
}

impl From<[u8; 32]> for RocksDBValue {
    fn from(v: [u8; 32]) -> RocksDBValue {
        RocksDBValue(v)
    }
}

impl Into<[u8; 32]> for RocksDBValue {
    fn into(self: RocksDBValue) -> [u8; 32] {
        self.0
    }
}
