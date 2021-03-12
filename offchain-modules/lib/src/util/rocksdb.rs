use force_eth_types::hasher::Blake2bHasher;
use rocksdb::ops::{Delete, Get, Open, WriteOps};
use rocksdb::{ReadOnlyDB, WriteBatch, DB};
// use serde::export::{Clone, Into};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use sparse_merkle_tree::error::Error;
use sparse_merkle_tree::traits::{Store, Value};
use sparse_merkle_tree::tree::{BranchNode, LeafNode};
use sparse_merkle_tree::H256;
use std::path::Path;
use std::sync::Arc;

pub const BRANCH_PREFIX: &[u8] = b"branch";
pub const LEAF_PREFIX: &[u8] = b"leaf";

pub type SMT =
    sparse_merkle_tree::SparseMerkleTree<Blake2bHasher, RocksDBValue, RocksDBStore<RocksDBValue>>;

type Map<K, V> = std::collections::HashMap<K, V>;

// write process only use db, read process only use read_only_db
#[derive(Clone)]
pub struct RocksDBStore<V> {
    pub db: Option<Arc<DB>>,
    pub read_only_db: Option<Arc<ReadOnlyDB>>,
    pub branch_map: Map<H256, BranchNode>,
    pub leaves_map: Map<H256, LeafNode<V>>,
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

impl<V: Clone + Serialize> RocksDBStore<V> {
    pub fn open_readonly(path: String) -> Self {
        let db_dir = shellexpand::tilde(path.as_str()).into_owned();
        let db_path = Path::new(db_dir.as_str());

        if !db_path.exists() {
            panic!("rocksdb path should exist when opening db");
        }
        let read_only_db = ReadOnlyDB::open_default(db_path).expect("open rocksdb");
        let read_only_db = Arc::new(read_only_db);

        RocksDBStore {
            db: None,
            read_only_db: Some(read_only_db),
            branch_map: Map::default(),
            leaves_map: Map::default(),
        }
    }
    pub fn open(path: String) -> Self {
        let db_dir = shellexpand::tilde(path.as_str()).into_owned();
        let db_path = Path::new(db_dir.as_str());

        if !db_path.exists() {
            panic!("rocksdb path should exist when opening db");
        }
        let db = DB::open_default(db_path).expect("open rocksdb");
        let db = Arc::new(db);

        RocksDBStore {
            db: Some(db),
            read_only_db: None,
            branch_map: Map::default(),
            leaves_map: Map::default(),
        }
    }
    pub fn new(path: String) -> Self {
        let db_dir = shellexpand::tilde(path.as_str()).into_owned();
        let db_path = Path::new(db_dir.as_str());

        if !db_path.exists() {
            std::fs::create_dir_all(db_path).expect("create db path dir");
        } else {
            panic!("rocksdb path should not exist when creating db");
        }
        let db = DB::open_default(db_path).expect("open rocksdb");
        let db = Arc::new(db);

        RocksDBStore {
            db: Some(db),
            read_only_db: None,
            branch_map: Map::default(),
            leaves_map: Map::default(),
        }
    }

