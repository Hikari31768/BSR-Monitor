use std::{future::Future, pin::Pin, sync::Arc};
use tokio::task::JoinSet;
use super::{ApiError, BeatLeaderApi, CompactScoreResponseWithMetadata};

type Page = CompactScoreResponseWithMetadata;
type Fetch = Arc<dyn Fn(i32) -> Pin<Box<dyn Future<Output = Result<Page, ApiError>> + Send>> + Send + Sync>;

/// Fetch pages in batches of up to five. Results are yielded as requests finish,
/// so the caller can save each page without waiting for earlier page numbers.
pub struct ScorePager {
    fetch: Fetch,
    pending: JoinSet<(i32, Result<Page, ApiError>)>,
    batch_remaining: usize,
    page: i32,
    total_pages: i32,
    count: i32,
    latest: Option<i64>,
    boundary_in_batch: bool,
    initialized: bool,
    done: bool,
    concurrency: usize,
}

impl ScorePager {
    pub fn new(api: BeatLeaderApi, player_id: String, count: i32, latest: Option<i64>, concurrency: usize) -> Self {
        let mut pager = Self::with_fetch(Arc::new(move |page| {
            let api = api.clone();
            let player_id = player_id.clone();
            Box::pin(async move {
                let started = std::time::Instant::now();
                let result = api.get_player_scores(&player_id, page, count).await;
                eprintln!("Score page {page}: request={:?}", started.elapsed());
                result
            })
        }), count, latest);
        pager.concurrency = concurrency.clamp(1, crate::settings::MAX_CONCURRENCY);
        pager
    }

    fn with_fetch(fetch: Fetch, count: i32, latest: Option<i64>) -> Self {
        Self {
            fetch,
            pending: JoinSet::new(),
            batch_remaining: 0,
            page: 1,
            total_pages: 0,
            count,
            latest,
            boundary_in_batch: false,
            initialized: false,
            done: false,
            concurrency: crate::settings::MAX_CONCURRENCY,
        }
    }

    pub async fn next_page(&mut self) -> Result<Option<(i32, Page)>, ApiError> {
        if self.done {
            return Ok(None);
        }

        // Fetch page 1 first so the API's metadata tells us how many pages exist.
        if !self.initialized {
            self.initialized = true;
            let response = (self.fetch)(1).await?;
            let page_size = if response.metadata.items_per_page > 0 {
                response.metadata.items_per_page
            } else {
                self.count.max(1)
            };
            self.total_pages = response.metadata.total.saturating_add(page_size - 1) / page_size;
            self.page = 2;
            self.done = response.data.is_empty()
                || 1 >= self.total_pages
                || self.is_incremental_boundary(&response);
            if !self.done {
                self.spawn_batch();
            }
            return Ok(Some((1, response)));
        }

        if self.batch_remaining == 0 {
            if self.boundary_in_batch || self.page > self.total_pages {
                self.done = true;
                return Ok(None);
            }
            self.spawn_batch();
        }

        let joined = self.pending.join_next().await
            .ok_or_else(|| ApiError::JsonParse("Score prefetch ended unexpectedly".into()))?;
        self.batch_remaining -= 1;
        let (page, response) = match joined {
            Ok((page, Ok(response))) => (page, response),
            Ok((_, Err(error))) => {
                self.done = true;
                self.pending.abort_all();
                return Err(error);
            }
            Err(error) => {
                self.done = true;
                self.pending.abort_all();
                return Err(ApiError::JsonParse(format!("Score prefetch failed: {error}")));
            }
        };

        if self.is_incremental_boundary(&response) {
            self.boundary_in_batch = true;
        }
        Ok(Some((page, response)))
    }

    fn spawn_batch(&mut self) {
        self.boundary_in_batch = false;
        while self.batch_remaining < self.concurrency && self.page <= self.total_pages {
            let page = self.page;
            self.page += 1;
            let fetch = self.fetch.clone();
            self.pending.spawn(async move { (page, fetch(page).await) });
            self.batch_remaining += 1;
        }
    }

