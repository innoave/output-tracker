#[allow(clippy::wildcard_imports)]
use assertor::*;
use output_tracker::non_threadsafe::{Error, OutputSubject, OutputTracker};
use thiserror as _;

struct Adapter {
    output_subject: OutputSubject<Message>,
}

impl Adapter {
    fn new() -> Self {
        Self {
            output_subject: OutputSubject::new(),
        }
    }

    fn track_messages(&self) -> Result<OutputTracker<Message>, Error> {
        self.output_subject.create_tracker()
    }

    fn send_message(&self, message: Message) {
        // do some I/O
        println!("sending message: '{} - {}'", message.topic, message.content);

        // track that message was sent
        // we ignore errors from the tracker here as it is not important for the business logic.
        _ = self.output_subject.emit(message);
    }
}

#[derive(Debug, Clone, PartialEq)]
struct Message {
    topic: String,
    content: String,
}

#[allow(clippy::unwrap_used)]
fn main() {
    let adapter = Adapter::new();

    let tracker = adapter.track_messages().unwrap();

    adapter.send_message(Message {
        topic: "weather report".to_string(),
        content: "it will be snowing tomorrow".to_string(),
    });

    adapter.send_message(Message {
        topic: "no shadow".to_string(),
        content: "keep your face to the sunshine and you cannot see a shadow".to_string(),
    });

    let tracker_output = tracker.output().unwrap();

    println!("\nTracked messages:");
    for message in &tracker_output {
        println!("|-> {}: {}", message.topic, message.content);
    }

    assert_that!(tracker_output).contains_exactly_in_order(vec![
        Message {
            topic: "weather report".to_string(),
            content: "it will be snowing tomorrow".to_string(),
        },
        Message {
            topic: "no shadow".to_string(),
            content: "keep your face to the sunshine and you cannot see a shadow".to_string(),
        },
    ]);
}

// workaround for false positive 'unused extern crate' warnings until
// Rust issue [#95513](https://github.com/rust-lang/rust/issues/95513) is fixed
mod dummy_extern_uses {
    use proptest as _;
    use version_sync as _;
}
