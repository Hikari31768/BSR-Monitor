use std::{future::Future, sync::Arc};
use tokio::task::JoinSet;

pub const PLAYER_CONCURRENCY: usize = 5;
pub const PLAYLIST_CONCURRENCY: usize = 5;
pub const MAP_INFO_CONCURRENCY: usize = 5;

#[derive(Default, Debug, PartialEq)]
pub struct BatchSummary {
    pub completed: usize,
    pub failed: usize,
}

/// Start a bounded number of jobs and replenish a slot whenever any job ends.
/// An individual error/panic counts as a failure, without stopping the queue.
pub async fn run_bounded<T, F, Fut, Report>(
    items: Vec<T>, limit: usize, work: F, mut report: Report,
) -> BatchSummary
where
    T: Send + 'static,
    F: Fn(T) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Result<String, String>> + Send + 'static,
    Report: FnMut(&BatchSummary, &Result<String, String>),
{
    let work = Arc::new(work);
    let mut items = items.into_iter();
    let mut jobs = JoinSet::new();
    for item in items.by_ref().take(limit.max(1)) {
        let work = work.clone();
        jobs.spawn(async move { work(item).await });
    }
    let mut summary = BatchSummary::default();
    while let Some(result) = jobs.join_next().await {
        let result = result.unwrap_or_else(|e| Err(format!("刷新任务异常: {e}")));
        summary.completed += 1;
        summary.failed += usize::from(result.is_err());
        report(&summary, &result);
        if let Some(item) = items.next() {
            let work = work.clone();
            jobs.spawn(async move { work(item).await });
        }
    }
    summary
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use tokio::sync::Barrier;

    #[tokio::test]
    async fn two_groups_overlap_and_each_respects_its_own_limit() {
        let barrier = Arc::new(Barrier::new(6));
        let make_work = |barrier: Arc<Barrier>| {
            let active = Arc::new(AtomicUsize::new(0));
            move |index: usize| {
                let (active, barrier) = (active.clone(), barrier.clone());
                async move {
                    let count = active.fetch_add(1, Ordering::SeqCst) + 1;
                    assert!(count <= 3);
                    // Both groups must supply three active jobs before any can finish.
                    if index < 3 { barrier.wait().await; }
                    tokio::task::yield_now().await;
                    active.fetch_sub(1, Ordering::SeqCst);
                    Ok(index.to_string())
                }
            }
        };
        let (players, playlists) = tokio::time::timeout(std::time::Duration::from_secs(3), async {
            tokio::join!(
                run_bounded((0..8).collect(), 3, make_work(barrier.clone()), |_, _| {}),
                run_bounded((0..7).collect(), 3, make_work(barrier), |_, _| {}),
            )
        }).await.expect("Both groups should run concurrently");
        assert_eq!(players, BatchSummary { completed: 8, failed: 0 });
        assert_eq!(playlists, BatchSummary { completed: 7, failed: 0 });
    }

    #[tokio::test]
    async fn errors_do_not_stop_other_jobs_and_progress_counts_completions() {
        let mut progress = Vec::new();
        let summary = run_bounded((0..7).collect(), 3, |index| async move {
            if index == 1 { return Err("offline".into()); }
            if index == 2 { panic!("simulated worker failure"); }
            Ok(index.to_string())
        }, |summary, _| progress.push(summary.completed)).await;
        assert_eq!(summary, BatchSummary { completed: 7, failed: 2 });
        assert_eq!(progress, (1..=7).collect::<Vec<_>>());
    }

    #[tokio::test]
    async fn empty_group_does_not_run_work() {
        let summary = run_bounded(Vec::<usize>::new(), 3,
            |_| async { panic!("empty group must not start work"); }, |_, _| {}).await;
        assert_eq!(summary, BatchSummary::default());
    }
}
