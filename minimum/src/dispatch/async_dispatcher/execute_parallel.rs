type ChildFuture<ErrorT> = dyn futures::future::Future<Item = (), Error = ErrorT> + Send;

// Given a list of futures, executes all futures in parallel. The result (whether success or failure)
// is ignored. This task should always succeed.
//
// This is different from tokio's join. It is actually prone to cause deadlocks for the way this
// crate tries to use it since the futures don't get dropped when they complete. (So any locks they
// were holding don't get released when the future is done.)
pub struct ExecuteParallel<ErrorT: Send + 'static> {
    state: ExecuteParallelState<ErrorT>,
}

enum ExecuteParallelState<ErrorT: Send + 'static> {
    NotStarted(Vec<Box<ChildFuture<ErrorT>>>),
    Started(Vec<tokio::sync::oneshot::Receiver<Result<(), ErrorT>>>),
    Finished,
}

impl<ErrorT: Send + 'static> ExecuteParallel<ErrorT> {
    pub fn new(futures: Vec<Box<ChildFuture<ErrorT>>>) -> Self {
        ExecuteParallel {
            state: ExecuteParallelState::NotStarted(futures),
        }
    }
}

impl<ErrorT: Send + 'static> futures::future::Future for ExecuteParallel<ErrorT> {
    type Item = ();
    type Error = ErrorT;

    fn poll(&mut self) -> futures::Poll<Self::Item, Self::Error> {
        loop {
            match &mut self.state {
                ExecuteParallelState::NotStarted(futures) => {
                    let futures = std::mem::replace(futures, Vec::new());
                    let mut receivers = Vec::with_capacity(futures.len());

                    // For each future, create a oneshot that will be triggered when that future completes
                    for future in futures {
                        let (tx, rx) = tokio::sync::oneshot::channel();

                        let future = future.then(|result| {
                            // Ignore the result, we don't care if the "owner" future was dropped (this
                            // could be considered a cancellation)
                            let _ = tx.send(result);
                            Ok(())
                        });

                        tokio::spawn(future);
                        receivers.push(rx);
                    }

                    self.state = ExecuteParallelState::Started(receivers)
                }
                ExecuteParallelState::Started(rx_list) => {
                    // Walk through all receivers until we find one that isn't complete. If all are complete, then
                    // return that this task is complete
                    loop {
                        match rx_list.last_mut() {
                            None => {
                                self.state = ExecuteParallelState::Finished;
                                return Ok(futures::Async::Ready(()));
                            }
                            Some(rx) => match rx.poll() {
                                Err(_) => {
                                    panic!("A task has been dropped without first sending a result")
                                }
                                Ok(futures::Async::NotReady) => {
                                    return Ok(futures::Async::NotReady)
                                }
                                Ok(_) => {
                                    rx_list.pop().unwrap();
                                }
                            },
                        }
                    }
                }
                ExecuteParallelState::Finished => unreachable!(),
            }
        }
    }
}