    fn is_incremental_boundary(&self, response: &Page) -> bool {
        response.data.is_empty() || self.latest.is_some_and(|latest| {
            response.data.iter().any(|score| score.score.epoch_time <= latest)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    fn page(number: i32, epoch: i64, total: i32) -> Page {
        serde_json::from_value(serde_json::json!({
            "metadata": {"page": number, "itemsPerPage": 1, "total": total},
            "data": [{"score": {"id": number, "accuracy": 0.9, "epochTime": epoch},
                      "leaderboard": {"id": "abc71"}}]
        })).unwrap()
    }

    #[tokio::test]
    async fn configured_limit_bounds_requests_and_allows_out_of_order_results() {
        let calls = Arc::new(AtomicUsize::new(0));
        let gates = Arc::new((0..9).map(|_| tokio::sync::Notify::new()).collect::<Vec<_>>());
        let (worker_calls, worker_gates) = (calls.clone(), gates.clone());
        let mut pager = ScorePager::with_fetch(Arc::new(move |n| {
            let (calls, gates) = (worker_calls.clone(), worker_gates.clone());
            Box::pin(async move {
                calls.fetch_add(1, Ordering::SeqCst);
                if n != 1 { gates[n as usize].notified().await; }
                Ok(page(n, 20, 8))
            })
        }), 1, None);
        pager.concurrency = 2;
        assert_eq!(pager.next_page().await.unwrap().unwrap().0, 1);
        tokio::task::yield_now().await;
        assert_eq!(calls.load(Ordering::SeqCst), 3);
        gates[3].notify_one();
        assert_eq!(pager.next_page().await.unwrap().unwrap().0, 3);
        assert_eq!(calls.load(Ordering::SeqCst), 3);
        gates[2].notify_one();
        assert_eq!(pager.next_page().await.unwrap().unwrap().0, 2);
        gates[4].notify_one();
        assert_eq!(pager.next_page().await.unwrap().unwrap().0, 4);
        tokio::task::yield_now().await;
        assert_eq!(calls.load(Ordering::SeqCst), 5);
    }

    #[tokio::test]
    async fn fetches_pages_concurrently_and_stops_after_incremental_boundary_batch() {
        let calls = Arc::new(AtomicUsize::new(0));
        let count = calls.clone();
        let mut pager = ScorePager::with_fetch(Arc::new(move |n| {
            count.fetch_add(1, Ordering::SeqCst);
            Box::pin(async move { Ok(page(n, if n == 1 { 20 } else { 10 }, 3)) })
        }), 1, Some(10));
        assert_eq!(pager.next_page().await.unwrap().unwrap().0, 1);
        tokio::task::yield_now().await;
        assert_eq!(calls.load(Ordering::SeqCst), 3);
        let mut returned = vec![pager.next_page().await.unwrap().unwrap().0,
            pager.next_page().await.unwrap().unwrap().0];
        returned.sort_unstable();
        assert_eq!(returned, vec![2, 3]);
        assert!(pager.next_page().await.unwrap().is_none());
        assert_eq!(calls.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn full_sync_stops_at_last_page_and_propagates_errors() {
        let mut pager = ScorePager::with_fetch(Arc::new(|n| Box::pin(async move {
            Ok(page(n, 20, 2))
        })), 1, None);
        assert_eq!(pager.next_page().await.unwrap().unwrap().0, 1);
        assert_eq!(pager.next_page().await.unwrap().unwrap().0, 2);
        assert!(pager.next_page().await.unwrap().is_none());

        let mut failed = ScorePager::with_fetch(Arc::new(|n| Box::pin(async move {
            if n == 1 { Ok(page(1, 20, 2)) } else { Err(ApiError::Timeout) }
        })), 1, None);
        assert!(failed.next_page().await.is_ok());
        assert!(matches!(failed.next_page().await, Err(ApiError::Timeout)));
        assert!(failed.next_page().await.unwrap().is_none());
    }

    #[tokio::test]
    async fn dropping_pager_cancels_inflight_prefetch() {
        struct Guard(Arc<AtomicUsize>);
        impl Drop for Guard { fn drop(&mut self) { self.0.fetch_add(1, Ordering::SeqCst); } }
        let dropped = Arc::new(AtomicUsize::new(0));
        let started = Arc::new(AtomicUsize::new(0));
        let (drop_count, start_count) = (dropped.clone(), started.clone());
        let mut pager = ScorePager::with_fetch(Arc::new(move |n| {
            let (drop_count, start_count) = (drop_count.clone(), start_count.clone());
            Box::pin(async move {
                if n == 1 { return Ok(page(1, 20, 3)); }
                let _guard = Guard(drop_count);
                start_count.fetch_add(1, Ordering::SeqCst);
                std::future::pending::<Result<Page, ApiError>>().await
            })
        }), 1, None);
        pager.next_page().await.unwrap();
        tokio::task::yield_now().await;
        assert_eq!(started.load(Ordering::SeqCst), 2);
        drop(pager);
        tokio::task::yield_now().await;
        assert_eq!(dropped.load(Ordering::SeqCst), 2);
    }
}
