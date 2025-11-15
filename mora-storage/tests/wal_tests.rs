use mora_core::{result::MoraError, traits::storage::Storage};
use mora_storage::wal_file_storage::WalFileStorage;
use std::sync::Mutex;
use tempfile::TempDir;

// Global lock to serialize environment variable access across tests
static ENV_LOCK: Mutex<()> = Mutex::new(());

/// Helper to create a test storage instance with a temporary directory
fn create_test_storage() -> (WalFileStorage, TempDir) {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let storage = create_storage_at_path(temp_dir.path().to_str().unwrap());
    (storage, temp_dir)
}

/// Helper to create storage with specific path
/// Uses a global lock to make environment variable access thread-safe
fn create_storage_at_path(path: &str) -> WalFileStorage {
    // Lock to prevent concurrent env var modifications
    let _lock = ENV_LOCK.lock().unwrap();

    // Temporarily override for this call only
    let original = std::env::var("MORA_WAL_PATH").ok();
    std::env::set_var("MORA_WAL_PATH", path);
    let storage = WalFileStorage::load().expect("Failed to load storage");

    // Restore original
    match original {
        Some(val) => std::env::set_var("MORA_WAL_PATH", val),
        None => std::env::remove_var("MORA_WAL_PATH"),
    }

    storage
}

// =============================================================================
// Container Operations Tests
// =============================================================================

#[test]
fn test_create_container_success() {
    let (mut storage, _temp) = create_test_storage();

    let result = storage.create_container(&"test_container".to_string());
    assert!(result.is_ok());

    // Verify container appears in list
    let containers = storage.list_containers().unwrap();
    assert_eq!(containers.len(), 1);
    assert!(containers.contains(&"test_container".to_string()));
}

#[test]
fn test_create_container_already_exists() {
    let (mut storage, _temp) = create_test_storage();

    storage
        .create_container(&"test_container".to_string())
        .unwrap();
    let result = storage.create_container(&"test_container".to_string());

    assert!(result.is_err());
    match result.unwrap_err() {
        MoraError::StorageError(e) => {
            assert!(e.to_string().contains("already exists"));
        }
        _ => panic!("Expected ContainerAlreadyExists error"),
    }
}

#[test]
fn test_delete_container_success() {
    let (mut storage, temp) = create_test_storage();

    storage
        .create_container(&"test_container".to_string())
        .unwrap();

    let result = storage.delete_container(&"test_container".to_string());
    assert!(result.is_ok());

    // Verify container removed from list
    let containers = storage.list_containers().unwrap();
    assert_eq!(containers.len(), 0);

    // Verify file is deleted
    let wal_path = temp.path().join("test_container.wal");
    assert!(!wal_path.exists());
}

#[test]
fn test_delete_container_not_found() {
    let (mut storage, _temp) = create_test_storage();

    let result = storage.delete_container(&"nonexistent".to_string());

    // Deleting non-existent container should fail
    // Either with "not found" or file system error (No such file or directory)
    assert!(result.is_err());
}

#[test]
fn test_list_containers_empty() {
    let (storage, _temp) = create_test_storage();

    let containers = storage.list_containers().unwrap();
    assert_eq!(containers.len(), 0);
}

#[test]
fn test_list_containers_multiple() {
    let (mut storage, _temp) = create_test_storage();

    storage.create_container(&"container1".to_string()).unwrap();
    storage.create_container(&"container2".to_string()).unwrap();
    storage.create_container(&"container3".to_string()).unwrap();

    let containers = storage.list_containers().unwrap();
    assert_eq!(containers.len(), 3);
    assert!(containers.contains(&"container1".to_string()));
    assert!(containers.contains(&"container2".to_string()));
    assert!(containers.contains(&"container3".to_string()));
}

// =============================================================================
// Item Operations Tests
// =============================================================================

#[test]
fn test_store_item_success() {
    let (mut storage, _temp) = create_test_storage();

    storage
        .create_container(&"test_container".to_string())
        .unwrap();

    let key: u128 = 12345;
    let value = vec![1, 2, 3, 4, 5];

    let result = storage.store_item(&"test_container".to_string(), &key, &value);
    assert!(result.is_ok());

    // Verify item is retrievable
    let items = storage
        .get_all_items(&"test_container".to_string())
        .unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(items.get(&key), Some(&value));
}

