#![cfg(feature = "mock-honcho")]
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    deprecated,
    clippy::print_stderr
)]

#[path = "../src/actor/mock.rs"]
mod mock;

use honcho_ai::Honcho;
use mock::{HonchoLike, MockHoncho};

fn get_test_client() -> Option<MockHoncho> {
    let url = std::env::var("TAICHO_HONCHO_URL").ok()?;
    let workspace = std::env::var("TAICHO_HONCHO_WORKSPACE").unwrap_or_else(|_| "test".to_owned());
    let inner = Honcho::new(&url, &workspace).expect("constructor ok");
    Some(MockHoncho::new(inner))
}

#[test]
#[ignore = "requires TAICHO_HONCHO_URL env var pointing to a real Honcho server"]
fn integration_list_peers() {
    let Some(mock) = get_test_client() else {
        eprintln!("skipping integration test: TAICHO_HONCHO_URL not set");
        return;
    };
    let client = mock.raw();

    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let result = client
            .peers_with_filters(std::collections::HashMap::new(), 1, 10, false)
            .await;
        match result {
            Ok(page) => {
                eprintln!("fetched {} peers", page.raw_items().len());
            }
            Err(e) => {
                eprintln!("list_peers failed: {:?}", e);
                panic!("list_peers should succeed against real server");
            }
        }
    });
}

#[test]
#[ignore = "requires TAICHO_HONCHO_URL env var pointing to a real Honcho server"]
fn integration_ensure_connection() {
    let Some(mock) = get_test_client() else {
        eprintln!("skipping integration test: TAICHO_HONCHO_URL not set");
        return;
    };
    let client = mock.raw();

    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let result = client.force_ensure().await;
        assert!(
            result.is_ok(),
            "force_ensure should succeed against real server"
        );
        eprintln!("workspace_id: {}", client.workspace_id());
    });
}
