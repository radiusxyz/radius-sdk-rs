use std::{any::type_name, fmt::Debug, path::Path, sync::Arc};

use rocksdb::{Options, Transaction, TransactionDB, TransactionDBOptions};
use serde::{de::DeserializeOwned, ser::Serialize};

use crate::error::KvStoreError;

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

    pub fn get<K, V>(&self, key: &K) -> Result<V, KvStoreError>
    where
        K: Debug + Serialize,
        V: Debug + DeserializeOwned + Serialize,
    {
        let key_vec = bincode::serialize(key).map_err(|error| KvStoreError::Serialize {
            type_name: type_name::<K>(),
            data: format!("{:?}", key),
            error,
        })?;

        let value_slice = self
            .database
            .get_pinned(key_vec)
            .map_err(KvStoreError::Get)?
            .ok_or(KvStoreError::NoneType)?;

        let value: V = bincode::deserialize(value_slice.as_ref()).map_err(|error| {
            KvStoreError::Deserialize {
                type_name: type_name::<V>(),
                error,
            }
        })?;

        Ok(value)
    }

    /// Get the value or return `V::default()`.
    pub fn get_or_default<K, V>(&self, key: &K) -> Result<V, KvStoreError>
    where
        K: Debug + Serialize,
        V: Debug + Default + DeserializeOwned + Serialize,
    {
        let key_vec = bincode::serialize(key).map_err(|error| KvStoreError::Serialize {
            type_name: type_name::<K>(),
            data: format!("{:?}", key),
            error,
        })?;

        let value_slice = self
            .database
            .get_pinned(key_vec)
            .map_err(KvStoreError::Get)?;

        match value_slice {
            Some(value_slice) => {
                let value: V = bincode::deserialize(value_slice.as_ref()).map_err(|error| {
                    KvStoreError::Deserialize {
                        type_name: type_name::<V>(),
                        error,
                    }
                })?;

                Ok(value)
            }
            None => Ok(V::default()),
        }
    }

    pub fn get_mut<K, V>(&self, key: &K) -> Result<Lock<V>, KvStoreError>
    where
        K: Debug + Serialize,
        V: Debug + DeserializeOwned + Serialize,
    {
        let key_vec = bincode::serialize(key).map_err(|error| KvStoreError::Serialize {
            type_name: type_name::<K>(),
            data: format!("{:?}", key),
            error,
        })?;

        let transaction = self.database.transaction();

        let value_vec = transaction
            .get_for_update(&key_vec, true)
            .map_err(KvStoreError::GetMut)?
            .ok_or(KvStoreError::NoneType)?;
        let value: V =
            bincode::deserialize(&value_vec).map_err(|error| KvStoreError::Deserialize {
                type_name: type_name::<V>(),
                error,
            })?;

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
        let key_vec = bincode::serialize(key).map_err(|error| KvStoreError::Serialize {
            type_name: type_name::<K>(),
            data: format!("{:?}", key),
            error,
        })?;

        let transaction = self.database.transaction();

        let value_vec = transaction
            .get_for_update(&key_vec, true)
            .map_err(KvStoreError::GetMut)?;
        match value_vec {
            Some(value_vec) => {
                let value: V = bincode::deserialize(&value_vec).map_err(|error| {
                    KvStoreError::Deserialize {
                        type_name: type_name::<V>(),
                        error,
                    }
                })?;
                let locked_value = Lock::new(Some(transaction), key_vec, value);

                Ok(locked_value)
            }
            None => {
                let value = V::default();
                let value_vec =
                    bincode::serialize(&value).map_err(|error| KvStoreError::Serialize {
                        type_name: type_name::<V>(),
                        data: format!("{:?}", value),
                        error,
                    })?;

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
        let key_vec = bincode::serialize(key).map_err(|error| KvStoreError::Serialize {
            type_name: type_name::<K>(),
            data: format!("{:?}", key),
            error,
        })?;

        let transaction = self.database.transaction();

        let value_vec = transaction
            .get_for_update(&key_vec, true)
            .map_err(KvStoreError::GetMut)?
            .ok_or(KvStoreError::NoneType)?;
        let value: V =
            bincode::deserialize(&value_vec).map_err(|error| KvStoreError::Deserialize {
                type_name: type_name::<V>(),
                error,
            })?;

        let mut locked_value = Lock::new(Some(transaction), key_vec, value);
        operation(&mut locked_value);
        locked_value.update()?;

        Ok(())
    }

    pub fn put<K, V>(&self, key: &K, value: &V) -> Result<(), KvStoreError>
    where
        K: Debug + Serialize,
        V: Debug + DeserializeOwned + Serialize,
    {
        let key_vec = bincode::serialize(key).map_err(|error| KvStoreError::Serialize {
            type_name: type_name::<K>(),
            data: format!("{:?}", key),
            error,
        })?;

        let value_vec = bincode::serialize(value).map_err(|error| KvStoreError::Serialize {
            type_name: type_name::<V>(),
            data: format!("{:?}", value),
            error,
        })?;

        let transaction = self.database.transaction();

        transaction
            .put(key_vec, value_vec)
            .map_err(KvStoreError::Put)?;
        transaction.commit().map_err(KvStoreError::CommitPut)?;

        Ok(())
    }

    pub fn delete<K>(&self, key: &K) -> Result<(), KvStoreError>
    where
        K: Debug + Serialize,
    {
        let key_vec = bincode::serialize(key).map_err(|error| KvStoreError::Serialize {
            type_name: type_name::<K>(),
            data: format!("{:?}", key),
            error,
        })?;

        let transaction = self.database.transaction();

        transaction.delete(key_vec).map_err(KvStoreError::Delete)?;
        transaction.commit().map_err(KvStoreError::CommitDelete)?;

        Ok(())
    }
}

pub struct Lock<'db, V>
where
    V: Debug + Serialize,
{
    transaction: Option<Transaction<'db, TransactionDB>>,
    key_vec: Vec<u8>,
    value: V,
}

impl<'db, V> std::ops::Deref for Lock<'db, V>
where
    V: Debug + Serialize,
{
    type Target = V;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<'db, V> std::ops::DerefMut for Lock<'db, V>
where
    V: Debug + Serialize,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}

impl<'db, V> Lock<'db, V>
where
    V: Debug + Serialize,
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
            let value_vec =
                bincode::serialize(&self.value).map_err(|error| KvStoreError::Serialize {
                    type_name: type_name::<V>(),
                    data: format!("{:?}", self.value),
                    error,
                })?;

            transaction
                .put(&self.key_vec, value_vec)
                .map_err(KvStoreError::Update)?;
            transaction.commit().map_err(KvStoreError::CommitUpdate)?;
        }

        Ok(())
    }
}