    pub fn commit(&mut self) {
        let mut batch = WriteBatch::default();
        for (key, branch) in &self.branch_map {
            let db_branch_node = DBBranchNode {
                fork_height: branch.fork_height,
                key: branch.key.into(),
                node: branch.node.into(),
                sibling: branch.sibling.into(),
            };
            let db_branch_node_raw = serde_json::to_vec(&db_branch_node).unwrap();
            batch
                .put(get_db_key_for_branch(key.as_slice()), db_branch_node_raw)
                .expect("put branch");
        }
        for (key, leaf) in &self.leaves_map {
            let db_leaf_node = DBLeafNode {
                key: leaf.key.into(),
                value: leaf.value.clone(),
            };
            let db_leaf_node_raw = serde_json::to_vec(&db_leaf_node).unwrap();
            batch
                .put(get_db_key_for_leaf(key.as_slice()), db_leaf_node_raw)
                .expect("put leaf");
        }
        let db = self.db.as_ref().expect("write mode should use db");
        db.write(&batch).expect("write batch rocksdb");
    }
}

impl<V: Clone + Serialize + DeserializeOwned> Store<V> for RocksDBStore<V> {
    // search key from cache first, if key not exists in cache, then search it from rocksdb.
    fn get_branch(&self, node: &H256) -> Result<Option<BranchNode>, Error> {
        let cache_value = self.branch_map.get(node).map(Clone::clone);
        if cache_value.is_some() {
            return Ok(cache_value);
        }

        let db_value = match self.db.is_some() {
            true => self
                .db
                .as_ref()
                .unwrap()
                .get(get_db_key_for_branch(node.as_slice()))
                .unwrap(),
            false => self
                .read_only_db
                .as_ref()
                .expect("should be read only db when db is none")
                .get(get_db_key_for_branch(node.as_slice()))
                .unwrap(),
        };

        match db_value {
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
        let cache_value = self.leaves_map.get(leaf_hash).map(Clone::clone);
        if cache_value.is_some() {
            return Ok(cache_value);
        }

        let db_value = match self.db.is_some() {
            true => self
                .db
                .as_ref()
                .unwrap()
                .get(get_db_key_for_leaf(leaf_hash.as_slice()))
                .unwrap(),
            false => self
                .read_only_db
                .as_ref()
                .expect("should be read only db when db is none")
                .get(get_db_key_for_leaf(leaf_hash.as_slice()))
                .unwrap(),
        };

        match db_value {
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
        self.branch_map.insert(node, branch);
        Ok(())
    }
    fn insert_leaf(&mut self, leaf_hash: H256, leaf: LeafNode<V>) -> Result<(), Error> {
        self.leaves_map.insert(leaf_hash, leaf);
        Ok(())
    }
    fn remove_branch(&mut self, node: &H256) -> Result<(), Error> {
        self.branch_map.remove(node);
        self.db
            .as_ref()
            .expect("only db can delete")
            .delete(get_db_key_for_branch(node.as_slice()))
            .unwrap();
        Ok(())
    }
    fn remove_leaf(&mut self, leaf_hash: &H256) -> Result<(), Error> {
        self.leaves_map.remove(leaf_hash);
        self.db
            .as_ref()
            .expect("only db can delete")
            .delete(get_db_key_for_leaf(leaf_hash.as_slice()))
            .unwrap();
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

fn get_db_key_for_branch(key: &[u8]) -> Vec<u8> {
    let mut db_key = vec![];
    db_key.extend_from_slice(BRANCH_PREFIX.as_ref());
    db_key.extend_from_slice(key);
    db_key
}

fn get_db_key_for_leaf(key: &[u8]) -> Vec<u8> {
    let mut db_key = vec![];
    db_key.extend_from_slice(LEAF_PREFIX.as_ref());
    db_key.extend_from_slice(key);
    db_key
}

pub fn open_rocksdb(path: String) -> Arc<DB> {
    let db_dir = shellexpand::tilde(path.as_str()).into_owned();
    let db_path = Path::new(db_dir.as_str());

    if !db_path.exists() {
        std::fs::create_dir_all(db_path).expect("create db path dir");
    }
    let db = DB::open_default(db_path).expect("open rocksdb");
    Arc::new(db)
}

pub fn open_readonly_rocksdb(path: String) -> Arc<ReadOnlyDB> {
    let db_dir = shellexpand::tilde(path.as_str()).into_owned();
    let db_path = Path::new(db_dir.as_str());

    if !db_path.exists() {
        std::fs::create_dir_all(db_path).expect("create db path dir");
    }
    let db = ReadOnlyDB::open_default(db_path).expect("open rocksdb");
    Arc::new(db)
}

// #[test]
// fn test_rocksdb() {
//     let mut db = RocksDBStore::new("~/.force-bridge/test-rocksdb".to_string());
//     for i in 1..100 {
//         let branch = BranchNode {
//             fork_height: i,
//             key: [i; 32].into(),
//             node: [i; 32].into(),
//             sibling: [i; 32].into(),
//         };
//
//         let leaf_value: RocksDBValue = [i; 32].into();
//         let leaf = LeafNode {
//             key: [i; 32].into(),
//             value: leaf_value,
//         };
//         db.insert_branch([i; 32].into(), branch).unwrap();
//         db.insert_leaf([i; 32].into(), leaf).unwrap();
//     }
//     db.commit();
//
//     db.remove_leaf(&[1u8; 32].into()).unwrap();
//     db.remove_leaf(&[200u8; 32].into()).unwrap();
//
//     let v1 = db.get_leaf(&[1u8; 32].into()).unwrap();
//     let v2 = db.get_leaf(&[200u8; 32].into()).unwrap();
//     let v3 = db.get_leaf(&[2u8; 32].into()).unwrap();
//     assert!(v1.is_none());
//     assert!(v2.is_none());
//     assert!(v3.is_some());
// }
