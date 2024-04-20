mod mpmc_queue;
mod mpmc;
mod mpsc;

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use crate::mpmc_queue::MpmcQueue;

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
            move || {
                loop {
                    if let Some(value) = queue.dequeue() {
                        println!("Popped {}", value);
                    } else {
                        break;
                    }
                }
            }
        });
        
        producer.join().unwrap();
        consumer.join().unwrap();
    }
}