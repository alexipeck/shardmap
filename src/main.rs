use shardmap::{
    mutex::{ShardMap as MutexShardMap, ShardSet as MutexShardSet},
    rwlock::{ShardMap as RwLockShardMap, ShardSet as RwLockShardSet},
};
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::Barrier;

// Define workload types
#[derive(Clone)]
enum WorkloadType {
    WriteHeavy,
    ReadHeavy,
    Mixed,
}

async fn run_benchmark<K, V, F>(
    map: Arc<K>,
    num_threads: usize,
    operations_per_thread: usize,
    workload_type: WorkloadType,
    operation: F,
) where
    K: Send + Sync + 'static,
    F: Fn(Arc<K>, Arc<Barrier>, usize, usize, WorkloadType) -> tokio::task::JoinHandle<()>
        + Send
        + Sync
        + 'static,
{
    let barrier = Arc::new(Barrier::new(num_threads + 1));
    let mut handles = vec![];

    for thread_id in 0..num_threads {
        let barrier = Arc::clone(&barrier);
        handles.push(operation(
            Arc::clone(&map),
            barrier,
            thread_id,
            operations_per_thread,
            workload_type.clone(),
        ));
    }

    println!("Waiting for all threads to initialize...");
    barrier.wait().await;
    println!("All threads started, waiting for completion...");

    for handle in handles {
        handle.await.unwrap();
    }
}

