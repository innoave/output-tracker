//! Basic example on how to use [`OutputSubject`] and [`OutputTracker`] to
//! track data in a component that can be read an asserted to verify if the
//! component does what is expected.
//!
//! This example uses the non-threadsafe variant.
//!
//! This same example but with using the threadsafe variant is available as
//! [`threadsafe_example`]

mod fixture;

#[allow(clippy::wildcard_imports)]
use assertor::*;
use output_tracker::non_threadsafe::{Error, OutputSubject, OutputTracker};
use thiserror as _;

//
// Production code
//

#[derive(Debug, Clone, PartialEq)]
struct Message {
    topic: String,
    content: String,
}

/// Outbound adapter for sending messages.
struct Adapter {
    // equip outbound adapter with output subject
    output_subject: OutputSubject<Message>,
}

impl Adapter {
    fn new() -> Self {
        Self {
            output_subject: OutputSubject::new(),
        }
    }

    /// Create an `OutputTracker` for tracking messages that are sent.
    fn track_messages(&self) -> Result<OutputTracker<Message>, Error> {
        self.output_subject.create_tracker()
    }

    fn send_message(&self, message: Message) {
        // do some I/O for production

        // track that message was sent
        // we ignore errors from the tracker here as it is not important for the business logic.
        _ = self.output_subject.emit(message);
    }
}

//
// Tests
//

#[test]
fn send_message_via_adapter() {
    //
    // Arrange
    //

    let adapter = Adapter::new();

    // activate the output tracker
    let tracker = adapter
        .track_messages()
        .unwrap_or_else(|err| panic!("failed to create message tracker {err}"));

    //
    // Act
    //

    adapter.send_message(Message {
        topic: "weather report".to_string(),
        content: "it will be snowing tomorrow".to_string(),
    });

    adapter.send_message(Message {
        topic: "no shadow".to_string(),
        content: "keep your face to the sunshine and you cannot see a shadow".to_string(),
    });

    //
    // Assert
    //

    // read the output from the output tracker
    let tracker_output = tracker
        .output()
        .unwrap_or_else(|err| panic!("failed to get output from tracker: {err}"));

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