#[test]
fn test_store_item_overwrite() {
    let (mut storage, _temp) = create_test_storage();

    storage
        .create_container(&"test_container".to_string())
        .unwrap();

    let key: u128 = 12345;
    let value1 = vec![1, 2, 3];
    let value2 = vec![4, 5, 6, 7];

    storage
        .store_item(&"test_container".to_string(), &key, &value1)
        .unwrap();
    storage
        .store_item(&"test_container".to_string(), &key, &value2)
        .unwrap();

    let items = storage
        .get_all_items(&"test_container".to_string())
        .unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(items.get(&key), Some(&value2));
}

#[test]
fn test_store_item_container_not_found() {
    let (mut storage, _temp) = create_test_storage();

    let key: u128 = 12345;
    let value = vec![1, 2, 3];

    let result = storage.store_item(&"nonexistent".to_string(), &key, &value);

    assert!(result.is_err());
    match result.unwrap_err() {
        MoraError::StorageError(e) => {
            assert!(e.to_string().contains("not found"));
        }
        _ => panic!("Expected ContainerNotFound error"),
    }
}

#[test]
fn test_store_items_batch_success() {
    let (mut storage, _temp) = create_test_storage();

    storage
        .create_container(&"test_container".to_string())
        .unwrap();

    let items = vec![
        (100u128, vec![1, 2]),
        (200u128, vec![3, 4]),
        (300u128, vec![5, 6]),
    ];

    let result = storage.store_items(&"test_container".to_string(), &items);
    assert!(result.is_ok());

    let stored_items = storage
        .get_all_items(&"test_container".to_string())
        .unwrap();
    assert_eq!(stored_items.len(), 3);
    assert_eq!(stored_items.get(&100), Some(&vec![1, 2]));
    assert_eq!(stored_items.get(&200), Some(&vec![3, 4]));
    assert_eq!(stored_items.get(&300), Some(&vec![5, 6]));
}

#[test]
fn test_store_items_batch_empty() {
    let (mut storage, _temp) = create_test_storage();

    storage
        .create_container(&"test_container".to_string())
        .unwrap();

    let items: Vec<(u128, Vec<u8>)> = vec![];
    let result = storage.store_items(&"test_container".to_string(), &items);
    assert!(result.is_ok());

    let stored_items = storage
        .get_all_items(&"test_container".to_string())
        .unwrap();
    assert_eq!(stored_items.len(), 0);
}

#[test]
fn test_delete_item_success() {
    let (mut storage, _temp) = create_test_storage();

    storage
        .create_container(&"test_container".to_string())
        .unwrap();

    let key: u128 = 12345;
    let value = vec![1, 2, 3];

    storage
        .store_item(&"test_container".to_string(), &key, &value)
        .unwrap();

    let result = storage.delete_item(&"test_container".to_string(), &key);
    assert!(result.is_ok());

    let items = storage
        .get_all_items(&"test_container".to_string())
        .unwrap();
    assert_eq!(items.len(), 0);
}

#[test]
fn test_delete_item_not_present() {
    let (mut storage, _temp) = create_test_storage();

    storage
        .create_container(&"test_container".to_string())
        .unwrap();

    let key: u128 = 12345;

    // Deleting non-existent item should succeed (idempotent)
    let result = storage.delete_item(&"test_container".to_string(), &key);
    assert!(result.is_ok());
}

#[test]
fn test_delete_items_batch_success() {
    let (mut storage, _temp) = create_test_storage();

    storage
        .create_container(&"test_container".to_string())
        .unwrap();

    // Store multiple items
    let items = vec![
        (100u128, vec![1, 2]),
        (200u128, vec![3, 4]),
        (300u128, vec![5, 6]),
    ];
    storage
        .store_items(&"test_container".to_string(), &items)
        .unwrap();

    // Delete some of them
    let keys_to_delete = vec![100u128, 300u128];
    let result = storage.delete_items(&"test_container".to_string(), &keys_to_delete);
    assert!(result.is_ok());

    let remaining = storage
        .get_all_items(&"test_container".to_string())
        .unwrap();
    assert_eq!(remaining.len(), 1);
    assert_eq!(remaining.get(&200), Some(&vec![3, 4]));
}

