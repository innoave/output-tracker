# Output-Tracker

[![crates.io][crates-badge]][crates-url]
[![docs.rs][docs-badge]][docs-url]
[![Apache-2.0 licensed][license-badge]][license-url]
![MSRV][msrv-badge]
[![code coverage][code-coverage-badge]][code-coverage-url]

Output-Tracker is a utility for writing state-based tests with [nullables] instead of mocks. It can
track the state of dependencies which can then be asserted in the test.

> Output-Tracker is created after the great article
> ["Testing without Mocks" by James Shore][output-tracking]
> and his concept of [nullables].

Architectural patterns like Ports & Adapters, Hexagonal Architecture or A-Frame Architecture also
help with easier testing of dependencies which are expensive to set up and/or lead to slow tests.
In software that is designed following such architectural patterns can use Output-Tracker to track
actions done by an outbound adapter which update the state. The sequence of actions can be asserted
in a test.

Although the motivation for using an output-tracker is mainly testability, it can also be used
in the production code for recording messages and state changes in a log.

## Usage

Add output-tracker as a dependency to the `Cargo.toml` file of your project:

```toml
[dependencies]
output-tracker = "0.1"
```

Making use of the output-tracker comprises the following steps:

1. Equip an outbound adapter with an `OutputSubject`
2. Create an `ObjectTracker` in the test
3. Assert tracked messages/state changes for completeness and order (if appropriate)

```rust
use output_tracker::non_threadsafe::{Error, OutputSubject, OutputTracker};

//
// Production code
//

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

//
// Test
//
use assertor::*;

// this is a test method in a test module
// main() method is used here in the example so that it is compiled and run during doc-tests
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
```

<!-- Badges and related URLs -->

[crates-badge]: https://img.shields.io/crates/v/output-tracker.svg

[crates-url]: https://crates.io/crates/output-tracker

[docs-badge]: https://docs.rs/output-tracker/badge.svg

[docs-url]: https://docs.rs/output-tracker

[license-badge]: https://img.shields.io/github/license/innoave/output-tracker?color=blue

[license-url]: https://github.com/innoave/output-tracker/blob/main/LICENSE

[msrv-badge]: https://img.shields.io/crates/msrv/output-tracker?color=chocolate

[code-coverage-badge]: https://codecov.io/github/innoave/output-tracker/graph/badge.svg?token=o0w7R7J0Op

[code-coverage-url]: https://codecov.io/github/innoave/output-tracker


<!-- External Links -->

[nullables]: https://www.jamesshore.com/v2/projects/nullables

[output-tracking]: https://www.jamesshore.com/v2/projects/nullables/testing-without-mocks#output-tracking

