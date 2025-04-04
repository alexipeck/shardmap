use std::{
    collections::HashSet,
    hash::{Hash, Hasher},
    sync::Arc,
};
use tokio::sync::Mutex;

pub struct ShardSet<K> {
    shards: Vec<Arc<Mutex<HashSet<K>>>>,
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
        let mut shard = self.shards[shard_index].lock().await;
        shard.insert(key)
    }

    pub async fn contains(&self, key: &K) -> bool {
        let shard_index = self.get_shard_index(key);
        let shard = self.shards[shard_index].lock().await;
        shard.contains(key)
    }

    pub async fn remove(&self, key: &K) -> bool {
        let shard_index = self.get_shard_index(key);
        let mut shard = self.shards[shard_index].lock().await;
        shard.remove(key)
    }
}
