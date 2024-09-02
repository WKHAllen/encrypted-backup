//! A synchronous task pool implementation.

use std::sync::mpsc::{sync_channel, Receiver, SendError, SyncSender};
use std::thread::spawn;

/// Type alias for a heap-allocated thread-safe synchronous task.
type Task<T> = Box<dyn FnOnce() -> T + Send>;

/// The sending side of a task channel.
#[derive(Debug, Clone)]
pub struct TaskRequestSender<T>(SyncSender<Task<T>>)
where
    T: Send;

impl<T> TaskRequestSender<T>
where
    T: Send,
{
    /// Sends a task through the channel where it will be executed on a worker
    /// thread.
    pub fn send<F>(&self, task: F) -> Result<(), SendError<Task<T>>>
    where
        F: FnOnce() -> T + Send + 'static,
    {
        self.0.send(Box::new(task))
    }
}

/// The receiving side of a task channel.
#[derive(Debug)]
pub struct TaskResponseReceiver<T>(Receiver<T>)
where
    T: Send;

impl<T> TaskResponseReceiver<T>
where
    T: Send,
{
    /// Receives the next task response from the channel. Task responses are
    /// the return values of each task, and will be received in the same order
    /// in which the task requests were sent. If no tasks are completed, this
    /// will block until a new response is available or all senders have
    /// disconnected. This will return `None` after all senders have dropped
    /// and all responses have been received.
    pub fn recv(&self) -> Option<T> {
        self.0.recv().ok()
    }
}

/// Creates a task pool of the given size and returns a
/// request sender/response receiver pair. The sender can be used to send
/// synchronous tasks to workers in the pool. The receiver can get the return
/// values of each task. The return values will be received in the same order
/// in which the task requests were sent.
///
/// Note that the pool will continue to exist until one or both halves of the
/// task channel have disconnected.
///
/// This will panic if `size` is 0.
pub fn task_channel<T>(size: usize) -> (TaskRequestSender<T>, TaskResponseReceiver<T>)
where
    T: Send + 'static,
{
    assert!(size > 0);

    let (request_sender, request_receiver) = sync_channel(size);
    let (response_sender, response_receiver) = sync_channel(size);

    let (worker_request_senders, worker_request_receivers): (Vec<_>, Vec<_>) =
        (0..size).map(|_| sync_channel::<Task<T>>(0)).unzip();
    let (worker_response_senders, worker_response_receivers): (Vec<_>, Vec<_>) =
        (0..size).map(|_| sync_channel::<T>(0)).unzip();
    let worker_channels = worker_request_receivers
        .into_iter()
        .zip(worker_response_senders);

    spawn(move || {
        let mut request_index = 0;

        while let Ok(request) = request_receiver.recv() {
            if worker_request_senders[request_index].send(request).is_err() {
                break;
            }

            request_index = (request_index + 1) % size;
        }
    });

    for (worker_request_receiver, worker_response_sender) in worker_channels {
        spawn(move || {
            while let Ok(request) = worker_request_receiver.recv() {
                let response = request();

                if worker_response_sender.send(response).is_err() {
                    break;
                }
            }
        });
    }

    spawn(move || {
        let mut response_index = 0;

        while let Ok(response) = worker_response_receivers[response_index].recv() {
            if response_sender.send(response).is_err() {
                break;
            }

            response_index = (response_index + 1) % size;
        }
    });

    (
        TaskRequestSender(request_sender),
        TaskResponseReceiver(response_receiver),
    )
}

/// Task pool tests.
#[cfg(test)]
mod tests {
    use super::*;
    use std::thread::sleep;
    use std::time::Duration;

    /// Tests the task pool.
    #[test]
    fn test_task_pool() {
        fn return_with_delay<T>(message: T, secs: f64) -> T {
            sleep(Duration::from_secs_f64(secs));
            message
        }

        let (request_sender, response_receiver) = task_channel(3);

        spawn(move || {
            request_sender
                .send(move || return_with_delay(1, 0.5))
                .unwrap();
            request_sender
                .send(move || return_with_delay(2, 0.3))
                .unwrap();
            request_sender
                .send(move || return_with_delay(3, 0.4))
                .unwrap();
            request_sender
                .send(move || return_with_delay(4, 0.3))
                .unwrap();
            request_sender
                .send(move || return_with_delay(5, 0.0))
                .unwrap();
            request_sender
                .send(move || return_with_delay(6, 0.4))
                .unwrap();
            request_sender
                .send(move || return_with_delay(7, 0.2))
                .unwrap();
        });

        assert_eq!(response_receiver.recv(), Some(1));
        assert_eq!(response_receiver.recv(), Some(2));
        assert_eq!(response_receiver.recv(), Some(3));
        assert_eq!(response_receiver.recv(), Some(4));
        assert_eq!(response_receiver.recv(), Some(5));
        assert_eq!(response_receiver.recv(), Some(6));
        assert_eq!(response_receiver.recv(), Some(7));
        assert_eq!(response_receiver.recv(), None);
    }
}
