// SPDX-FileCopyrightText: 2026 Epic Games, Inc.
// SPDX-License-Identifier: MIT
#[cfg(test)]
mod test {
    use std::time::Duration;
    use std::time::Instant;
    use std::time::SystemTime;

    use lore_base::runtime::LORE_CONTEXT;
    use lore_revision::util::task_queue::TaskQueue;
    use tokio::sync::Semaphore;
    use tokio::task::JoinSet;

    include!("helper.rs");

    type TestResult = Result<(), Box<dyn std::error::Error>>;

    #[tokio::test]
    async fn test_throughput_limit() -> TestResult {
        let execution = setup_test_execution();
        LORE_CONTEXT
            .scope(execution, async move {
                let queue =
                    TaskQueue::new(1, Semaphore::MAX_PERMITS, Semaphore::MAX_PERMITS, vec![]);

                let now = Instant::now();

                let rx1 = queue.submit(Box::pin(async { 0 })).await?;
                let rx2 = queue.submit(Box::pin(async { 1 })).await?;

                assert_eq!(0, rx1.await.unwrap());
                assert_eq!(1, rx2.await.unwrap());

                // This is a super-janky way to test this, but unfortunately there doesn't seem to be
                // any way to use governor's built in `FakeRelativeClock` when we're converting a stream to
                // a rate-limited stream (`StreamRateLimitExt` requires the clock impl to be
                // `ReasonablyRealTime` which `FakeRelativeClock` is not). So... we resort to the fact that
                // we're only allowed to process 1 task per second, so it should have taken more than 1
                // second to process the two tasks we submitted.
                assert!(now.elapsed().as_millis() >= 1000);

                let queue =
                    TaskQueue::new(2, Semaphore::MAX_PERMITS, Semaphore::MAX_PERMITS, vec![]);

                let now = Instant::now();

                // Add a delay to the task execution such that if the tasks were run serially they would
                // exceed the deadline.
                let rx3 = queue
                    .submit(Box::pin(async {
                        tokio::time::sleep(Duration::from_millis(750)).await;
                        2
                    }))
                    .await?;
                let rx4 = queue
                    .submit(Box::pin(async {
                        tokio::time::sleep(Duration::from_millis(750)).await;
                        3
                    }))
                    .await?;

                assert_eq!(2, rx3.await.unwrap());
                assert_eq!(3, rx4.await.unwrap());

                // With a quota of 2 per second, we should now have processed the tasks in less than a
                // second.
                assert!(now.elapsed().as_millis() < 1000);

                Ok(())
            })
            .await
    }

    #[tokio::test]
    async fn test_concurrency_limit() -> TestResult {
        let execution = setup_test_execution();
        LORE_CONTEXT
            .scope(execution, async move {
                let queue = TaskQueue::new(u32::MAX, 1, Semaphore::MAX_PERMITS, vec![]);

                let start = SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)?
                    .as_millis();

                let rx1 = queue
                    .submit(Box::pin(async move {
                        // We need to introduce some delay to ensure the task doesn't complete before the
                        // next one is submitted.
                        tokio::time::sleep(Duration::from_millis(100)).await;
                        SystemTime::now()
                            .duration_since(SystemTime::UNIX_EPOCH)
                            .unwrap()
                            .as_millis()
                            - start
                    }))
                    .await?;
                let rx2 = queue
                    .submit(Box::pin(async move {
                        SystemTime::now()
                            .duration_since(SystemTime::UNIX_EPOCH)
                            .unwrap()
                            .as_millis()
                            - start
                    }))
                    .await?;

                let elapsed = rx1.await.unwrap();
                let elapsed2 = rx2.await.unwrap();

                assert!(elapsed >= 100);
                assert!(elapsed2 >= elapsed);

                Ok(())
            })
            .await
    }

    #[tokio::test]
    async fn test_concurrency_limit_not_exceeded() -> TestResult {
        let execution = setup_test_execution();
        LORE_CONTEXT
            .scope(execution, async move {
                let limit = 10;
                let queue = TaskQueue::new(u32::MAX, limit, Semaphore::MAX_PERMITS, vec![]);

                let mut tasks = JoinSet::new();
                for i in 0..limit {
                    #[allow(clippy::disallowed_methods)]
                    tasks.spawn(
                        queue
                            .submit(Box::pin(async move {
                                tokio::time::sleep(Duration::from_millis(100)).await;
                                i
                            }))
                            .await
                            .unwrap(),
                    );
                }

                while tasks.join_next().await.is_some() {}

                assert_eq!(0, queue.rate_limited_count().await);

                Ok(())
            })
            .await
    }
}
