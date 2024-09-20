use std::{
    any::{type_name, Any},
    collections::HashMap,
    fmt::Debug,
    sync::Arc,
};

use serde::Serialize;
use tokio::sync::{Mutex, MutexGuard, OwnedMutexGuard};

type Key = Vec<u8>;
type ValueAny = Box<dyn Any + Send + Sync>;

fn serialize_key<K>(key: &K) -> Result<Key, CachedKvStoreError>
where
    K: Debug + Serialize,
{
    bincode::serialize(key).map_err(|error| CachedKvStoreError::Serialize {
        type_name: type_name::<K>(),
        data: format!("{:?}", key),
        error,
    })
}

fn downcast<V>(
    database: MutexGuard<'_, HashMap<Key, ValueAny>>,
    key_vec: Vec<u8>,
) -> Result<Arc<Mutex<V>>, CachedKvStoreError>
where
    V: Clone + Any + Send + 'static,
{
    let value = database
        .get(&key_vec)
        .ok_or(CachedKvStoreError::KeyError(type_name::<V>()))?
        .downcast_ref::<Arc<Mutex<V>>>()
        .ok_or(CachedKvStoreError::Downcast(type_name::<V>()))?
        .clone();

    Ok(value)
}

pub struct CachedKvStore {
    inner: Arc<Mutex<HashMap<Key, ValueAny>>>,
}

unsafe impl Send for CachedKvStore {}

unsafe impl Sync for CachedKvStore {}

impl Clone for CachedKvStore {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl Default for CachedKvStore {
    fn default() -> Self {
        Self {
            inner: Arc::new(Mutex::new(HashMap::default())),
        }
    }
}

impl CachedKvStore {
    pub fn blocking_put<K, V>(&self, key: &K, value: V) -> Result<(), CachedKvStoreError>
    where
        K: Debug + Serialize,
        V: Clone + Any + Send + 'static,
    {
        let key_vec = serialize_key(key)?;
        let value_any: ValueAny = Box::new(Arc::new(Mutex::new(value)));

        let mut database = self.inner.blocking_lock();
        database.insert(key_vec, value_any);

        Ok(())
    }

    pub async fn put<K, V>(&self, key: &K, value: V) -> Result<(), CachedKvStoreError>
    where
        K: Debug + Serialize,
        V: Clone + Any + Send + 'static,
    {
        let key_vec = serialize_key(key)?;
        let value_any: ValueAny = Box::new(Arc::new(Mutex::new(value)));

        let mut database = self.inner.lock().await;
        database.insert(key_vec, value_any);

        Ok(())
    }

    pub fn blocking_get<K, V>(&self, key: &K) -> Result<V, CachedKvStoreError>
    where
        K: Debug + Serialize,
        V: Clone + Any + Send + 'static,
    {
        let key_vec = serialize_key(key)?;

        let database = self.inner.blocking_lock();
        let value = downcast::<V>(database, key_vec)?;

        let value_inner = value.blocking_lock().clone();

        Ok(value_inner)
    }

    pub async fn get<K, V>(&self, key: &K) -> Result<V, CachedKvStoreError>
    where
        K: Debug + Serialize,
        V: Clone + Any + Send + 'static,
    {
        let key_vec = serialize_key(key)?;

        let database = self.inner.lock().await;
        let value = downcast::<V>(database, key_vec)?;

        let value_inner = value.lock().await.clone();

        Ok(value_inner)
    }

    pub fn blocking_get_mut<K, V>(&self, key: &K) -> Result<Value<V>, CachedKvStoreError>
    where
        K: Debug + Serialize,
        V: Clone + Any + Send + 'static,
    {
        let key_vec = serialize_key(key)?;

        let database = self.inner.blocking_lock();
        let value = downcast::<V>(database, key_vec)?;

        Ok(Value::blocking_lock(value))
    }

    pub async fn get_mut<K, V>(&self, key: &K) -> Result<Value<V>, CachedKvStoreError>
    where
        K: Debug + Serialize,
        V: Clone + Any + Send + 'static,
    {
        let key_vec = serialize_key(key)?;

        let database = self.inner.lock().await;
        let value = downcast::<V>(database, key_vec)?;

        Ok(Value::lock(value).await)
    }

    pub fn blocking_delete<K, V>(&self, key: &K) -> Result<(), CachedKvStoreError>
    where
        K: Debug + Serialize,
        V: Clone + Any + Send + 'static,
    {
        let key_vec = serialize_key(key)?;

        let mut database = self.inner.blocking_lock();
        database.remove(&key_vec);

        Ok(())
    }

    pub async fn delete<K, V>(&self, key: &K) -> Result<(), CachedKvStoreError>
    where
        K: Debug + Serialize,
        V: Clone + Any + Send + 'static,
    {
        let key_vec = serialize_key(key)?;

        let mut database = self.inner.lock().await;
        database.remove(&key_vec);

        Ok(())
    }
}

/// An owned mutex equivalent to [`crate::Lock`] except that [`Value<V>`] does
/// not require the user to call [`crate::Lock::update()`].
///
/// # Examples
///
/// ```rust
/// // Async context
/// #[derive(Clone, Debug)]
/// pub struct User {
///     pub name: String,
///     pub age: u8,
/// }
///
/// let database = CachedKvStore::default();
///
/// let user = User {
///     name: "User Name".to_owned(),
///     age: 32,
/// };
///
/// database.put(&"user", user).await.unwrap();
/// let mut user: Value<User> = database.get_mut(&"user").await.unwrap();
/// user.age += 1;
///
/// // Other threads may access the value after `drop(user);`.
/// drop(user);
///
/// let user: User = database.get(&"user").await.unwrap();
///
/// // Age increased by 1.
/// println!("{:?}", user);
/// ```
#[derive(Debug)]
pub struct Value<V>(OwnedMutexGuard<V>);

unsafe impl<V> Send for Value<V> {}

unsafe impl<V> Sync for Value<V> {}

impl<V> std::ops::Deref for Value<V> {
    type Target = OwnedMutexGuard<V>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<V> std::ops::DerefMut for Value<V> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<V> Value<V> {
    pub fn blocking_lock(value: Arc<Mutex<V>>) -> Self {
        Self(value.blocking_lock_owned())
    }

    pub async fn lock(value: Arc<Mutex<V>>) -> Self {
        Self(value.lock_owned().await)
    }
}

#[derive(Debug)]
pub enum CachedKvStoreError {
    Serialize {
        type_name: &'static str,
        data: String,
        error: bincode::Error,
    },
    KeyError(&'static str),
    Downcast(&'static str),
}

impl std::fmt::Display for CachedKvStoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for CachedKvStoreError {}
