use tokio::sync::mpsc::{UnboundedSender, UnboundedReceiver};
use tokio::prelude::*;
use crate::network::Error;

pub struct TwoWayChannel<T> {
    sender: UnboundedSender<T>,
    receiver: UnboundedReceiver<T>,
}

pub fn channel<T>() -> (TwoWayChannel<T>, TwoWayChannel<T>) {
    let (sender_in_here, receiver_in_there) = tokio::sync::mpsc::unbounded_channel::<T>();
    let (sender_in_there, receiver_in_here) = tokio::sync::mpsc::unbounded_channel::<T>();

    let here = TwoWayChannel::new(sender_in_here, receiver_in_here);
    let there = TwoWayChannel::new(sender_in_there, receiver_in_there);

    (here, there)
}

impl<T> TwoWayChannel<T> {
    pub fn new(sender: UnboundedSender<T>, receiver: UnboundedReceiver<T>) -> TwoWayChannel<T> {
        TwoWayChannel { sender, receiver }
    }
}

impl<T> Sink for TwoWayChannel<T> {
    type SinkItem = T;
    type SinkError = Error;

    fn start_send(
        &mut self,
        item: Self::SinkItem,
    ) -> Result<AsyncSink<Self::SinkItem>, Self::SinkError> {
        self.sender
            .start_send(item)
            .map_err(|e| Self::SinkError::from(e))
    }

    fn poll_complete(&mut self) -> Result<Async<()>, Self::SinkError> {
        self.sender
            .poll_complete()
            .map_err(|e| Self::SinkError::from(e))
    }

    fn close(&mut self) -> Result<Async<()>, Self::SinkError> {
        self.sender.close().map_err(|e| Self::SinkError::from(e))
    }
}

impl<T> Stream for TwoWayChannel<T> {
    type Item = T;
    type Error = Error;

    fn poll(&mut self) -> Result<Async<Option<Self::Item>>, Self::Error> {
        self.receiver.poll().map_err(|e| Self::Error::from(e))
    }
}