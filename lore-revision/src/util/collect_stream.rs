use std::future::Future;
use std::pin::pin;

use tokio::sync::mpsc;

/// Drain a streaming producer into a `Vec` for callers that do not benefit
/// from streaming.
///
/// The producer takes an `mpsc::Sender` and emits `Ok(T)` items as it goes,
/// returning `Ok(())` on success or `Err(E)` on failure. `collect_stream`
/// runs the producer concurrently with a receive loop in the same task via
/// `tokio::select!`, so there is no `tokio::spawn` and no `JoinError`. When
/// the producer completes first, the loop drains any remaining buffered
/// items before returning.
///
/// The first error wins: a producer error or an `Err(E)` item short-circuits
/// the loop and propagates up.
pub async fn collect_stream<T, E, Fut>(
    f: impl FnOnce(mpsc::Sender<Result<T, E>>) -> Fut,
) -> Result<Vec<T>, E>
where
    Fut: Future<Output = Result<(), E>>,
{
    let (tx, mut rx) = mpsc::channel(256);
    let mut driver = pin!(f(tx));
    let mut out = Vec::new();
    loop {
        tokio::select! {
            biased;
            item = rx.recv() => if let Some(item) = item {
                out.push(item?);
            } else {
                (&mut driver).await?;
                break;
            },
            result = &mut driver => {
                result?;
                while let Some(item) = rx.recv().await {
                    out.push(item?);
                }
                break;
            }
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn happy_path_emits_in_order() {
        let result: Result<Vec<u32>, ()> = collect_stream(|tx| async move {
            for n in 0..10 {
                tx.send(Ok(n)).await.unwrap();
            }
            Ok(())
        })
        .await;
        assert_eq!(result.unwrap(), (0..10).collect::<Vec<u32>>());
    }

    #[tokio::test]
    async fn empty_producer_yields_empty_vec() {
        let result: Result<Vec<u32>, ()> = collect_stream(|_tx| async move { Ok(()) }).await;
        assert_eq!(result.unwrap(), Vec::<u32>::new());
    }

    #[tokio::test]
    async fn item_error_short_circuits() {
        let result: Result<Vec<u32>, &'static str> = collect_stream(|tx| async move {
            tx.send(Ok(1)).await.unwrap();
            tx.send(Err("nope")).await.unwrap();
            tx.send(Ok(2)).await.unwrap();
            Ok(())
        })
        .await;
        assert_eq!(result, Err("nope"));
    }

    #[tokio::test]
    async fn producer_error_propagates() {
        let result: Result<Vec<u32>, &'static str> = collect_stream(|tx| async move {
            tx.send(Ok(1)).await.unwrap();
            Err("boom")
        })
        .await;
        assert_eq!(result, Err("boom"));
    }

    #[tokio::test]
    async fn drains_after_producer_finishes() {
        // Producer fills the channel and returns; collect_stream must drain
        // the remaining buffered items rather than dropping them.
        let result: Result<Vec<u32>, ()> = collect_stream(|tx| async move {
            for n in 0..50 {
                tx.send(Ok(n)).await.unwrap();
            }
            Ok(())
        })
        .await;
        assert_eq!(result.unwrap(), (0..50).collect::<Vec<u32>>());
    }

    #[tokio::test]
    async fn large_burst_exercises_channel_backpressure() {
        // Producer emits more items than the channel's capacity (256), so
        // `tx.send` must await the receiver between batches. The driver and
        // receiver share a task, so this proves the `tokio::select!` shape
        // makes progress on both branches without deadlock.
        let result: Result<Vec<u32>, ()> = collect_stream(|tx| async move {
            for n in 0..1000 {
                tx.send(Ok(n)).await.unwrap();
            }
            Ok(())
        })
        .await;
        assert_eq!(result.unwrap(), (0..1000).collect::<Vec<u32>>());
    }

    #[tokio::test]
    async fn producer_error_after_dropping_tx_still_yields_error() {
        // Producer drops the sender, yields once so the receive loop observes
        // the channel close, then returns an error. The driver's error must
        // win even though `rx.recv()` returned `None` first.
        let result: Result<Vec<u32>, &'static str> = collect_stream(|tx| async move {
            drop(tx);
            tokio::task::yield_now().await;
            Err("boom")
        })
        .await;
        assert_eq!(result, Err("boom"));
    }

    #[tokio::test]
    async fn producer_error_after_items_still_yields_error() {
        // Producer emits a few items, then errors. The returned `Err`
        // takes priority over partial item accumulation; the caller does
        // not see a partial `Vec`.
        let result: Result<Vec<u32>, &'static str> = collect_stream(|tx| async move {
            tx.send(Ok(1)).await.unwrap();
            tx.send(Ok(2)).await.unwrap();
            Err("boom")
        })
        .await;
        assert_eq!(result, Err("boom"));
    }
}
