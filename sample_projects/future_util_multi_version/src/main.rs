use std::pin::Pin;
use std::task::{Context, Poll};

struct StupidFuture01;

impl futures01::Future for StupidFuture01 {
    type Item = ();
    type Error = String;

    fn poll(&mut self) -> futures01::Poll<Self::Item, Self::Error> {
        Ok(futures01::Async::NotReady)
    }
}

struct StupidFuture;

impl futures::Future for StupidFuture {
    type Output = ();

    fn poll(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Self::Output> {
        Poll::Pending
    }
}

fn main() {
    {
        use futures01::Future;
        let mut future01 = StupidFuture01;
        assert_eq!(future01.poll(), Ok(futures01::Async::NotReady)); 
    }

    let _future = StupidFuture;

    println!("Hello, world!");
}