#[test]
fn test_delete_items_batch_empty() {
    let (mut storage, _temp) = create_test_storage();

    storage
        .create_container(&"test_container".to_string())
        .unwrap();

    let keys: Vec<u128> = vec![];
    let result = storage.delete_items(&"test_container".to_string(), &keys);
    assert!(result.is_ok());
}

#[test]
fn test_get_all_items_empty() {
    let (mut storage, _temp) = create_test_storage();

    storage
        .create_container(&"test_container".to_string())
        .unwrap();

    let items = storage
        .get_all_items(&"test_container".to_string())
        .unwrap();
    assert_eq!(items.len(), 0);
}

#[test]
fn test_get_all_items_container_not_found() {
    let (mut storage, _temp) = create_test_storage();

    let result = storage.get_all_items(&"nonexistent".to_string());

    assert!(result.is_err());
    match result.unwrap_err() {
        MoraError::StorageError(e) => {
            assert!(e.to_string().contains("not found"));
        }
        _ => panic!("Expected ContainerNotFound error"),
    }
}

// =============================================================================
// WAL Replay Tests
// =============================================================================

#[test]
fn test_replay_empty_wal() {
    let temp = TempDir::new().unwrap();
    let path = temp.path().to_str().unwrap();

    // Create storage and container
    {
        let mut storage = create_storage_at_path(path);
        storage.create_container(&"test".to_string()).unwrap();
    }

    // Reload storage (replays WAL)
    let mut storage = create_storage_at_path(path);

    let items = storage.get_all_items(&"test".to_string()).unwrap();
    assert_eq!(items.len(), 0);
}

#[test]
fn test_replay_with_items() {
    let temp = TempDir::new().unwrap();
    let path = temp.path().to_str().unwrap();

    // Create storage, add items, drop
    {
        let mut storage = create_storage_at_path(path);
        storage.create_container(&"test".to_string()).unwrap();
        storage
            .store_item(&"test".to_string(), &100, &vec![1, 2, 3])
            .unwrap();
        storage
            .store_item(&"test".to_string(), &200, &vec![4, 5, 6])
            .unwrap();
    }

    // Reload storage (replays WAL)
    let mut storage = create_storage_at_path(path);

    let items = storage.get_all_items(&"test".to_string()).unwrap();
    assert_eq!(items.len(), 2);
    assert_eq!(items.get(&100), Some(&vec![1, 2, 3]));
    assert_eq!(items.get(&200), Some(&vec![4, 5, 6]));
}

#[test]
fn test_replay_with_tombstones() {
    let temp = TempDir::new().unwrap();
    let path = temp.path().to_str().unwrap();

    // Create storage, add items, delete some, drop
    {
        let mut storage = create_storage_at_path(path);
        storage.create_container(&"test".to_string()).unwrap();
        storage
            .store_item(&"test".to_string(), &100, &vec![1, 2, 3])
            .unwrap();
        storage
            .store_item(&"test".to_string(), &200, &vec![4, 5, 6])
            .unwrap();
        storage
            .store_item(&"test".to_string(), &300, &vec![7, 8, 9])
            .unwrap();
        storage.delete_item(&"test".to_string(), &200).unwrap();
    }

    // Reload storage (replays WAL)
    let mut storage = create_storage_at_path(path);

    let items = storage.get_all_items(&"test".to_string()).unwrap();
    assert_eq!(items.len(), 2);
    assert_eq!(items.get(&100), Some(&vec![1, 2, 3]));
    assert_eq!(items.get(&300), Some(&vec![7, 8, 9]));
    assert_eq!(items.get(&200), None); // Deleted
}

#[test]
fn test_replay_with_overwrites() {
    let temp = TempDir::new().unwrap();
    let path = temp.path().to_str().unwrap();

    // Create storage, overwrite items, drop
    {
        let mut storage = create_storage_at_path(path);
        storage.create_container(&"test".to_string()).unwrap();
        storage
            .store_item(&"test".to_string(), &100, &vec![1, 2, 3])
            .unwrap();
        storage
            .store_item(&"test".to_string(), &100, &vec![4, 5, 6, 7])
            .unwrap();
    }

    // Reload storage (replays WAL)
    let mut storage = create_storage_at_path(path);

    let items = storage.get_all_items(&"test".to_string()).unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(items.get(&100), Some(&vec![4, 5, 6, 7]));
}

