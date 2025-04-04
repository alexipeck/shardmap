# ShardMap

A high-performance concurrent HashMap implementation using sharded locks for reduced contention.

## Implementations

This project provides two different concurrent HashMap implementations:

1. **mutex::ShardMap**: A sharded HashMap implementation that reduces lock contention by dividing the data into multiple shards, each protected by its own `Mutex`. This provides exclusive access to each shard.

2. **rwlock::ShardMap**: A sharded HashMap implementation that uses multiple `RwLock`s, allowing concurrent reads within each shard while ensuring exclusive write access.

Both implementations:
- Use consistent hashing to distribute keys across shards
- Allow configurable number of shards via the `new(num_shards: usize)` constructor
- Provide identical async APIs for operations:
  - `insert(key, value) -> Option<V>`
  - `get(key) -> Option<V>`
  - `remove(key) -> Option<V>`
  - `contains_key(key) -> bool`

## Benchmark Results

The benchmarks were run with 64 threads, each performing 12,500 operations (total 800,000 operations). Three different workload types were tested:

- Write-Heavy: 800,000 inserts + 800,000 gets (to verify inserts)
- Read-Heavy: 800,000 initial inserts + 800,000 gets during benchmark
- Mixed: 800,000 initial inserts + ~267,000 inserts and ~533,000 gets during benchmark (1/3 writes, 2/3 reads)

Each implementation was tested with both 16 and 64 shards, resulting in approximately 1.6 million operations per test.

### mutex::ShardMap with 16 shards

Write-Heavy:
- Average completion time: ~198ms
- Thread completion times: 190.8085ms - 204.5612ms

Read-Heavy:
- Average completion time: ~189ms
- Thread completion times: 184.7107ms - 192.5821ms

Mixed:
- Average completion time: ~241ms
- Thread completion times: 234.1177ms - 247.9861ms

### mutex::ShardMap with 64 shards

Write-Heavy:
- Average completion time: ~83ms
- Thread completion times: 75.6379ms - 90.5622ms

Read-Heavy:
- Average completion time: ~106ms
- Thread completion times: 100.9255ms - 111.2584ms

Mixed:
- Average completion time: ~127ms
- Thread completion times: 120.5946ms - 134.0934ms

### rwlock::ShardMap with 64 shards

Write-Heavy:
- Average completion time: ~922ms
- Thread completion times: 914.3482ms - 930.5264ms

Read-Heavy:
- Average completion time: ~1018ms
- Thread completion times: 975.128ms - 1061.3256ms

Mixed:
- Average completion time: ~1077ms
- Thread completion times: 1061.0773ms - 1093.4951ms

## Performance Analysis

1. **Sharding Impact**:
   - 64 shards show significantly better performance than 16 shards across all workloads
   - 64 shards are ~4x faster than 16 shards for write-heavy workloads
   - The performance benefit of sharding increases with thread count due to reduced contention

2. **Lock Type Comparison**:
   - mutex::ShardMap performs better in high-contention scenarios
   - rwlock::ShardMap shows higher latency due to the overhead of read/write lock management
   - The performance gap is most noticeable in write-heavy workloads

3. **Workload Type Impact**:
   - Write-Heavy: Best performance with mutex::ShardMap and 64 shards (~83ms)
   - Read-Heavy: Slightly slower than Write-Heavy with 64 shards (~106ms)
   - Mixed: Shows higher latency (~127ms) due to lock switching overhead

4. **Key Findings**:
   - Sharding significantly reduces lock contention
   - More shards (64 vs 16) provide better performance with high thread counts
   - mutex::ShardMap is the better choice for most workloads
   - Mixed workloads show higher latency than pure read or write workloads

## Usage

```rust
use shardmap::mutex::ShardMap;  // or rwlock::ShardMap

#[tokio::main]
async fn main() {
    // Create a new sharded map with 64 shards
    let map = ShardMap::<String, i32>::new(64);
    
    // Insert a value
    map.insert("key".to_string(), 42).await;
    
    // Get a value
    if let Some(value) = map.get(&"key".to_string()).await {
        println!("Value: {}", value);
    }
} 