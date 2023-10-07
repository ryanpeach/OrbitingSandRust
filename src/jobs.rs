use std::collections::VecDeque;
use std::sync::mpsc;
use std::sync::Arc;
use std::sync::{Condvar, Mutex};
use std::thread;
use uuid::Uuid;

type JobId = Uuid;

// Define the structure of a job.
struct Job<T> {
    id: JobId,
    data: T,
    task: Box<dyn FnOnce(&mut T) + Send>,
}

struct Scheduler<T> {
    jobs: Arc<Mutex<VecDeque<Job<T>>>>, // Wrap the Mutex in an Arc.
    cvar: Arc<Condvar>,                 // Wrap the Condvar in an Arc.
}

impl<T> Scheduler<T>
where
    T: Send + 'static,
{
    fn new(num_threads: usize) -> Self {
        let scheduler = Scheduler {
            jobs: Arc::new(Mutex::new(VecDeque::new())),
            cvar: Arc::new(Condvar::new()),
        };

        for _ in 0..num_threads {
            let thread_jobs = scheduler.jobs.clone();
            let thread_cvar = scheduler.cvar.clone();

            thread::spawn(move || loop {
                let mut jobs = thread_jobs.lock().unwrap();

                if let Some(mut job) = jobs.pop_front() {
                    (job.task)(&mut job.data);
                } else {
                    // This replaces the old lock guard with the new one from the wait() call.
                    jobs = thread_cvar.wait(jobs).unwrap();
                }
            });
        }

        scheduler
    }

    fn add_job<F>(&self, data: T, f: F)
    where
        F: FnOnce(&mut T) + Send + 'static,
    {
        let id = Uuid::new_v4();
        let job = Job {
            id,
            data,
            task: Box::new(f),
        };

        let mut jobs = self.jobs.lock().unwrap();
        jobs.push_back(job);
        self.cvar.notify_one(); // Notify worker threads that a new job is available.
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_mutation() {
        let scheduler = Scheduler::new(2);

        let data = 5;

        // Create a channel to communicate between the job and the test.
        let (tx, rx) = mpsc::channel();

        scheduler.add_job(data, move |x| {
            // Note the `move` keyword to capture the tx sender.
            *x *= 2;
            tx.send(*x).unwrap(); // Send the result through the channel.
        });

        // Allow time for job to be processed.
        thread::sleep(std::time::Duration::from_secs(1));

        // Receive the result from the channel.
        let result = rx.recv().unwrap();
        assert_eq!(result, 10, "Data was not mutated as expected.");
    }
}
