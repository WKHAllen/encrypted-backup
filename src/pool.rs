//! A synchronous task pool implementation.

#![allow(dead_code)]

use std::sync::mpsc::{sync_channel, SyncSender};
use std::thread::{spawn, JoinHandle};

/// Type alias for a heap-allocated thread-safe synchronous task.
type Task = Box<dyn FnOnce() + Send>;

/// A task request for a worker.
enum TaskRequest {
    /// A task needs to be run.
    Run(Task),
    /// Exit the task running context.
    Exit,
}

/// A message signifying that a worker is ready.
enum TaskResponse {
    /// The worker is ready.
    Ready(usize),
}

/// A worker that can perform tasks.
struct Worker {
    /// The join handle for the worker.
    handle: JoinHandle<()>,
    /// The channel through which task requests can be sent.
    task_sender: SyncSender<TaskRequest>,
}

impl Worker {
    /// Creates a new worker.
    pub fn new(id: usize, ready_sender: SyncSender<TaskResponse>) -> Self {
        let (task_sender, task_receiver) = sync_channel(1);

        let handle = spawn(move || {
            ready_sender.send(TaskResponse::Ready(id)).unwrap();

            while let Ok(req) = task_receiver.recv() {
                match req {
                    TaskRequest::Run(job) => {
                        job();
                    }
                    TaskRequest::Exit => {
                        break;
                    }
                }

                ready_sender.send(TaskResponse::Ready(id)).unwrap();
            }
        });

        Self {
            handle,
            task_sender,
        }
    }

    /// Signals the worker to run the given task.
    pub fn run(&self, task: Task) {
        self.task_sender.send(TaskRequest::Run(task)).unwrap();
    }

    /// Sends the exit request and waits for the worker to finish.
    pub fn finish(self) {
        self.task_sender.send(TaskRequest::Exit).unwrap();
        self.handle.join().unwrap();
    }
}

/// A task pool.
#[allow(clippy::module_name_repetitions)]
pub struct TaskPool {
    /// The handle to the background task handling task requests.
    handle: JoinHandle<()>,
    /// The channel through which task requests can be sent.
    task_sender: SyncSender<TaskRequest>,
}

impl TaskPool {
    /// Creates a new task pool with the given number of tasks.
    pub fn new(size: usize) -> Self {
        let (task_sender, task_receiver) = sync_channel(size);
        let (ready_sender, ready_receiver) = sync_channel(size);

        let handle = spawn(move || {
            let workers = (0..size)
                .map(|id| Worker::new(id, ready_sender.clone()))
                .collect::<Vec<_>>();

            while let Ok(req) = task_receiver.recv() {
                match req {
                    TaskRequest::Run(task) => {
                        let res = ready_receiver.recv().unwrap();

                        match res {
                            TaskResponse::Ready(worker_id) => {
                                workers.get(worker_id).unwrap().run(task);
                            }
                        }
                    }
                    TaskRequest::Exit => {
                        break;
                    }
                }
            }

            for worker in workers {
                worker.finish();
            }
        });

        Self {
            handle,
            task_sender,
        }
    }

    /// Queues a task to be executed.
    pub fn queue<F>(&self, task: F)
    where
        F: FnOnce() + Send + 'static,
    {
        self.task_sender
            .send(TaskRequest::Run(Box::new(task)))
            .unwrap();
    }

    /// Instructs all tasks to finish and awaits their collective completion.
    pub fn finish(self) {
        self.task_sender.send(TaskRequest::Exit).unwrap();
        self.handle.join().unwrap();
    }
}

/// Task pool tests.
#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::mpsc::{channel, Sender};
    use std::thread::sleep;
    use std::time::Duration;

    /// Tests the task pool.
    #[test]
    fn test_task_pool() {
        #[allow(clippy::needless_pass_by_value)]
        fn send_with_delay<T>(sender: Sender<T>, message: T, secs: f64) {
            sleep(Duration::from_secs_f64(secs));
            sender.send(message).unwrap();
        }

        let pool = TaskPool::new(3);
        let (sender, receiver) = channel::<usize>();

        pool.queue({
            let sender = sender.clone();
            move || send_with_delay(sender, 1, 0.5)
        }); // 0.0 - 0.5
        pool.queue({
            let sender = sender.clone();
            move || send_with_delay(sender, 2, 0.3)
        }); // 0.0 - 0.3
        pool.queue({
            let sender = sender.clone();
            move || send_with_delay(sender, 3, 0.4)
        }); // 0.0 - 0.4
        pool.queue({
            let sender = sender.clone();
            move || send_with_delay(sender, 4, 0.3)
        }); // 0.3 - 0.6
        pool.queue({
            let sender = sender.clone();
            move || send_with_delay(sender, 5, 0.0)
        }); // 0.4 - 0.4
        pool.queue({
            let sender = sender.clone();
            move || send_with_delay(sender, 6, 0.4)
        }); // 0.4 - 0.8
        pool.queue({
            let sender = sender.clone();
            move || send_with_delay(sender, 7, 0.2)
        }); // 0.5 - 0.7
        pool.finish();
        drop(sender);

        assert_eq!(receiver.recv(), Ok(2));
        assert_eq!(receiver.recv(), Ok(3));
        assert_eq!(receiver.recv(), Ok(5));
        assert_eq!(receiver.recv(), Ok(1));
        assert_eq!(receiver.recv(), Ok(4));
        assert_eq!(receiver.recv(), Ok(7));
        assert_eq!(receiver.recv(), Ok(6));
        assert!(receiver.recv().is_err());
    }
}
