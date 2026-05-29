// SPDX-FileCopyrightText: 2026 Epic Games, Inc.
// SPDX-License-Identifier: MIT
use std::sync::Arc;
use std::sync::Weak;
use std::time::Duration;

use crate::immutable_store::ImmutableStore;

/// Evictor task: enforces max capacity at regular intervals.
pub async fn evictor(
    store: Weak<dyn ImmutableStore>,
    max_capacity: usize,
    eviction_delay: Option<Duration>,
    sync_data: bool,
) {
    use std::cmp::max;

    let max_capacity = max(max_capacity, 1024 * 1024);
    let eviction_delay = eviction_delay.unwrap_or(Duration::from_secs(10));
    tokio::time::sleep(Duration::from_millis(100)).await;
    lore_base::lore_debug!("Store evictor enforcing max capacity of {max_capacity}");
    loop {
        {
            let Some(real_store) = store.upgrade() else {
                break;
            };
            if let Err(err) = real_store.evict(max_capacity, sync_data).await {
                lore_base::lore_warn!("Store evictor failed: {err}");
            }
        }
        tokio::time::sleep(eviction_delay).await;
    }
    lore_base::lore_debug!("Store evictor exiting");
}

/// Compactor task: enforces max size at regular intervals.
pub async fn compactor(
    store: Weak<dyn ImmutableStore>,
    max_size: usize,
    compaction_delay: Option<Duration>,
    sync_data: bool,
) {
    let compaction_delay = compaction_delay.unwrap_or(Duration::from_secs(60 * 60 * 24));
    lore_base::lore_debug!("Store compactor enforcing max size of {max_size}");
    let mut at = if let Some(store) = store.upgrade() {
        store.compact_resume_at().await
    } else {
        None
    };
    loop {
        {
            let Some(real_store) = store.upgrade() else {
                break;
            };
            match real_store.compact(max_size, at, sync_data).await {
                Ok(Some(step_at)) => {
                    at = Some(step_at);
                    lore_base::lore_debug!(
                        "Store compactor completed a step, now at {}",
                        at.unwrap_or_default()
                    );
                }
                Ok(None) => {
                    at = None;
                    lore_base::lore_debug!("Store compactor finished");
                }
                Err(err) => {
                    lore_base::lore_warn!("Store compactor failed: {err}");
                    break;
                }
            }
        }
        if at.is_none() {
            tokio::time::sleep(compaction_delay).await;
        }
    }
    lore_base::lore_debug!("Store compactor exiting");
}