#[tokio::main]
async fn main() {
    let num_threads = 64;
    let operations_per_thread = 12_500;
    let shard_counts = [16, 64];
    let workload_types = [
        (WorkloadType::WriteHeavy, "Write-Heavy"),
        (WorkloadType::ReadHeavy, "Read-Heavy"),
        (WorkloadType::Mixed, "Mixed"),
    ];

    println!(
        "Starting benchmarks with {} threads, {} operations per thread (total: {} operations)",
        num_threads,
        operations_per_thread,
        num_threads * operations_per_thread
    );

    // Benchmark ShardMap implementations
    for &shard_count in &shard_counts {
        for (workload_type, workload_name) in &workload_types {
            println!(
                "\nBenchmarking MutexShardMap with {} shards ({})",
                shard_count, workload_name
            );
            match workload_type {
                WorkloadType::WriteHeavy => println!(
                    "Operations: {} inserts + {} gets",
                    num_threads * operations_per_thread,
                    num_threads * operations_per_thread
                ),
                WorkloadType::ReadHeavy => println!(
                    "Operations: {} initial inserts + {} gets",
                    num_threads * operations_per_thread,
                    num_threads * operations_per_thread
                ),
                WorkloadType::Mixed => println!(
                    "Operations: {} initial inserts + ~{} inserts + ~{} gets",
                    num_threads * operations_per_thread,
                    (num_threads * operations_per_thread) / 3,
                    (num_threads * operations_per_thread * 2) / 3
                ),
            }
            let shard_map = Arc::new(MutexShardMap::<String, i32>::new(shard_count));
            run_benchmark::<MutexShardMap<String, i32>, (), _>(
                Arc::clone(&shard_map),
                num_threads,
                operations_per_thread,
                workload_type.clone(),
                |map, barrier, thread_id, ops, workload_type| {
                    tokio::spawn(async move {
                        barrier.wait().await;
                        let start = Instant::now();

                        if matches!(workload_type, WorkloadType::ReadHeavy | WorkloadType::Mixed) {
                            for i in 0..ops {
                                let key = format!("key_{}_{}", thread_id, i);
                                map.insert(key.clone(), i as i32).await;
                            }
                        }

                        for i in 0..ops {
                            let key = format!("key_{}_{}", thread_id, i);

                            match workload_type {
                                WorkloadType::WriteHeavy => {
                                    map.insert(key.clone(), i as i32).await;
                                    map.get(&key, |value| {
                                        assert_eq!(*value, i as i32);
                                    }).await;
                                }
                                WorkloadType::ReadHeavy => {
                                    map.get(&key, |value| {
                                        assert_eq!(*value, i as i32);
                                    }).await;
                                }
                                WorkloadType::Mixed => {
                                    if i % 3 == 0 {
                                        map.insert(key.clone(), i as i32).await;
                                    } else {
                                        map.get(&key, |value| {
                                            assert_eq!(*value, i as i32);
                                        }).await;
                                    }
                                }
                            }
                        }

                        let duration = start.elapsed();
                        println!("Thread {} completed in {:?}", thread_id, duration);
                    })
                },
            )
            .await;
        }
    }

    // Benchmark RwLockShardMap
    for (workload_type, workload_name) in &workload_types {
        println!(
            "\nBenchmarking RwLockShardMap with 64 shards ({})",
            workload_name
        );
        match workload_type {
            WorkloadType::WriteHeavy => println!(
                "Operations: {} inserts + {} gets",
                num_threads * operations_per_thread,
                num_threads * operations_per_thread
            ),
            WorkloadType::ReadHeavy => println!(
                "Operations: {} initial inserts + {} gets",
                num_threads * operations_per_thread,
                num_threads * operations_per_thread
            ),
            WorkloadType::Mixed => println!(
                "Operations: {} initial inserts + ~{} inserts + ~{} gets",
                num_threads * operations_per_thread,
                (num_threads * operations_per_thread) / 3,
                (num_threads * operations_per_thread * 2) / 3
            ),
        }
        let rwlock_map = Arc::new(RwLockShardMap::<String, i32>::new(64));
        run_benchmark::<RwLockShardMap<String, i32>, (), _>(
            Arc::clone(&rwlock_map),
            num_threads,
            operations_per_thread,
            workload_type.clone(),
            |map, barrier, thread_id, ops, workload_type| {
                tokio::spawn(async move {
                    barrier.wait().await;
                    let start = Instant::now();

                    if matches!(workload_type, WorkloadType::ReadHeavy | WorkloadType::Mixed) {
                        for i in 0..ops {
                            let key = format!("key_{}_{}", thread_id, i);
                            map.insert(key.clone(), i as i32).await;
                        }
                    }

                    for i in 0..ops {
                        let key = format!("key_{}_{}", thread_id, i);

                        match workload_type {
                            WorkloadType::WriteHeavy => {
                                map.insert(key.clone(), i as i32).await;
                                map.get(&key, |value| {
                                    assert_eq!(*value, i as i32);
                                }).await;
                            }
                            WorkloadType::ReadHeavy => {
                                map.get(&key, |value| {
                                    assert_eq!(*value, i as i32);
                                }).await;
                            }
                            WorkloadType::Mixed => {
                                if i % 3 == 0 {
                                    map.insert(key.clone(), i as i32).await;
                                } else {
                                    map.get(&key, |value| {
                                        assert_eq!(*value, i as i32);
                                    }).await;
                                }
                            }
                        }
                    }

                    let duration = start.elapsed();
                    println!("Thread {} completed in {:?}", thread_id, duration);
                })
            },
        )
        .await;
    }

    // Benchmark ShardSet implementations
    for &shard_count in &shard_counts {
        for (workload_type, workload_name) in &workload_types {
            println!(
                "\nBenchmarking MutexShardSet with {} shards ({})",
                shard_count, workload_name
            );
            match workload_type {
                WorkloadType::WriteHeavy => println!(
                    "Operations: {} inserts + {} contains",
                    num_threads * operations_per_thread,
                    num_threads * operations_per_thread
                ),
                WorkloadType::ReadHeavy => println!(
                    "Operations: {} initial inserts + {} contains",
                    num_threads * operations_per_thread,
                    num_threads * operations_per_thread
                ),
                WorkloadType::Mixed => println!(
                    "Operations: {} initial inserts + ~{} inserts + ~{} contains",
                    num_threads * operations_per_thread,
                    (num_threads * operations_per_thread) / 3,
                    (num_threads * operations_per_thread * 2) / 3
                ),
            }
            let shard_set = Arc::new(MutexShardSet::<String>::new(shard_count));
            run_benchmark::<MutexShardSet<String>, (), _>(
                Arc::clone(&shard_set),
                num_threads,
                operations_per_thread,
                workload_type.clone(),
                |set, barrier, thread_id, ops, workload_type| {
                    tokio::spawn(async move {
                        barrier.wait().await;
                        let start = Instant::now();

                        if matches!(workload_type, WorkloadType::ReadHeavy | WorkloadType::Mixed) {
                            for i in 0..ops {
                                let key = format!("key_{}_{}", thread_id, i);
                                set.insert(key).await;
                            }
                        }

                        for i in 0..ops {
                            let key = format!("key_{}_{}", thread_id, i);

                            match workload_type {
                                WorkloadType::WriteHeavy => {
                                    set.insert(key.clone()).await;
                                    assert!(set.contains(&key).await);
                                }
                                WorkloadType::ReadHeavy => {
                                    assert!(set.contains(&key).await);
                                }
                                WorkloadType::Mixed => {
                                    if i % 3 == 0 {
                                        set.insert(key.clone()).await;
                                    } else {
                                        assert!(set.contains(&key).await);
                                    }
                                }
                            }
                        }

                        let duration = start.elapsed();
                        println!("Thread {} completed in {:?}", thread_id, duration);
                    })
                },
            )
            .await;
        }
    }

    // Benchmark RwLockShardSet
    for (workload_type, workload_name) in &workload_types {
        println!(
            "\nBenchmarking RwLockShardSet with 64 shards ({})",
            workload_name
        );
        match workload_type {
            WorkloadType::WriteHeavy => println!(
                "Operations: {} inserts + {} contains",
                num_threads * operations_per_thread,
                num_threads * operations_per_thread
            ),
            WorkloadType::ReadHeavy => println!(
                "Operations: {} initial inserts + {} contains",
                num_threads * operations_per_thread,
                num_threads * operations_per_thread
            ),
            WorkloadType::Mixed => println!(
                "Operations: {} initial inserts + ~{} inserts + ~{} contains",
                num_threads * operations_per_thread,
                (num_threads * operations_per_thread) / 3,
                (num_threads * operations_per_thread * 2) / 3
            ),
        }
        let rwlock_set = Arc::new(RwLockShardSet::<String>::new(64));
        run_benchmark::<RwLockShardSet<String>, (), _>(
            Arc::clone(&rwlock_set),
            num_threads,
            operations_per_thread,
            workload_type.clone(),
            |set, barrier, thread_id, ops, workload_type| {
                tokio::spawn(async move {
                    barrier.wait().await;
                    let start = Instant::now();

                    if matches!(workload_type, WorkloadType::ReadHeavy | WorkloadType::Mixed) {
                        for i in 0..ops {
                            let key = format!("key_{}_{}", thread_id, i);
                            set.insert(key).await;
                        }
                    }

                    for i in 0..ops {
                        let key = format!("key_{}_{}", thread_id, i);

                        match workload_type {
                            WorkloadType::WriteHeavy => {
                                set.insert(key.clone()).await;
                                assert!(set.contains(&key).await);
                            }
                            WorkloadType::ReadHeavy => {
                                assert!(set.contains(&key).await);
                            }
                            WorkloadType::Mixed => {
                                if i % 3 == 0 {
                                    set.insert(key.clone()).await;
                                } else {
                                    assert!(set.contains(&key).await);
                                }
                            }
                        }
                    }

                    let duration = start.elapsed();
                    println!("Thread {} completed in {:?}", thread_id, duration);
                })
            },
        )
        .await;
    }
}
