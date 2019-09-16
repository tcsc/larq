use ::futures::{Async, Future, IntoFuture, Poll};
use std::fmt;
use std::mem;

#[derive(Debug)]
enum TaskState<T>
where
    T: Future,
{
    Pending(T),
    Done(Result<T::Item, T::Error>),
}

#[must_use = "futures do nothing unless polled"]
pub struct GatherAll<I>
where
    I: IntoIterator,
    I::Item: IntoFuture,
{
    tasks: Vec<TaskState<<I::Item as IntoFuture>::Future>>,
}

pub fn gather_all<I>(i: I) -> GatherAll<I>
where
    I: IntoIterator,
    I::Item: IntoFuture,
{
    let tasks = i
        .into_iter()
        .map(|f| TaskState::Pending(f.into_future()))
        .collect();
    GatherAll { tasks }
}

// impl<I> fmt::Debug for GatherAll<I>
//     where I: IntoIterator,
//           I::Item: IntoFuture,
//           <<I as IntoIterator>::Item as IntoFuture>::Future: fmt::Debug,
//           <<I as IntoIterator>::Item as IntoFuture>::Item: fmt::Debug,
// {
//     fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
//         fmt.debug_struct("GatherAll")
//             .field("tasks", &self.tasks)
//             .finish()
//     }
// }

impl<I> Future for GatherAll<I>
where
    I: IntoIterator,
    I::Item: IntoFuture,
{
    type Item = Vec<Result<<I::Item as IntoFuture>::Item, <I::Item as IntoFuture>::Error>>;
    type Error = <I::Item as IntoFuture>::Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        let mut all_done = true;
        for item in self.tasks.iter_mut() {
            if let TaskState::Pending(ref mut f) = item {
                match f.poll() {
                    Ok(Async::Ready(v)) => *item = TaskState::Done(Ok(v)),
                    Ok(Async::NotReady) => {
                        all_done = false;
                    }
                    Err(e) => *item = TaskState::Done(Err(e)),
                }
            }
        }

        if all_done {
            let tasks = mem::replace(&mut self.tasks, Vec::new());
            let results = tasks
                .into_iter()
                .map(|x| match x {
                    TaskState::Done(result) => result,
                    _ => unreachable!(),
                })
                .collect();
            Ok(Async::Ready(results))
        } else {
            Ok(Async::NotReady)
        }
    }
}
