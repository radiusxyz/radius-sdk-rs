use std::{
    fmt::Debug,
    mem::MaybeUninit,
    path::Path,
    sync::{Arc, Once},
};

use rocksdb::{Options, Transaction, TransactionDB, TransactionDBOptions};
use serde::{de::DeserializeOwned, ser::Serialize};

use crate::data_type::{deserialize, serialize};

static mut KVSTORE: MaybeUninit<KvStore> = MaybeUninit::uninit();
static INIT: Once = Once::new();

pub fn kvstore() -> Result<&'static KvStore, KvStoreError> {
    match INIT.is_completed() {
        true => unsafe { Ok(KVSTORE.assume_init_ref()) },
        false => Err(KvStoreError::Initialize),
    }
}

pub struct KvStore {
    database: Arc<TransactionDB>,
}

unsafe impl Send for KvStore {}

unsafe impl Sync for KvStore {}

impl Clone for KvStore {
    fn clone(&self) -> Self {
        Self {
            database: self.database.clone(),
        }
    }
}

impl KvStore {
    pub fn new(path: impl AsRef<Path>) -> Result<Self, KvStoreError> {
        let mut database_options = Options::default();
        database_options.create_if_missing(true);

        let transaction_database_options = TransactionDBOptions::default();
        let transaction_database =
            TransactionDB::open(&database_options, &transaction_database_options, path)
                .map_err(KvStoreError::Open)?;

        Ok(Self {
            database: Arc::new(transaction_database),
        })
    }

    pub fn init(self) {
        unsafe {
            INIT.call_once(|| {
                KVSTORE.write(self);
            });
        }
    }

    pub fn put<K, V>(&self, key: &K, value: &V) -> Result<(), KvStoreError>
    where
        K: Debug + Serialize,
        V: Debug + DeserializeOwned + Serialize,
    {
        let key_vec = serialize(key)?;
        let value_vec = serialize(value)?;

        let transaction = self.database.transaction();

        transaction
            .put(key_vec, value_vec)
            .map_err(KvStoreError::Put)?;
        transaction.commit().map_err(KvStoreError::CommitPut)?;

        Ok(())
    }

    pub fn get<K, V>(&self, key: &K) -> Result<V, KvStoreError>
    where
        K: Debug + Serialize,
        V: Debug + DeserializeOwned + Serialize,
    {
        let key_vec = serialize(key)?;

        let value_slice = self
            .database
            .get_pinned(key_vec)
            .map_err(KvStoreError::Get)?
            .ok_or(KvStoreError::NoneType)?;
        let value: V = deserialize(value_slice)?;

        Ok(value)
    }

    /// Get the value or return `V::default()`.
    pub fn get_or_default<K, V>(&self, key: &K) -> Result<V, KvStoreError>
    where
        K: Debug + Serialize,
        V: Debug + Default + DeserializeOwned + Serialize,
    {
        let key_vec = serialize(key)?;

        let value_slice = self
            .database
            .get_pinned(key_vec)
            .map_err(KvStoreError::Get)?;

        match value_slice {
            Some(value_slice) => deserialize(value_slice).map_err(|error| error.into()),
            None => Ok(V::default()),
        }
    }

    pub fn get_mut<K, V>(&self, key: &K) -> Result<Lock<V>, KvStoreError>
    where
        K: Debug + Serialize,
        V: Debug + DeserializeOwned + Serialize,
    {
        let key_vec = serialize(key)?;

        let transaction = self.database.transaction();

        let value_vec = transaction
            .get_for_update(&key_vec, true)
            .map_err(KvStoreError::GetMut)?
            .ok_or(KvStoreError::NoneType)?;
        let value: V = deserialize(value_vec)?;
        let locked_value = Lock::new(Some(transaction), key_vec, value);

        Ok(locked_value)
    }

