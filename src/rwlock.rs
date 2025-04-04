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

    pub async fn get(&self, key: &K) -> Option<V> {
        let shard_index = self.get_shard_index(key);
        let shard = self.shards[shard_index].read().await;
        shard.get(key).cloned()
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
