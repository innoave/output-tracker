//! An example demonstrating how to test a repository component using the
//! `nullables` pattern and an output-tracker.
//!
//! This example is more involved and shows details on how to write tests
//! without using mocks in more real world like example.

mod fixture;

//
// Production code
//

mod todo_domain {
    pub struct NewTodo {
        pub subject: String,
    }
}

mod todo_repository {
    use crate::todo_domain::NewTodo;
    use output_tracker::non_threadsafe::Error as OtError;
    use output_tracker::non_threadsafe::OutputSubject;
    use output_tracker::non_threadsafe::OutputTracker;

    #[derive(thiserror::Error, Debug)]
    #[error("failed to access the database")]
    pub struct DbError {
        pub message: String,
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct TodoEntity {
        pub subject: String,
    }

    /// An example of a minimal repository holding to-do items.
    pub struct TodoRepository {
        db: Box<dyn DbAccess>,
        todo_subject: OutputSubject<TodoEntity>,
    }

    impl TodoRepository {
        #[allow(dead_code)]
        pub fn new() -> Self {
            Self {
                db: Box::new(RealDb),
                todo_subject: OutputSubject::new(),
            }
        }

        pub fn nulled() -> Self {
            Self {
                db: Box::new(NulledDb),
                todo_subject: OutputSubject::new(),
            }
        }

        pub fn track_todos(&self) -> Result<OutputTracker<TodoEntity>, OtError> {
            self.todo_subject.create_tracker()
        }

        pub fn insert(&self, new_todo: NewTodo) -> Result<(), DbError> {
            let todo_entity = TodoEntity {
                subject: new_todo.subject,
            };

            // any code before and after this function call to `self.db.insert_todo` is tested
            // even with the nulled variant of the `TodoRepository` instance
            let result = self.db.insert_todo(todo_entity.clone());

            result.inspect(|()| {
                _ = self.todo_subject.emit(todo_entity);
            })
        }
    }

    #[allow(dead_code)]
    struct RealDb;

    impl DbAccess for RealDb {
        fn insert_todo(&self, _todo_entity: TodoEntity) -> Result<(), DbError> {
            // here is the lowest level real code
            // to insert a new to-do entity
            // into the database
            unimplemented!("not implemented for the example")
        }
    }

    //
    // Nullability
    //

    // trait defining the low level access to the database
    trait DbAccess {
        fn insert_todo(&self, todo_entity: TodoEntity) -> Result<(), DbError>;
    }

    struct NulledDb;

    impl DbAccess for NulledDb {
        fn insert_todo(&self, _todo_entity: TodoEntity) -> Result<(), DbError> {
            Ok(())
        }
    }
}

//
// Tests
//

use crate::todo_domain::NewTodo;
use crate::todo_repository::{TodoEntity, TodoRepository};
use assertor::*;

#[test]
fn insert_new_todo_item_into_repository() {
    //
    // Arrange
    //

    let todo_repo = TodoRepository::nulled();

    // activate the output tracker
    let todo_tracker = todo_repo
        .track_todos()
        .unwrap_or_else(|err| panic!("could not create todo tracker: {err}"));

    //
    // Act
    //

    let inserted = todo_repo.insert(NewTodo {
        subject: "remember the milk".into(),
    });

    //
    // Assert
    //

    assert_that!(inserted).is_ok();

    // read the output from the output tracker
    let inserted_todos = todo_tracker
        .output()
        .unwrap_or_else(|err| panic!("could not read output of todo tracker: {err}"));

    assert_that!(inserted_todos).contains_exactly_in_order(vec![TodoEntity {
        subject: "remember the milk".into(),
    }]);
}