    /// Get [`Lock<V>`] or put the default value for `V` and get [`Lock<V>`] if
    /// the key does not exist. Note that even if the key does not exist, the
    /// returning value might not necessarily be [`V::default()`] because
    /// internally, the operation putting [`V::default()`] and getting
    /// [`Lock<V>`] are different transactions.
    pub fn get_mut_or_default<K, V>(&self, key: &K) -> Result<Lock<V>, KvStoreError>
    where
        K: Debug + Serialize,
        V: Debug + Default + DeserializeOwned + Serialize,
    {
        let key_vec = serialize(key)?;

        let transaction = self.database.transaction();

        let value_vec = transaction
            .get_for_update(&key_vec, true)
            .map_err(KvStoreError::GetMut)?;
        match value_vec {
            Some(value_vec) => {
                let value: V = deserialize(value_vec)?;
                let locked_value = Lock::new(Some(transaction), key_vec, value);

                Ok(locked_value)
            }
            None => {
                let value = V::default();
                let value_vec = serialize(&value)?;

                transaction
                    .put(&key_vec, value_vec)
                    .map_err(KvStoreError::Put)?;

                // After the `commit()`, other threads may access [`V::default`].
                transaction.commit().map_err(KvStoreError::CommitPut)?;

                let transaction = self.database.transaction();

                transaction
                    .get_for_update(&key_vec, true)
                    .map_err(KvStoreError::GetMut)?;
                let locked_value = Lock::new(Some(transaction), key_vec, value);

                Ok(locked_value)
            }
        }
    }

    /// Apply the operation inside the closure and put the value.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use radius_sequencer_sdk::kvstore::{KvStore, Lock};
    /// use serde::{Deserialize, Serialize};
    ///
    /// #[derive(Clone, Debug, Default, Deserialize, Serialize)]
    /// pub struct User {
    ///     pub name: String,
    ///     pub age: u8,
    /// }
    ///
    /// let database = KvStore::new("database").unwrap();
    /// database.put(&"user", &User::default()).unwrap();
    /// database
    ///     .apply(&"user", |value: &mut Lock<User>| {
    ///         value.name = "User Name".to_owned();
    ///         value.age += 1;
    ///     })
    ///     .unwrap();
    ///
    /// let user: User = database.get(&"user").unwrap();
    /// println!("{:?}", user);
    /// ```
    pub fn apply<K, V, F>(&self, key: &K, operation: F) -> Result<(), KvStoreError>
    where
        K: Debug + Serialize,
        V: Debug + DeserializeOwned + Serialize,
        F: FnOnce(&mut Lock<V>),
    {
        let key_vec = serialize(key)?;

        let transaction = self.database.transaction();

        let value_vec = transaction
            .get_for_update(&key_vec, true)
            .map_err(KvStoreError::GetMut)?
            .ok_or(KvStoreError::NoneType)?;
        let value: V = deserialize(value_vec)?;

        let mut locked_value = Lock::new(Some(transaction), key_vec, value);
        operation(&mut locked_value);
        locked_value.update()?;

        Ok(())
    }

    pub fn delete<K>(&self, key: &K) -> Result<(), KvStoreError>
    where
        K: Debug + Serialize,
    {
        let key_vec = serialize(key)?;

        let transaction = self.database.transaction();

        transaction.delete(key_vec).map_err(KvStoreError::Delete)?;
        transaction.commit().map_err(KvStoreError::CommitDelete)?;

        Ok(())
    }
}

pub struct Lock<'db, V>
where
    V: Debug + Serialize + DeserializeOwned,
{
    transaction: Option<Transaction<'db, TransactionDB>>,
    key_vec: Vec<u8>,
    value: V,
}

impl<'db, V> std::ops::Deref for Lock<'db, V>
where
    V: Debug + Serialize + DeserializeOwned,
{
    type Target = V;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<'db, V> std::ops::DerefMut for Lock<'db, V>
where
    V: Debug + Serialize + DeserializeOwned,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}

impl<'db, V> Lock<'db, V>
where
    V: Debug + Serialize + DeserializeOwned,
{
    pub fn new(
        transaction: Option<Transaction<'db, TransactionDB>>,
        key_vec: Vec<u8>,
        value: V,
    ) -> Self {
        Self {
            transaction,
            key_vec,
            value,
        }
    }

    pub fn update(mut self) -> Result<(), KvStoreError> {
        if let Some(transaction) = self.transaction.take() {
            let value_vec = serialize(&self.value)?;

            transaction
                .put(&self.key_vec, value_vec)
                .map_err(KvStoreError::Update)?;
            transaction.commit().map_err(KvStoreError::CommitUpdate)?;
        }

        Ok(())
    }
}

#[derive(Debug)]
pub enum KvStoreError {
    Open(rocksdb::Error),
    DataType(crate::data_type::DataTypeError),
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

impl From<crate::data_type::DataTypeError> for KvStoreError {
    fn from(value: crate::data_type::DataTypeError) -> Self {
        Self::DataType(value)
    }
}

impl KvStoreError {
    pub fn is_none_type(&self) -> bool {
        match self {
            Self::NoneType => true,
            _others => false,
        }
    }
}
