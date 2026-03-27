// Integration test: ThemeManager → fs-inventory
//
// Verifies that installing a Store theme writes the correct record to
// fs-inventory so the Inventory remains the single source of truth for
// what is installed on this node.

use fs_inventory::{Inventory, ResourceStatus};
use fs_manager_theme::ThemeManager;
use fs_types::ResourceType;

#[tokio::test]
async fn theme_install_recorded_in_inventory() {
    let inv = Inventory::open(":memory:").await.expect("open inventory");

    let mgr = ThemeManager::with_noop();
    mgr.install_from_store(&inv, "midnight-blue", "1.0.0")
        .await
        .expect("install_from_store failed");

    let resource = inv
        .resource("midnight-blue")
        .await
        .expect("inventory query failed")
        .expect("midnight-blue not found in inventory");

    assert_eq!(resource.id, "midnight-blue");
    assert_eq!(resource.version, "1.0.0");
    assert_eq!(resource.resource_type, ResourceType::ColorScheme);
    assert_eq!(resource.status, ResourceStatus::Active);
}

#[tokio::test]
async fn theme_install_idempotent() {
    let inv = Inventory::open(":memory:").await.expect("open inventory");
    let mgr = ThemeManager::with_noop();

    // First install
    mgr.install_from_store(&inv, "nordic-dark", "1.0.0")
        .await
        .expect("first install failed");

    // Second call (same id) should not fail — upsert semantics
    mgr.install_from_store(&inv, "nordic-dark", "1.0.1")
        .await
        .expect("second install failed");

    // The resource must still exist
    let resource = inv
        .resource("nordic-dark")
        .await
        .expect("inventory query failed")
        .expect("nordic-dark not found");

    assert_eq!(resource.id, "nordic-dark");
}