#[test]
fn test_replay_multiple_containers() {
    let temp = TempDir::new().unwrap();
    let path = temp.path().to_str().unwrap();

    // Create multiple containers with items
    {
        let mut storage = create_storage_at_path(path);
        storage.create_container(&"container1".to_string()).unwrap();
        storage.create_container(&"container2".to_string()).unwrap();

        storage
            .store_item(&"container1".to_string(), &100, &vec![1, 2])
            .unwrap();
        storage
            .store_item(&"container2".to_string(), &200, &vec![3, 4])
            .unwrap();
    }

    // Reload storage
    let mut storage = create_storage_at_path(path);

    let containers = storage.list_containers().unwrap();
    assert_eq!(containers.len(), 2);

    let items1 = storage.get_all_items(&"container1".to_string()).unwrap();
    assert_eq!(items1.len(), 1);
    assert_eq!(items1.get(&100), Some(&vec![1, 2]));

    let items2 = storage.get_all_items(&"container2".to_string()).unwrap();
    assert_eq!(items2.len(), 1);
    assert_eq!(items2.get(&200), Some(&vec![3, 4]));
}

// =============================================================================
// Compaction Tests
// =============================================================================

#[test]
fn test_compaction_basic() {
    let (mut storage, _temp) = create_test_storage();

    storage.create_container(&"test".to_string()).unwrap();

    // Add many items
    for i in 0..100 {
        storage
            .store_item(&"test".to_string(), &i, &vec![i as u8])
            .unwrap();
    }

    // Delete many (create tombstones)
    for i in 0..50 {
        storage.delete_item(&"test".to_string(), &i).unwrap();
    }

    // Manually trigger compaction
    let result = storage.compact_container("test");
    assert!(result.is_ok());

    // Verify data is preserved
    let items = storage.get_all_items(&"test".to_string()).unwrap();
    assert_eq!(items.len(), 50);

    for i in 50..100 {
        assert_eq!(items.get(&i), Some(&vec![i as u8]));
    }
}

#[test]
fn test_compaction_empty_container() {
    let (mut storage, _temp) = create_test_storage();

    storage.create_container(&"test".to_string()).unwrap();

    let result = storage.compact_container("test");
    assert!(result.is_ok());

    let items = storage.get_all_items(&"test".to_string()).unwrap();
    assert_eq!(items.len(), 0);
}

#[test]
fn test_compaction_survives_reload() {
    let temp = TempDir::new().unwrap();
    let path = temp.path().to_str().unwrap();

    // Create, populate, compact
    {
        let mut storage = create_storage_at_path(path);
        storage.create_container(&"test".to_string()).unwrap();

        for i in 0..100 {
            storage
                .store_item(&"test".to_string(), &i, &vec![i as u8])
                .unwrap();
        }

        for i in 0..50 {
            storage.delete_item(&"test".to_string(), &i).unwrap();
        }

        storage.compact_container("test").unwrap();
    }

    // Reload and verify - container already exists from WAL replay
    let mut storage = create_storage_at_path(path);

    // Container should be loaded from WAL
    let containers = storage.list_containers().unwrap();
    assert!(containers.contains(&"test".to_string()));

    let items = storage.get_all_items(&"test".to_string()).unwrap();
    assert_eq!(items.len(), 50);
}

// =============================================================================
// Large Data Tests
// =============================================================================

#[test]
fn test_large_item_storage() {
    let (mut storage, _temp) = create_test_storage();

    storage.create_container(&"test".to_string()).unwrap();

    // Store 1MB item
    let large_data = vec![42u8; 1024 * 1024];
    let result = storage.store_item(&"test".to_string(), &12345, &large_data);
    assert!(result.is_ok());

    let items = storage.get_all_items(&"test".to_string()).unwrap();
    assert_eq!(items.get(&12345), Some(&large_data));
}

#[test]
fn test_zero_sort_key() {
    let (mut storage, _temp) = create_test_storage();

    storage.create_container(&"test".to_string()).unwrap();

    let key: u128 = 0;
    let value = vec![1, 2, 3];

    storage
        .store_item(&"test".to_string(), &key, &value)
        .unwrap();

    let items = storage.get_all_items(&"test".to_string()).unwrap();
    assert_eq!(items.get(&key), Some(&value));
}

