use core::marker::PhantomData;
use rocksdb::ops::{Delete, Get, Open, Put};
use rocksdb::DB;
use serde::export::{Clone, Into};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use sparse_merkle_tree::error::Error;
use sparse_merkle_tree::traits::{Store, Value};
use sparse_merkle_tree::tree::{BranchNode, LeafNode};
use sparse_merkle_tree::H256;
use std::path::Path;
use std::sync::Arc;

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

impl<V: Clone + Serialize + DeserializeOwned> Store<V> for RocksDBStore<V> {
    fn get_branch(&self, node: &H256) -> Result<Option<BranchNode>, Error> {
        let value = self.branch_db.get(node.as_slice()).unwrap();
        if value.is_some() {
            let n: DBBranchNode = serde_json::from_slice(value.unwrap().as_ref()).unwrap();
            let branch_node = BranchNode {
                fork_height: n.fork_height,
                key: n.key.into(),
                node: n.node.into(),
                sibling: n.sibling.into(),
            };
            Ok(Some(branch_node))
        } else {
            Ok(None)
        }
    }
    fn get_leaf(&self, leaf_hash: &H256) -> Result<Option<LeafNode<V>>, Error> {
        let value = self.leaves_db.get(leaf_hash.as_slice()).unwrap();
        if value.is_some() {
            let n: DBLeafNode<V> = serde_json::from_slice(value.unwrap().as_ref()).unwrap();
            let node = LeafNode {
                key: n.key.clone().into(),
                value: n.value.clone(),
            };
            Ok(Some(node))
        } else {
            Ok(None)
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

pub fn open_db(path: &str) -> Arc<DB> {
    let path = Path::new(path);
    std::fs::create_dir_all(path.clone()).expect("create dir");
    let db = DB::open_default(path).expect("open rocksdb");
    Arc::new(db)
}
