use std::collections::VecDeque;
use std::sync::{Condvar, Mutex};
pub struct MpmcQueue<T> {
    inner_: Mutex<VecDeque<T>>,
    cv_: Condvar,
}

impl<T> MpmcQueue<T> {
    pub fn new() -> Self {
        MpmcQueue {
            inner_: Mutex::new(VecDeque::<T>::new()),
            cv_: Condvar::new(),
        }
    }

    pub fn enqueue(&self, p_data: T) {
        let mut inner = self.inner_.lock().unwrap();
        inner.push_back(p_data);
        self.cv_.notify_one();
    }

    pub fn dequeue(&self) -> Option<T> {
        let mut inner = self.inner_.lock().unwrap();
        while inner.is_empty() {
            inner = self.cv_.wait(inner).unwrap();
        }
        inner.pop_front()
    }
}

#[cfg(test)]
mod tests {
    use self::MpmcQueue;
    use super::*;
    use std::sync::Arc;

    #[test]
    fn basic_test() {
        let queue = Arc::new(MpmcQueue::new());

        let producer = std::thread::spawn({
            println!("Producer thread started!");
            let queue = Arc::clone(&queue);
            println!("Producer thread took the copy of queue!");
            move || {
                println!("Producer thread trying to send 0 to 10!");
                for i in 0..100 {
                    println!("Producer thread trying to send {i}");
                    queue.enqueue(i);
                    println!("enqueued {i}");
                    std::thread::sleep(std::time::Duration::from_millis(1));
                }
            }
        });

        let consumer = std::thread::spawn({
            let queue = Arc::clone(&queue);
            move || loop {
                if let Some(value) = queue.dequeue() {
                    println!("Popped {}", value);
                } else {
                    break;
                }
            }
        });

        producer.join().unwrap();
        consumer.join().unwrap();
    }
}