#[test]
fn test_max_sort_key() {
    let (mut storage, _temp) = create_test_storage();

    storage.create_container(&"test".to_string()).unwrap();

    let key: u128 = u128::MAX;
    let value = vec![1, 2, 3];

    storage
        .store_item(&"test".to_string(), &key, &value)
        .unwrap();

    let items = storage.get_all_items(&"test".to_string()).unwrap();
    assert_eq!(items.get(&key), Some(&value));
}

#[test]
fn test_empty_item_data() {
    let (mut storage, _temp) = create_test_storage();

    storage.create_container(&"test".to_string()).unwrap();

    let key: u128 = 12345;
    let value: Vec<u8> = vec![];

    storage
        .store_item(&"test".to_string(), &key, &value)
        .unwrap();

    let items = storage.get_all_items(&"test".to_string()).unwrap();
    assert_eq!(items.get(&key), Some(&value));
}

#[test]
fn test_special_container_names() {
    let (mut storage, _temp) = create_test_storage();

    let special_names = vec![
        "container-with-dashes",
        "container_with_underscores",
        "container.with.dots",
        "UPPERCASE",
        "123numeric",
    ];

    for name in &special_names {
        let result = storage.create_container(&name.to_string());
        assert!(result.is_ok(), "Failed to create container: {}", name);
    }

    let containers = storage.list_containers().unwrap();
    assert_eq!(containers.len(), special_names.len());
}

// =============================================================================
// Concurrent Operations (Sequential Simulation)
// =============================================================================

#[test]
fn test_interleaved_operations() {
    let (mut storage, _temp) = create_test_storage();

    storage.create_container(&"test".to_string()).unwrap();

    // Simulate interleaved adds and deletes
    storage
        .store_item(&"test".to_string(), &1, &vec![1])
        .unwrap();
    storage
        .store_item(&"test".to_string(), &2, &vec![2])
        .unwrap();
    storage.delete_item(&"test".to_string(), &1).unwrap();
    storage
        .store_item(&"test".to_string(), &3, &vec![3])
        .unwrap();
    storage.delete_item(&"test".to_string(), &2).unwrap();
    storage
        .store_item(&"test".to_string(), &1, &vec![10])
        .unwrap(); // Re-add key 1

    let items = storage.get_all_items(&"test".to_string()).unwrap();
    assert_eq!(items.len(), 2);
    assert_eq!(items.get(&1), Some(&vec![10]));
    assert_eq!(items.get(&3), Some(&vec![3]));
}

// =============================================================================
// Persistence Tests
// =============================================================================

#[test]
fn test_persistence_across_multiple_sessions() {
    let temp = TempDir::new().unwrap();
    let path = temp.path().to_str().unwrap();

    // Session 1: Create and add data
    {
        let mut storage = create_storage_at_path(path);
        storage.create_container(&"persistent".to_string()).unwrap();
        storage
            .store_item(&"persistent".to_string(), &100, &vec![1, 2, 3])
            .unwrap();
    }

    // Session 2: Add more data
    {
        let mut storage = create_storage_at_path(path);
        storage
            .store_item(&"persistent".to_string(), &200, &vec![4, 5, 6])
            .unwrap();
    }

    // Session 3: Verify all data persisted
    {
        let mut storage = create_storage_at_path(path);
        let items = storage.get_all_items(&"persistent".to_string()).unwrap();
        assert_eq!(items.len(), 2);
        assert_eq!(items.get(&100), Some(&vec![1, 2, 3]));
        assert_eq!(items.get(&200), Some(&vec![4, 5, 6]));
    }
}

#[test]
fn test_batch_operations_performance() {
    let (mut storage, _temp) = create_test_storage();

    storage.create_container(&"test".to_string()).unwrap();

    // Create batch of 1000 items
    let items: Vec<(u128, Vec<u8>)> = (0..1000)
        .map(|i| (i, vec![i as u8, (i >> 8) as u8]))
        .collect();

    // Store batch should be faster than individual stores
    let result = storage.store_items(&"test".to_string(), &items);
    assert!(result.is_ok());

    let stored = storage.get_all_items(&"test".to_string()).unwrap();
    assert_eq!(stored.len(), 1000);
}
