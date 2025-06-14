use std::{
    collections::{HashMap, HashSet},
    hash::{Hash, Hasher},
    sync::Arc,
};
use tokio::sync::RwLock;

pub struct ShardMap<K, V> {
    shards: Vec<Arc<RwLock<HashMap<K, V>>>>,
    num_shards: usize,
}

impl<K, V> ShardMap<K, V>
where
    K: Hash + Eq + Clone,
    V: Clone,
{
    pub fn new(num_shards: usize) -> Self {
        let mut shards = Vec::with_capacity(num_shards);
        for _ in 0..num_shards {
            shards.push(Default::default());
        }
        ShardMap { shards, num_shards }
    }

    fn get_shard_index(&self, key: &K) -> usize {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        key.hash(&mut hasher);
        (hasher.finish() as usize) % self.num_shards
    }

    pub async fn insert(&self, key: K, value: V) -> Option<V> {
        let shard_index = self.get_shard_index(&key);
        let mut shard = self.shards[shard_index].write().await;
        shard.insert(key, value)
    }

    pub async fn get<F>(&self, key: &K, f: F) -> bool
    where
        F: FnOnce(&V),
    {
        let shard_index = self.get_shard_index(key);
        let guard = self.shards[shard_index].read().await;
        if let Some(value) = guard.get(key) {
            f(value);
            true
        } else {
            false
        }
    }

    pub async fn map<F, U, R>(&self, key: &K, f: F) -> Option<R>
    where
        F: FnOnce(&V) -> U,
        U: Into<Option<R>>,
    {
        let shard_index = self.get_shard_index(key);
        let guard = self.shards[shard_index].read().await;
        guard.get(key).map(f).and_then(|u| u.into())
    }

    pub async fn get_cloned(&self, key: &K) -> Option<V> {
        let shard_index = self.get_shard_index(key);
        let guard = self.shards[shard_index].read().await;
        guard.get(key).cloned()
    }

    pub async fn get_mut<F>(&self, key: &K, f: F) -> bool
    where
        F: FnOnce(&mut V),
    {
        let shard_index = self.get_shard_index(key);
        let mut guard = self.shards[shard_index].write().await;
        if let Some(value) = guard.get_mut(key) {
            f(value);
            true
        } else {
            false
        }
    }

    pub async fn get_mut_then_clone<F>(&self, key: &K, f: F) -> Option<V>
    where
        F: FnOnce(&mut V),
    {
        let shard_index = self.get_shard_index(key);
        let mut guard = self.shards[shard_index].write().await;
        if let Some(value) = guard.get_mut(key) {
            f(value);
            Some(value.clone())
        } else {
            None
        }
    }

    pub async fn remove(&self, key: &K) -> Option<V> {
        let shard_index = self.get_shard_index(key);
        let mut shard = self.shards[shard_index].write().await;
        shard.remove(key)
    }

    pub async fn contains_key(&self, key: &K) -> bool {
        let shard_index = self.get_shard_index(key);
        let shard = self.shards[shard_index].read().await;
        shard.contains_key(key)
    }
}

pub struct ShardSet<K> {
    shards: Vec<Arc<RwLock<HashSet<K>>>>,
    num_shards: usize,
}

impl<K> ShardSet<K>
where
    K: Hash + Eq + Clone,
{
    pub fn new(num_shards: usize) -> Self {
        let mut shards = Vec::with_capacity(num_shards);
        for _ in 0..num_shards {
            shards.push(Default::default());
        }
        ShardSet { shards, num_shards }
    }

    fn get_shard_index(&self, key: &K) -> usize {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        key.hash(&mut hasher);
        (hasher.finish() as usize) % self.num_shards
    }

    pub async fn insert(&self, key: K) -> bool {
        let shard_index = self.get_shard_index(&key);
        let mut shard = self.shards[shard_index].write().await;
        shard.insert(key)
    }

    pub async fn contains(&self, key: &K) -> bool {
        let shard_index = self.get_shard_index(key);
        let shard = self.shards[shard_index].read().await;
        shard.contains(key)
    }

    pub async fn remove(&self, key: &K) -> bool {
        let shard_index = self.get_shard_index(key);
        let mut shard = self.shards[shard_index].write().await;
        shard.remove(key)
    }
}
