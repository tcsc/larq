use std::{
    cmp::{Ordering, Eq, Ord, PartialEq, PartialOrd},
    collections::BinaryHeap
};

use futures::{
    stream::{FuturesUnordered, StreamExt},
    Future,
    TryFuture,
};

struct TaggedResult<T>{
    tag: usize,
    result: T
}

impl<T> PartialEq for TaggedResult<T> {
    fn eq(&self, other: &Self) -> bool {
        self.tag == other.tag
    }
}

impl<T> Eq for TaggedResult<T> {}

impl<T> PartialOrd for TaggedResult<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<T> Ord for TaggedResult<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        // BinaryHeap is a max heap, so compare backwards here.
        other.tag.cmp(&self.tag)
    }
}

async fn tag_task<F>(tag: usize, task: F) -> Result<TaggedResult<<F as TryFuture>::Ok>, <F as TryFuture>::Error> 
where
    F: TryFuture + Future<Output=Result<<F as TryFuture>::Ok, <F as TryFuture>::Error>>
{
    match task.await {
        Ok(x) => Ok(TaggedResult{tag, result: x}),
        Err(e) => Err(e)
    }
}

pub async fn try_join_all<I>(n: usize, i: I) -> 
    Result<
        Vec<<<I as IntoIterator>::Item as TryFuture>::Ok>, 
        <<I as IntoIterator>::Item as TryFuture>::Error
    >
where
    I: IntoIterator,
    I::Item: TryFuture + Future<
        Output=Result<
            <<I as IntoIterator>::Item as TryFuture>::Ok, 
            <<I as IntoIterator>::Item as TryFuture>::Error
        >
    >
{
    // Wrap the supplied task iterator so that it produces tasks tagged 
    // with the original task index. This allows us to re-order the 
    // un-ordered task results as they come in so we can preserve the
    // input ordering in the output buffer.
    let mut tasks = i.into_iter().enumerate().map(|(i, t)| tag_task(i, t));
    let mut workers = FuturesUnordered::new();
    let mut results = Vec::new();
    let mut ordered_results = BinaryHeap::new();
    let mut next_output = 0;

    // Prime the worker pool with tasks up to the prescribed throttle limit.
    while let Some(t) = tasks.next() {
        workers.push(t);
        if workers.len() == n {
            break;
        } 
    }

    loop {
        match workers.next().await {
            Some(Ok(r)) => {
                // We want to force the ordering of the results to match the
                // ordering of the input jobs, so we buffer the un-ordered 
                // results that come out of the worker stream in a heap until
                // we know we can emit them in sequence.
                ordered_results.push(r);

                while let Some(tr) = ordered_results.peek() {
                    if tr.tag != next_output {
                        break;
                    }

                    let tr = ordered_results.pop().unwrap();
                    results.push(tr.result);
                    next_output += 1;
                }

                // Replace the finished task with the next ofn off the queue 
                if let Some(t) = tasks.next() {
                    workers.push(t);
                } 
            }, 
            Some(Err(e)) => {
                return Err(e)
            },
            None => {
                break
            }
        }
    }

    Ok(results)
}


#[cfg(test)]
mod test {
    use std::sync::{Arc, Mutex};
    use tokio::time::{sleep, Duration};
    use super::*;

    #[derive(Debug)]
    struct StateData {
        current: isize,
        count: isize,
        max: isize,
    }

    async fn random_wait(n: isize, state: Arc<Mutex<StateData>>) -> Result<isize, isize> {
        let mut s = state.lock().unwrap();
        s.current += 1;
        s.count += 1;
        s.max = std::cmp::max(s.current, s.max);
        drop(s);

        // delay 
        let v = rand::random::<f64>();
        sleep(Duration::from_millis((100.0 * v) as u64)).await;

        let mut s = state.lock().unwrap();
        s.current -= 1;
        drop(s);

        if n < 0 {
            Err(-1)
        } else { 
            Ok(n)
        }
    }

    #[tokio::test]
    async fn success() {
        let mut tasks = Vec::with_capacity(100);
        let state = Arc::new(Mutex::new(StateData{current: 0, count: 0, max: 0}));
        for x in 0..100 {
            tasks.push(random_wait(x, state.clone()));
        }

        match try_join_all(5, tasks).await {
            Ok(v) => {
                assert_eq!(v.len(), 100);
                assert_eq!(v, (0..100).collect::<Vec<isize>>());

                let s = state.lock().unwrap();
                assert_eq!(s.current, 0);
                assert!(s.max <= 5, "Max concurrent should be <= 5: {:?}", s);
            },
            Err(e) => {
                assert!(false, "Expected try_join_all to succeed: {}", e);
            }
        }
    }

    #[tokio::test]
    async fn fewer_jobs_than_throttle() {
        let mut tasks = Vec::with_capacity(5);
        let state = Arc::new(Mutex::new(StateData{current: 0, count: 0, max: 0}));
        for x in 0..tasks.capacity() {
            tasks.push(random_wait(x as isize, state.clone()));
        }

        match try_join_all(100, tasks).await {
            Ok(v) => {
                assert_eq!(v.len(), 5);
                assert_eq!(v, (0..5).collect::<Vec<isize>>());
            },
            Err(e) => {
                assert!(false, "Expected try_join_all to succeed: {}", e);
            }
        }
    }

    #[tokio::test]
    async fn failure() {
        let mut tasks = Vec::with_capacity(100);
        let state = Arc::new(Mutex::new(StateData{current: 0, count: 0, max: 0}));
        for x in 0..100 {
            let y = if x == 10 { -100 } else { x };
            tasks.push(random_wait(y, state.clone()));
        }

        match try_join_all(5, tasks).await {
            Ok(_) => assert!(false, "Expected try_join_all() to fail"),
            Err(e) => {
                assert_eq!(e, -1);

                let s = state.lock().unwrap();
                assert!(s.count < 50, "Max concurrent should be <= 5: {:?}", s);
            }
        }
    }
}