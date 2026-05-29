// SPDX-FileCopyrightText: 2026 Epic Games, Inc.
// SPDX-License-Identifier: MIT
#[cfg(test)]
mod test {
    use std::time::Duration;

    use lore_revision::util::inflight::InflightOutput;
    use lore_revision::util::inflight::RequestRole;

    #[test]
    fn first_request_returns_request_maker() {
        let inflight = InflightOutput::<&str, String>::default();
        assert!(matches!(
            inflight.request("a"),
            RequestRole::RequestMaker(_)
        ));
    }

    #[test]
    fn concurrent_request_returns_result_awaiter() {
        let inflight = InflightOutput::<&str, String>::default();
        let _guard = match inflight.request("a") {
            RequestRole::RequestMaker(g) => g,
            RequestRole::ResultAwaiter(_) => panic!("expected RequestMaker"),
        };
        assert!(matches!(
            inflight.request("a"),
            RequestRole::ResultAwaiter(_)
        ));
    }

    #[tokio::test]
    async fn broadcast_delivers_to_awaiter() {
        let inflight = InflightOutput::<&str, String>::default();

        let guard = match inflight.request("a") {
            RequestRole::RequestMaker(g) => g,
            RequestRole::ResultAwaiter(_) => panic!("expected RequestMaker"),
        };
        let mut receiver = match inflight.request("a") {
            RequestRole::ResultAwaiter(r) => r,
            RequestRole::RequestMaker(_) => panic!("expected ResultAwaiter"),
        };

        guard.broadcast(&"hello".to_string());
        assert_eq!(receiver.recv().await.unwrap(), "hello");
    }

    #[test]
    fn broadcast_removes_entry_so_next_request_is_maker() {
        let inflight = InflightOutput::<&str, String>::default();

        let guard = match inflight.request("a") {
            RequestRole::RequestMaker(g) => g,
            RequestRole::ResultAwaiter(_) => panic!("expected RequestMaker"),
        };
        guard.broadcast(&"done".to_string());

        // The entry was removed — next request for the same key must be a fresh RequestMaker
        assert!(matches!(
            inflight.request("a"),
            RequestRole::RequestMaker(_)
        ));
    }

    #[test]
    fn drop_without_broadcast_removes_entry() {
        let inflight = InflightOutput::<&str, String>::default();

        let guard = match inflight.request("a") {
            RequestRole::RequestMaker(g) => g,
            RequestRole::ResultAwaiter(_) => panic!("expected RequestMaker"),
        };
        drop(guard);

        // Entry cleaned up on drop — next caller retries as RequestMaker
        assert!(matches!(
            inflight.request("a"),
            RequestRole::RequestMaker(_)
        ));
    }

    #[tokio::test]
    async fn sequential_request_after_broadcast_does_not_hang() {
        let inflight = InflightOutput::<&str, String>::default();

        // First request completes normally
        let guard = match inflight.request("a") {
            RequestRole::RequestMaker(g) => g,
            RequestRole::ResultAwaiter(_) => panic!("expected RequestMaker"),
        };
        guard.broadcast(&"first".to_string());

        // Second request for the same key should get a fresh RequestMaker (not hang)
        let result = tokio::time::timeout(Duration::from_millis(100), async {
            match inflight.request("a") {
                RequestRole::RequestMaker(g) => {
                    g.broadcast(&"second".to_string());
                    "second".to_string()
                }
                RequestRole::ResultAwaiter(mut r) => r.recv().await.unwrap(),
            }
        })
        .await;

        assert_eq!(result.unwrap(), "second");
    }

    #[test]
    fn distinct_keys_are_independent() {
        let inflight = InflightOutput::<&str, String>::default();
        let _guard_a = match inflight.request("a") {
            RequestRole::RequestMaker(g) => g,
            RequestRole::ResultAwaiter(_) => panic!("expected RequestMaker for a"),
        };
        assert!(matches!(
            inflight.request("b"),
            RequestRole::RequestMaker(_)
        ));
    }

    #[tokio::test]
    async fn drop_without_broadcast_wakes_awaiter_with_error() {
        let inflight = InflightOutput::<&str, String>::default();

        let guard = match inflight.request("a") {
            RequestRole::RequestMaker(g) => g,
            RequestRole::ResultAwaiter(_) => panic!("expected RequestMaker"),
        };
        let mut receiver = match inflight.request("a") {
            RequestRole::ResultAwaiter(r) => r,
            RequestRole::RequestMaker(_) => panic!("expected ResultAwaiter"),
        };

        // Drop the guard without broadcasting — sender is dropped, receiver gets Closed
        drop(guard);
        assert!(receiver.recv().await.is_err());
    }
}
