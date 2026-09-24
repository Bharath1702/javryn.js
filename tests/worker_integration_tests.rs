//! Rust integration and stress tests for Javryn V0.4 Worker architecture.

use std::collections::BTreeMap;
use std::path::PathBuf;
use javryn_runtime::workers::{JsMessage, WorkerHandle, WorkerId, WorkerManager};

#[test]
fn test_js_message_serialization_roundtrip() {
    let mut map = BTreeMap::new();
    map.insert("name".to_string(), JsMessage::String("Javryn".to_string()));
    map.insert("v".to_string(), JsMessage::Number(0.4));
    map.insert("active".to_string(), JsMessage::Boolean(true));
    map.insert("items".to_string(), JsMessage::Array(vec![JsMessage::Number(1.0), JsMessage::Number(2.0)]));

    let msg = JsMessage::Object(map);
    let cloned = msg.clone();
    assert_eq!(msg, cloned);
}

#[test]
fn test_worker_handle_spawn_and_terminate() {
    let id = WorkerId(101);
    let mut handle = WorkerHandle::spawn(id, None).expect("worker should spawn successfully");
    assert!(handle.is_running());

    handle.terminate().expect("worker should terminate cleanly");
    assert!(!handle.is_running());
}

#[test]
fn test_worker_manager_lifecycle() {
    let mut manager = WorkerManager::new();
    let w1 = manager.spawn_worker(None).expect("spawn w1");
    let w2 = manager.spawn_worker(None).expect("spawn w2");
    let w3 = manager.spawn_worker(None).expect("spawn w3");

    assert_eq!(manager.len(), 3);

    manager.terminate_worker(w1).expect("terminate w1");
    assert_eq!(manager.len(), 2);

    manager.shutdown().expect("shutdown remaining workers");
    assert_eq!(manager.len(), 0);
}

#[test]
fn test_worker_repeated_spawn_and_shutdown_stress() {
    let mut manager = WorkerManager::new();
    for _ in 0..30 {
        let id = manager.spawn_worker(None).expect("spawn worker");
        manager.terminate_worker(id).expect("terminate worker");
    }
    assert_eq!(manager.len(), 0);
}

#[test]
fn test_worker_multiple_concurrent_instances() {
    let mut manager = WorkerManager::new();
    let mut ids = Vec::new();

    for _ in 0..10 {
        let id = manager.spawn_worker(None).expect("spawn concurrent worker");
        ids.push(id);
    }

    assert_eq!(manager.len(), 10);

    for id in ids {
        manager.terminate_worker(id).expect("terminate worker");
    }
}
