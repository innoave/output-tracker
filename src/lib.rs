//! A utility for writing state-based tests using [nullables] instead of mocks.
//! It can track the state of dependencies which can then be asserted in a test.
//!
//! In architectural patterns like Ports & Adapters or Hexagonal Architecture
//! code that interacts with the outside world is encapsulated from the domain
//! logic in some adapter or connector. An adapter or connector may implement
//! an interface (or trait) that is interchangeable for different infrastructure
//! or APIs of some third-party component or service.
//!
//! The calling code should not know which implementation is currently used. The
//! instance of an adapter or connector to be used in a certain situation is
//! injected into the calling service (inversion of control). Adapters and
//! connectors are therefore also called dependencies.
//!
//! To test our code we have to set up all dependencies. Setting up the
//! dependencies might be complex and running the tests needs some
//! infrastructure to be set up as well. Often running such tests is slow and
//! the dependencies must be configured separately for different test
//! environments.
//!
//! [Nullables] are a pattern to test as much as possible of our code without
//! actually using the infrastructure. Therefore, testing with nullables is
//! easy to set up and the tests are running fast like unit tests.
//!
//! ## How does it work?
//!
//! We have two main structs, the
//! [`OutputTracker`][non_threadsafe::OutputTracker] and the
//! [`OutputSubject`][non_threadsafe::OutputSubject].
//!
//! An [`OutputTracker`][non_threadsafe::OutputTracker] can track any state of
//! some component or any actions executed by the component.
//! [`OutputTracker`][non_threadsafe::OutputTracker]s can only be created by
//! calling the function [`create_tracker()`][non_threadsafe::OutputSubject::create_tracker]
//! of an [`OutputSubject`][non_threadsafe::OutputSubject].
//!
//! The [`OutputSubject`][non_threadsafe::OutputSubject] holds all
//! [`OutputTracker`][non_threadsafe::OutputTracker] created through its
//! [`create_tracker()`][non_threadsafe::OutputSubject::create_tracker]
//! function. We can emit state or action data to all active
//! [`OutputTracker`][non_threadsafe::OutputTracker]s by calling the function
//! [`emit(data)`][non_threadsafe::OutputSubject::emit] on the
//! [`OutputSubject`][non_threadsafe::OutputSubject].
//!
//! To read and assert the state or action data collected by an
//! [`OutputTracker`][non_threadsafe::OutputTracker] we call the
//! [`output()`][non_threadsafe::OutputTracker::output] function on the
//! [`OutputTracker`][non_threadsafe::OutputTracker].
//!
//! That summarizes the basic usage of [`OutputSubject`][non_threadsafe::OutputSubject]
//! and [`OutputTracker`][non_threadsafe::OutputTracker]. This API is provided
//! in a threadsafe and a non-threadsafe variant. Both variants have the same
//! API. The difference is in the implementation whether the struct can be sent
//! and synced over different threads or not. For details on how to use the two
//! variants see the chapter "Threadsafe and non-threadsafe variants" down
//! below.
//!
//! ## Example
//!
//! Let's assume we have production code that uses an adapter called
//! `MessageSender` to send messages to the outside world.
//!
//! ```no_run
//! struct DomainMessage {
//!     subject: String,
//!     content: String,
//! }
//!
//! #[derive(thiserror::Error, Debug, PartialEq, Eq)]
//! #[error("failed to send message because {message}")]
//! struct Error {
//!     message: String,
//! }
//!
//! struct MessageSender {
//! }
//!
//! impl MessageSender {
//!     fn send_message(&self, message: DomainMessage) -> Result<(), Error> {
//!         unimplemented!("here we are sending the message to the outside world")
//!     }
//! }
//! ```
//!
//! To be able to test this code without using any infrastructure we make the
//! code "nullable". This is done by implementing the lowest possible level
//! that touches the infrastructure for real world usage and in a nulled
//! variant.
//!
//! ```no_run
//! # struct DomainMessage {
//! #     subject: String,
//! #     content: String,
//! # }
//! #
//! # #[derive(thiserror::Error, Debug, PartialEq, Eq)]
//! # #[error("failed to send message because {message}")]
//! # struct Error {
//! #     message: String,
//! # }
//! #
//! # #[derive(thiserror::Error, Debug)]
//! # #[error("some error occurred in the mail api")]
//! # struct ApiError;
//! #
//! //
//! // Production code
//! //
//!
//! #[derive(Debug, Clone, PartialEq, Eq)]
//! struct ApiMessage {
//!     subject: String,
//!     content: String,
//! }
//!
//! struct MessageSender {
//!     mail_api: Box<dyn MailApi>,
//! }
//!
//! impl MessageSender {
//!     // this constructor function is used in production code
//!     fn new() -> Self {
//!         Self {
//!             mail_api: Box::new(RealMail)
//!         }
//!     }
//!
//!     // this constructor function is used in tests using the nullable pattern
//!     fn nulled() -> Self {
//!         Self {
//!             mail_api: Box::new(NulledMail)
//!         }
//!     }
//! }
//!
//! impl MessageSender {
//!     fn send_message(&self, message: DomainMessage) -> Result<(), Error> {
//!         let mail = ApiMessage {
//!             subject: message.subject,
//!             content: message.content,
//!         };
//!
//!         // code before and after this call to the `MailApi` is tested by our tests
//!         let result = self.mail_api.send_mail(mail);
//!
//!         result.map_err(|err| Error { message: err.to_string() })
//!     }
//! }
//!
//! //
//! // Nullability
//! //
//!
//! trait MailApi {
//!     fn send_mail(&self, message: ApiMessage) -> Result<(), ApiError>;
//! }
//!
//! struct RealMail;
//!
//! impl MailApi for RealMail {fn send_mail(&self, message: ApiMessage) -> Result<(), ApiError> {
//!         unimplemented!("implementation is left out for the example as it is not executed in tests using nullables")
//!     }
//! }
//!
//! struct NulledMail;
//!
//! impl MailApi for NulledMail {fn send_mail(&self, message: ApiMessage) -> Result<(), ApiError> {
//!         // nothing to do here in the simplest case
//!         Ok(())
//!     }
//! }
//! ```
//!
//! Now we need some way to assert that the code is actually doing the right
//! things. This is where the output-tracker is used. To do so we equip the
//! `MessageSender` with an `OutputSubject`.
//!
//! ```no_run
//! # struct DomainMessage {
//! #     subject: String,
//! #     content: String,
//! # }
//! #
//! # #[derive(thiserror::Error, Debug, PartialEq, Eq)]
//! # #[error("failed to send message because {message}")]
//! # struct Error {
//! #     message: String,
//! # }
//! #
//! # #[derive(Debug, Clone, PartialEq, Eq)]
//! # struct ApiMessage {
//! #     subject: String,
//! #     content: String,
//! # }
//! #
//! # #[derive(thiserror::Error, Debug)]
//! # #[error("some error occurred in the mail api")]
//! # struct ApiError;
//! #
//! # trait MailApi {
//! #     fn send_mail(&self, message: ApiMessage) -> Result<(), ApiError>;
//! # }
//! #
//! # struct RealMail;
//! #
//! # impl MailApi for RealMail {fn send_mail(&self, message: ApiMessage) -> Result<(), ApiError> {
//! #         unimplemented!("implementation is left out for the example as it is not executed in tests using nullables")
//! #     }
//! # }
//! #
//! # struct NulledMail;
//! #
//! # impl MailApi for NulledMail {fn send_mail(&self, message: ApiMessage) -> Result<(), ApiError> {
//! #         // nothing to do here in the simplest case
//! #         Ok(())
//! #     }
//! # }
//! #
//! use output_tracker::non_threadsafe::{Error as OtError, OutputTracker, OutputSubject};
//!
//! struct MessageSender {
//!     mail_api: Box<dyn MailApi>,
//!     // the output-subject to create output-trackers from
//!     message_subject: OutputSubject<ApiMessage>,
//! }
//!
//! impl MessageSender {
//!     // this constructor function is used in production code
//!     fn new() -> Self {
//!         Self {
//!             mail_api: Box::new(RealMail),
//!             message_subject: OutputSubject::new(),
//!         }
//!     }
//!
//!     // this constructor function is used in tests using the nullable pattern
//!     fn nulled() -> Self {
//!         Self {
//!             mail_api: Box::new(NulledMail),
//!             message_subject: OutputSubject::new(),
//!         }
//!     }
//!
//!     // function to create output-tracker for tracking sent messages
//!     fn track_messages(&self) -> Result<OutputTracker<ApiMessage>, OtError> {
//!         self.message_subject.create_tracker()
//!     }
//!
//!     fn send_message(&self, message: DomainMessage) -> Result<(), Error> {
//!         let mail = ApiMessage {
//!             subject: message.subject,
//!             content: message.content,
//!         };
//!
//!         // code before and after this call to the `MailApi` is tested by our tests
//!         let result = self.mail_api.send_mail(mail.clone());
//!
//!         result.map_err(|err| Error { message: err.to_string() })
//!             // emit sent mail to all active output-trackers
//!             .inspect(|()| _ = self.message_subject.emit(mail))
//!     }
//! }
//! ```
//!
//! Now we can write a test to verify if a domain message is sent via the
//! Mail-API.
//!
//! ```
//! # struct DomainMessage {
//! #     subject: String,
//! #     content: String,
//! # }
//! #
//! # #[derive(thiserror::Error, Debug, PartialEq, Eq)]
//! # #[error("failed to send message because {message}")]
//! # struct Error {
//! #     message: String,
//! # }
//! #
//! # #[derive(Debug, Clone, PartialEq, Eq)]
//! # struct ApiMessage {
//! #     subject: String,
//! #     content: String,
//! # }
//! #
//! # #[derive(thiserror::Error, Debug)]
//! # #[error("some error occurred in the mail api")]
//! # struct ApiError;
//! #
//! # use output_tracker::non_threadsafe::{Error as OtError, OutputTracker, OutputSubject};
//! #
//! # struct MessageSender {
//! #     mail_api: Box<dyn MailApi>,
//! #     // the output-subject to create output-trackers from
//! #     message_subject: OutputSubject<ApiMessage>,
//! # }
//! #
//! # impl MessageSender {
//! #     // this constructor function is used in production code
//! #     fn new() -> Self {
//! #         Self {
//! #             mail_api: Box::new(RealMail),
//! #             message_subject: OutputSubject::new(),
//! #         }
//! #     }
//! #
//! #     // this constructor function is used in tests using the nullable pattern
//! #     fn nulled() -> Self {
//! #         Self {
//! #             mail_api: Box::new(NulledMail),
//! #             message_subject: OutputSubject::new(),
//! #         }
//! #     }
//! #
//! #     // function to create output-tracker for tracking sent messages
//! #     fn track_messages(&self) -> Result<OutputTracker<ApiMessage>, OtError> {
//! #         self.message_subject.create_tracker()
//! #     }
//! #
//! #     fn send_message(&self, message: DomainMessage) -> Result<(), Error> {
//! #         let mail = ApiMessage {
//! #             subject: message.subject,
//! #             content: message.content,
//! #         };
//! #
//! #         // code before and after this call to the `MailApi` is tested by our tests
//! #         let result = self.mail_api.send_mail(mail.clone());
//! #
//! #         result.map_err(|err| Error { message: err.to_string() })
//! #             // emit sent mail to all active output-trackers
//! #             .inspect(|()| _ = self.message_subject.emit(mail))
//! #     }
//! # }
//! #
//! # trait MailApi {
//! #     fn send_mail(&self, message: ApiMessage) -> Result<(), ApiError>;
//! # }
//! #
//! # struct RealMail;
//! #
//! # impl MailApi for RealMail {fn send_mail(&self, message: ApiMessage) -> Result<(), ApiError> {
//! #         unimplemented!("implementation is left out for this example
//! #                + as it is not executed in tests using nullables")
//! #     }
//! # }
//! #
//! # struct NulledMail;
//! #
//! # impl MailApi for NulledMail {
//! #     fn send_mail(&self, message: ApiMessage) -> Result<(), ApiError> {
//! #         // nothing to do here in the simplest case
//! #         Ok(())
//! #     }
//! # }
//! #
//! # fn main() {
//! #     domain_message_is_sent_via_the_mail_api();
//! # }
//! #
//! //#[test]
//! fn domain_message_is_sent_via_the_mail_api() {
//!     //
//!     // Arrange
//!     //
//!
//!     // set up nulled `MessageSender`
//!     let message_sender = MessageSender::nulled();
//!
//!     // create an output-tracker to track sent messages
//!     let message_tracker = message_sender.track_messages()
//!         .unwrap_or_else(|err| panic!("could not create message tracker because {err}"));
//!
//!     //
//!     // Act
//!     //
//!
//!     let message = DomainMessage {
//!         subject: "Monthly report for project X".into(),
//!         content: "Please provide the monthly report for project X due by end of the week".into(),
//!     };
//!
//!     let result = message_sender.send_message(message);
//!
//!     //
//!     // Assert
//!     //
//!
//!     assert_eq!(result, Ok(()));
//!
//!     // read the output from the message tracker
//!     let output = message_tracker.output()
//!         .unwrap_or_else(|err| panic!("could not read output of message tracker because {err}"));
//!
//!     assert_eq!(output, vec![
//!         ApiMessage {
//!             subject: "Monthly report for project X".into(),
//!             content: "Please provide the monthly report for project X due by end of the week".into(),
//!         }
//!     ])
//! }
//! ```
//!
//! See the integration tests of this crate as they demonstrate the usage of
//! output-tracker in a more involved and complete way.
//!
//! ## Threadsafe and non-threadsafe variants
//!
//! The output-tracker functionality is provided in a non-threadsafe variant and
//! a threadsafe one. The different variants are gated behind crate features and
//! can be activated as needed. The API of the two variants is interchangeable.
//! That is the struct names and functions are identical for both variants. The
//! module from which the structs are imported determines which variant is going
//! to be used.
//!
//! By default, only the non-threadsafe variant is compiled. One can activate
//! only one variant or both variants as needed. If the feature `threadsafe` is
//! specified, only the threadsafe variant is compiled. To use both variants at
//! the same time both features must be specified. The crate features and the
//! variants which are activated by each feature are listed in the table below.
//!
//! | Crate feature    | Variant        | Rust module import                                        |
//! |:-----------------|:---------------|:----------------------------------------------------------|
//! | `non-threadsafe` | non-threadsafe | [`use output_tracker::non_threadsafe::*`][non_threadsafe] |
//! | `threadsafe`     | threadsafe     | [`use output_tracker::threadsafe::*`][threadsafe]         |
//!
//! [nullables]: https://www.jamesshore.com/v2/projects/nullables

#![doc(html_root_url = "https://docs.rs/output-tracker/0.1.0")]

mod inner_subject;
mod inner_tracker;
#[cfg(any(feature = "non-threadsafe", not(feature = "threadsafe")))]
pub mod non_threadsafe;
#[cfg(feature = "threadsafe")]
pub mod threadsafe;
mod tracker_handle;

// test code snippets in the README.md
#[cfg(doctest)]
#[doc = include_str!("../README.md")]
#[allow(dead_code)]
type TestExamplesInReadme = ();

// workaround for false positive 'unused extern crate' warnings until
// Rust issue [#95513](https://github.com/rust-lang/rust/issues/95513) is fixed
#[cfg(test)]
mod dummy_extern_uses {
    use version_sync as _;
}
