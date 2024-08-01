#[derive(Debug)]
pub enum KvStoreError {
    Open(rocksdb::Error),
    Serialize {
        type_name: &'static str,
        data: String,
        error: bincode::Error,
    },
    Deserialize {
        type_name: &'static str,
        error: bincode::Error,
    },
    Get(rocksdb::Error),
    GetMut(rocksdb::Error),
    Put(rocksdb::Error),
    CommitPut(rocksdb::Error),
    Delete(rocksdb::Error),
    CommitDelete(rocksdb::Error),
    Update(rocksdb::Error),
    CommitUpdate(rocksdb::Error),
    NoneType,
    Initialize,
}

impl std::fmt::Display for KvStoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for KvStoreError {}

impl KvStoreError {
    pub fn is_none_type(&self) -> bool {
        match self {
            Self::NoneType => true,
            _others => false,
        }
    }
}
