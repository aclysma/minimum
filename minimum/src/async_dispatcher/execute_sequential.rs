type ChildFuture<ErrorT> = dyn futures::future::Future<Item = (), Error = ErrorT> + Send;

// Executes all given futures in sequence. The result of one is not passed to the other. If any task
// results in an error, we stop executing the futures and return that error
pub struct ExecuteSequential<ErrorT> {
    futures: Vec<Box<ChildFuture<ErrorT>>>,
    next_future_index: usize,
}

impl<ErrorT> ExecuteSequential<ErrorT> {
    pub fn new(futures: Vec<Box<ChildFuture<ErrorT>>>) -> Self {
        ExecuteSequential {
            futures,
            next_future_index: 0,
        }
    }
}

impl<ErrorT> futures::future::Future for ExecuteSequential<ErrorT> {
    type Item = ();
    type Error = ErrorT;

    fn poll(&mut self) -> futures::Poll<Self::Item, Self::Error> {
        loop {
            if self.next_future_index >= self.futures.len() {
                return Ok(futures::Async::Ready(()));
            }

            let result = self.futures[self.next_future_index].poll();
            match result {
                Err(e) => return Err(e),
                Ok(futures::Async::NotReady) => return Ok(futures::Async::NotReady),
                Ok(futures::Async::Ready(_)) => {
                    self.next_future_index += 1;
                }
            }
        }
    }
}