/// Run compaction and eviction in a single pass.
pub async fn gc(
    store: Arc<dyn ImmutableStore>,
    max_size: usize,
    max_capacity: usize,
    sync_data: bool,
) {
    let mut at = store.clone().compact_resume_at().await;

    if max_size > 0 {
        loop {
            let store = store.clone();
            match store.clone().compact(max_size, at, sync_data).await {
                Ok(Some(step_at)) => {
                    at = Some(step_at);
                    lore_base::lore_debug!(
                        "Store compactor completed a step, now at {}",
                        at.unwrap_or_default()
                    );
                }
                Ok(None) => {
                    lore_base::lore_debug!("Store compactor finished");
                    break;
                }
                Err(err) => {
                    lore_base::lore_warn!("Store compactor failed: {err}");
                    break;
                }
            }
        }
        lore_base::lore_debug!("Store compactor done");
    }

    if max_capacity > 0 {
        let _ = store.evict(max_capacity, sync_data).await;
        lore_base::lore_debug!("Store evictor done");
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;
    use crate::local::immutable_store::ImmutableStoreSettings;
    use crate::local::immutable_store::LocalImmutableStore;
    use crate::test_util::TempDir;

    fn generate_tempdir() -> TempDir {
        TempDir::new("lore-storage-maintenance-test-")
    }

    async fn create_test_store(path: Option<PathBuf>) -> Arc<dyn ImmutableStore> {
        LocalImmutableStore::new(path, ImmutableStoreSettings::default())
            .await
            .unwrap()
    }

    #[tokio::test]
    async fn gc_compaction_reduces_fragment_count() {
        let dir = generate_tempdir();
        let store = create_test_store(Some(dir.to_path_buf())).await;

        let partition = crate::Partition::from([0x01; 16]);

        for i in 0u8..10 {
            let data = vec![i; 1024];
            let hash = crate::hash_slice(&data);
            let address = crate::Address {
                hash,
                context: crate::Context::from([i; 16]),
            };
            let frag = crate::Fragment {
                flags: 0,
                size_payload: data.len() as u32,
                size_content: data.len() as u64,
            };
            store
                .clone()
                .put(
                    partition,
                    address,
                    frag,
                    Some(bytes::Bytes::from(data)),
                    false,
                )
                .await
                .unwrap();
        }

        store.clone().flush(true).await.unwrap();

        let count_before = store.clone().fragment_count().await;

        // Run gc with a very small max_size to trigger compaction, and small capacity
        // for eviction (1 byte to force eviction).
        gc(store.clone(), 1, 1, false).await;

        let count_after = store.clone().fragment_count().await;

        assert!(
            count_after.unwrap_or(0) < count_before.unwrap_or(0),
            "gc should reduce fragment count: before={count_before:?}, after={count_after:?}"
        );
    }

    #[tokio::test]
    async fn gc_skips_compaction_when_max_size_zero() {
        let store = create_test_store(None).await;
        gc(store, 0, 0, false).await;
    }

    #[tokio::test]
    async fn gc_runs_eviction_only() {
        let dir = generate_tempdir();
        let store = create_test_store(Some(dir.to_path_buf())).await;

        let partition = crate::Partition::from([0x02; 16]);

        for i in 0u8..5 {
            let data = vec![i; 2048];
            let hash = crate::hash_slice(&data);
            let address = crate::Address {
                hash,
                context: crate::Context::from([i; 16]),
            };
            let frag = crate::Fragment {
                flags: 0,
                size_payload: data.len() as u32,
                size_content: data.len() as u64,
            };
            store
                .clone()
                .put(
                    partition,
                    address,
                    frag,
                    Some(bytes::Bytes::from(data)),
                    false,
                )
                .await
                .unwrap();
        }
        store.clone().flush(true).await.unwrap();

        // max_size=0 skips compaction, max_capacity=1 triggers eviction
        gc(store.clone(), 0, 1, false).await;

        let count = store.clone().fragment_count().await.unwrap_or(0);
        assert!(
            count < 5,
            "eviction should have removed some fragments: count={count}"
        );
    }

    #[tokio::test]
    async fn evictor_exits_when_store_dropped() {
        let store = create_test_store(None).await;
        let weak = Arc::downgrade(&store);
        drop(store);

        evictor(weak, 1024 * 1024, Some(Duration::from_millis(10)), false).await;
    }

    #[tokio::test]
    async fn compactor_exits_when_store_dropped() {
        let store = create_test_store(None).await;
        let weak = Arc::downgrade(&store);
        drop(store);

        compactor(weak, 1024, Some(Duration::from_millis(10)), false).await;
    }

    #[tokio::test]
    async fn evict_bucket() {
        use tokio::sync::RwLock;

        use crate::local::immutable_store::ImmutableData;
        use crate::local::immutable_store::ImmutableStoreBucket;
        use crate::local::immutable_store::ImmutableStoreEntry;

        let mut bucket = ImmutableStoreBucket::default();

        bucket.entry.push(ImmutableStoreEntry {
            address: rand::random::<crate::Address>(),
            partition: rand::random::<crate::Partition>(),
            data: ImmutableData {
                flags: 0,
                size_payload: 100,
                size_content: 100,
                pack_offset: 0,
                pack_file: 0,
                last_access: 100,
            },
        });
        bucket.entry.push(ImmutableStoreEntry {
            address: rand::random::<crate::Address>(),
            partition: rand::random::<crate::Partition>(),
            data: ImmutableData {
                flags: 0,
                size_payload: 101,
                size_content: 101,
                pack_offset: 0,
                pack_file: 0,
                last_access: 101,
            },
        });
        bucket.entry.push(ImmutableStoreEntry {
            address: rand::random::<crate::Address>(),
            partition: rand::random::<crate::Partition>(),
            data: ImmutableData {
                flags: 0,
                size_payload: 99,
                size_content: 99,
                pack_offset: 0,
                pack_file: 0,
                last_access: 99,
            },
        });
        bucket.entry.push(ImmutableStoreEntry {
            address: rand::random::<crate::Address>(),
            partition: rand::random::<crate::Partition>(),
            data: ImmutableData {
                flags: 0,
                size_payload: 500,
                size_content: 500,
                pack_offset: 0,
                pack_file: 0,
                last_access: 500,
            },
        });
        bucket.entry.push(ImmutableStoreEntry {
            address: rand::random::<crate::Address>(),
            partition: rand::random::<crate::Partition>(),
            data: ImmutableData {
                flags: 0,
                size_payload: 100,
                size_content: 100,
                pack_offset: 0,
                pack_file: 0,
                last_access: 100,
            },
        });
        bucket.entry.push(ImmutableStoreEntry {
            address: rand::random::<crate::Address>(),
            partition: rand::random::<crate::Partition>(),
            data: ImmutableData {
                flags: 0,
                size_payload: 1000,
                size_content: 1000,
                pack_offset: 0,
                pack_file: 0,
                last_access: 1000,
            },
        });

        // Sorting not important for eviction test, it can be invalid order
        bucket.sorted_index.push(1);
        bucket.sorted_index.push(4);
        bucket.sorted_index.push(0);
        bucket.sorted_index.push(3);
        bucket.sorted_index.push(5);
        bucket.sorted_index.push(2);

        let bucket = Arc::new(RwLock::new(bucket));
        let dirty = std::sync::atomic::AtomicBool::new(false);

        let evict_count = LocalImmutableStore::evict_oldest_bucket(bucket.clone(), &dirty, 3).await;

        assert_eq!(evict_count, 3);

        let bucket = bucket.read().await;
        for entry in bucket.entry.iter() {
            assert!(entry.data.last_access > 100);
            // We marked the entries to be the same last access as size, make sure data was preserved
            assert_eq!(entry.data.last_access, entry.data.size_payload as u64);
        }
    }

    #[tokio::test]
    async fn compact_bucket() {
        use std::sync::OnceLock;

        use bytes::Bytes;
        use tokio::task::JoinSet;

        use crate::local::immutable_store::BUCKET_COUNT;
        use crate::local::immutable_store::ImmutableData;
        use crate::local::immutable_store::ImmutableStoreEntry;
        use crate::local::immutable_store::ImmutableStoreGroup;
        use crate::packstore::PackStore;

        let tempdir = generate_tempdir();
        let group = Arc::new(ImmutableStoreGroup {
            bucket: [const { OnceLock::new() }; BUCKET_COUNT],
            dirty: std::array::from_fn(|_| std::sync::atomic::AtomicBool::new(false)),
            bucket_count: std::sync::atomic::AtomicUsize::new(
                crate::local::fan_out::FAN_OUT_LEVEL_MAX,
            ),
            serialize_version: std::sync::atomic::AtomicU32::new(
                crate::local::immutable_store::ImmutableStoreVersion::LazyFanOut as u32,
            ),
            fan_out_threshold: crate::local::fan_out::FAN_OUT_THRESHOLD_DEFAULT,
            committed_level: std::sync::atomic::AtomicUsize::new(
                crate::local::fan_out::FAN_OUT_LEVEL_MAX,
            ),
            packstore: PackStore::new(Some(tempdir.to_path_buf()), 1),
            flush: tokio::sync::Mutex::new(JoinSet::new()),
        });

        // Buffer lengths are primes to ensure test actually verify the correct thing
        let first_buffer = Bytes::copy_from_slice(&[0, 1, 2, 3, 4, 5, 6]);
        let second_buffer = Bytes::copy_from_slice(&[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
        let third_buffer = Bytes::copy_from_slice(&[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12]);

        let first_hash = crate::Hash::hash_buffer(first_buffer.as_ref());
        let second_hash = crate::Hash::hash_buffer(second_buffer.as_ref());
        let third_hash = crate::Hash::hash_buffer(third_buffer.as_ref());

        let mut hashed = [
            (first_hash, first_buffer),
            (second_hash, second_buffer),
            (third_hash, third_buffer),
        ];
        hashed.sort_by_key(|a| a.0);

        let (smaller_hash, smaller_buffer) = hashed[0].clone();
        let (mid_hash, mid_buffer) = hashed[1].clone();
        let (greater_hash, greater_buffer) = hashed[2].clone();

        {
            let smaller_packdata = group
                .packstore
                .store(smaller_buffer.clone())
                .await
                .expect("Failed to store packdata");

            let mid_packdata = group
                .packstore
                .store(mid_buffer.clone())
                .await
                .expect("Failed to store packdata");

            let greater_packdata = group
                .packstore
                .store(greater_buffer.clone())
                .await
                .expect("Failed to store packdata");

            let mut bucket = group.bucket(0).write().await;

            let mut mid_context: crate::Context = rand::random();
            let mut mid_repository: crate::Partition = rand::random();

            // Ensure some order
            mid_context.data_mut()[0] = 1;
            mid_repository.data_mut()[0] = 1;

            let mut smaller_context = mid_context;
            smaller_context.data_mut()[0] = 0;

            let mut smaller_repository = mid_repository;
            smaller_repository.data_mut()[0] = 0;

            let mut greater_context = mid_context;
            greater_context.data_mut()[0] = 2;

            let mut greater_repository = mid_repository;
            greater_repository.data_mut()[0] = 2;

            // index 0, sort order 2, Deduplicated, should be compacted to same packfile as previous
            bucket.entry.push(ImmutableStoreEntry {
                address: crate::Address {
                    hash: mid_hash,
                    context: mid_context,
                },
                partition: greater_repository,
                data: ImmutableData {
                    flags: 0,
                    size_payload: mid_buffer.len() as u32,
                    size_content: mid_buffer.len() as u64,
                    pack_offset: mid_packdata.offset,
                    pack_file: mid_packdata.id,
                    last_access: 0,
                },
            });

            // index 1, sort order 4, Should be compacted to a new packfile
            bucket.entry.push(ImmutableStoreEntry {
                address: crate::Address {
                    hash: greater_hash,
                    context: greater_context,
                },
                partition: greater_repository,
                data: ImmutableData {
                    flags: 0,
                    size_payload: greater_buffer.len() as u32,
                    size_content: greater_buffer.len() as u64,
                    pack_offset: greater_packdata.offset,
                    pack_file: greater_packdata.id,
                    last_access: 0,
                },
            });

            // index 2, sort order 0, This should remain due to other packfile
            bucket.entry.push(ImmutableStoreEntry {
                address: crate::Address {
                    hash: smaller_hash,
                    context: mid_context,
                },
                partition: mid_repository,
                data: ImmutableData {
                    flags: 0,
                    size_payload: smaller_buffer.len() as u32,
                    size_content: smaller_buffer.len() as u64,
                    pack_offset: smaller_packdata.offset,
                    pack_file: smaller_packdata.id + 1,
                    last_access: 0,
                },
            });

            // index 3, sort order 1, This should be compacted to new packfile
            bucket.entry.push(ImmutableStoreEntry {
                address: crate::Address {
                    hash: mid_hash,
                    context: smaller_context,
                },
                partition: smaller_repository,
                data: ImmutableData {
                    flags: 0,
                    size_payload: mid_buffer.len() as u32,
                    size_content: mid_buffer.len() as u64,
                    pack_offset: mid_packdata.offset,
                    pack_file: mid_packdata.id,
                    last_access: 0,
                },
            });

            // index 4, sort order 3, This should remain, different packfile
            bucket.entry.push(ImmutableStoreEntry {
                address: crate::Address {
                    hash: greater_hash,
                    context: mid_context,
                },
                partition: mid_repository,
                data: ImmutableData {
                    flags: 0,
                    size_payload: greater_buffer.len() as u32,
                    size_content: greater_buffer.len() as u64,
                    pack_offset: greater_packdata.offset,
                    pack_file: greater_packdata.id + 2,
                    last_access: 0,
                },
            });

            bucket.sorted_index.push(2);
            bucket.sorted_index.push(3);
            bucket.sorted_index.push(0);
            bucket.sorted_index.push(4);
            bucket.sorted_index.push(1);
        }

        group
            .packstore
            .stop_write(1)
            .await
            .expect("Failed to stop write");

        let compacted_size =
            LocalImmutableStore::compact_bucket_packfile_impl(&group, 0, 0, 1, false).await;

        // Two instances of the data should have been rewritten to new packfiles
        assert_eq!(compacted_size, mid_buffer.len() + greater_buffer.len());
    }
}
