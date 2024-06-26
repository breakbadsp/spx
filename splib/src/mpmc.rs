use std::{
    collections::VecDeque,
    sync::{Arc, Condvar, Mutex},
};

struct Inner<T> {
    queue: VecDeque<T>,
    senders: usize,
}

struct SharedQueue<T> {
    inner: Mutex<Inner<T>>,
    cv: Condvar,
}

pub struct Sender<T> {
    shared: Arc<SharedQueue<T>>,
}

pub struct Receiver<T> {
    shared: Arc<SharedQueue<T>>,
}

impl<T> Sender<T> {
    pub fn enqueue(&self, p_data: T) {
        let lock_result = self.shared.inner.lock();
        match lock_result {
            Err(_) => {
                return; //TODO:: Handle lock result poison error properly
            }

            Ok(mut guarded_queue) => {
                guarded_queue.queue.push_back(p_data);
                drop(guarded_queue);
                self.shared.cv.notify_one();
            }
        }
    }
}

impl<T> Clone for Sender<T> {
    fn clone(&self) -> Self {
        let mut gurded_queue = self.shared.inner.lock().unwrap(); //TODO:: Handle lock result PoisonError properly
        gurded_queue.senders += 1;
        drop(gurded_queue);
        Sender {
            shared: self.shared.clone(),
        }
    }
}

impl<T> Drop for Sender<T> {
    fn drop(&mut self) {
        let mut gurded_queue = self.shared.inner.lock().unwrap(); //TODO:: Handle lock result PoisonError properly
        gurded_queue.senders -= 1;
        if gurded_queue.senders == 0 {
            self.shared.cv.notify_one()
        }
    }
}

impl<T> Receiver<T> {
    pub fn dequeue(&mut self) -> Option<T> {
        let mut guarded_queue = self.shared.inner.lock().unwrap(); //TODO:: Handle lock result PoisonError properly
        while guarded_queue.queue.is_empty() && guarded_queue.senders > 0 {
            guarded_queue = self.shared.cv.wait(guarded_queue).unwrap(); //TODO:: Handle lock result PoisonError properly
        }
        let data = guarded_queue.queue.pop_front();
        data
    }
}

impl<T> Clone for Receiver<T> {
  fn clone(&self) -> Self {
      Receiver {
          shared: self.shared.clone(),
      }
  }
}

pub fn channel<T>() -> (Sender<T>, Receiver<T>) {
    let inner = Inner {
        queue: VecDeque::default(),
        senders: 1,
    };

    let queue = SharedQueue {
        inner: Mutex::new(inner),
        cv: Condvar::new(),
    };

    let shared_queue = Arc::new(queue);

    let tx = Sender {
        shared: shared_queue.clone(),
    };

    let rc = Receiver {
        shared: shared_queue.clone(),
    };

    (tx, rc)
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::time;

    #[test]
    fn spsc_test() {
        let (tx, mut rc) = channel();

        let producer = std::thread::spawn({
            let tx_copy = tx.clone();
            println!("Producer started ");
            move || {
                for i in 0..100 {
                    println!("Sending {i}");
                    tx_copy.enqueue(i);
                }
            }
        });

        let consumer = std::thread::spawn({
            println!("Consumer started!");
            move || loop {
                let data_or_none = rc.dequeue();
                match data_or_none {
                    None => return,
                    Some(data) => {
                        println!("Received data {data}");
                    }
                }
            }
        });
        drop(tx);
        producer.join().unwrap();
        consumer.join().unwrap();
    }

    #[test]
    fn mpsc_test() {
        let (tx, mut rc) = channel();

        let mut nums = vec![];
        let consumer = std::thread::spawn({
            println!("Consumer started : ");
            move || {
                loop {
                    let data_or_none = rc.dequeue();
                    match data_or_none {
                        None => {
                            println!("Producers are done sending, Terminating constumer");
                            break;
                        }
                        Some(data) => {
                            println!("Received data: {data}");
                            nums.push(data);
                        }
                    }
                }
                assert_eq!(nums.len(), 100);
            }
        });

        let mut producers = vec![];
        for i in 0..100 {
            let producer = std::thread::spawn({
                let tx_copy = tx.clone();
                println!("Producer started : ");
                move || {
                    println!("Sending {i}");
                    tx_copy.enqueue(i);
                    std::thread::sleep(time::Duration::from_millis(1));
                }
            });
            producers.push(producer);
        }

        for prod in producers {
            prod.join().unwrap();
        }
        drop(tx);
        consumer.join().unwrap();
    }

    #[test]
    fn ping_pong() {
        let (tx, mut rx) = channel();
        tx.enqueue(42);
        assert_eq!(rx.dequeue(), Some(42));
    }

    #[test]
    fn closed_tx() {
        let (tx, mut rx) = channel::<()>();
        drop(tx);
        assert_eq!(rx.dequeue(), None);
    }

    #[test]
    fn closed_rx() {
        let (tx, rx) = channel();
        drop(rx);
        tx.enqueue(42);
    }
}
