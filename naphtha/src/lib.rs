#![deny(missing_docs)]
//! This library provides several traits in order to make database access a lot
//! easier. In addition when using `naphtha`, it is possible to change
//! the database that is used for specific models in your application without
//! the requirement of changing any code.
//!
//! It implements the most common operations on a database like `insert`, `update`
//! and `remove` for you, while also providing the ability to send custom queries
//! to the database.
//! See the [examples](#examples) below.
//!
//! ## Features overview
//!
//! * Change database on specific model in your application without the need to
//! change your code.
//! * Most common function implementations `insert`, `update`, `remove` for your
//! models.
//! * Possibility to query a model from the database by using one of its member *(NOT FINISHED YET)*.
//! * Thread safe handling of the database connection.
//!
//! ## Supported databases
//!
//! * SQlite3 (using [diesel](diesel) under the hood).
//!
//! ## Examples
//!
//! In this chapter, minimal usages are shown. Please have a look at the examples
//! in the repository for more and detailed use.
//!
//! ### Connecting to a database
//!
//! ```rust
//! use naphtha::{DatabaseConnection, DatabaseConnect};
//! let db = DatabaseConnection::connect(":memory:");
//! // do some database work
//! ```
//!
//! ### Defining a model and use database connection
//!
//! ```rust
//! #[macro_use]
//! extern crate diesel;
//! #[macro_use]
//! extern crate naphtha;
//!
//! use {naphtha::model, diesel::table};
//!
//! #[model(table_name = "persons")]
//! pub struct Person {
//!     id: i32,
//!     pub description: Option<String>,
//! }
//!
//! pub mod schema {
//!     table! {
//!         persons (id) {
//!             id -> Int4,
//!             description -> Nullable<Varchar>,
//!         }
//!     }
//! }
//!
//! impl naphtha::DatabaseModel for Person {
//!     type PrimaryKey = i32;
//!     fn primary_key(&self) -> Self::PrimaryKey {
//!         self.id
//!     }
//!
//!     fn set_primary_key(&mut self, value: &Self::PrimaryKey) {
//!         self.id = *value;
//!     }
//!
//!     fn default_primary_key() -> Self::PrimaryKey {
//!         0
//!     }
//!
//!     fn table_name() -> &'static str {
//!         "persons"
//!     }
//! }
//!
//! fn main() {
//!     use naphtha::{DatabaseConnection, DatabaseConnect, DatabaseModel};
//!     let db = DatabaseConnection::connect(":memory:").unwrap();
//!     let mut p = Person {
//!         id: Person::default_primary_key(),
//!         description: Some("The new person is registered".into()),
//!     };
//!     p.insert(&db);
//!     // id member is set to the correct number given by the database.
//!
//!     // do a custom query to the database
//!     db.custom::<diesel::result::QueryResult::<Person>>(|c| {
//!         use schema::{persons, persons::dsl::*};
//!         persons.filter(id.eq(1)).first(c)
//!     });
//!
//!     p.remove(&db);
//!     // p not available anymore
//! }
//! ```
//!
//! ## Upcoming
//!
//! * The query by methods will be implemented.
//! * More databases will be implemented, at least PostgreSQL and MySql.
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

/// Defines your `struct` as a model and implements the required traits for
/// interacting with the database. Currently only *named* `struct` member are
/// supported.
pub use naphtha_proc_macro::model;

mod database_impl;
mod tests;

/// Thin wrapper around a [Connection](diesel::Connection).
pub struct DatabaseConnection<T>(Arc<Mutex<T>>);

impl<T> DatabaseConnection<T> {
    /// Aquires a lock to the wrapped connection.
    pub fn lock(
        &self,
    ) -> Result<MutexGuard<'_, T>, PoisonError<MutexGuard<'_, T>>> {
        self.0.lock()
    }

    /// Executes the custom function to the database instance.
    pub fn custom<R>(&self, query: fn(&T) -> R) -> R {
        let c = self.0.lock().expect("Could not aquire connection lock!");
        query(&*c)
    }
}

/// Contains functions database connection handling.
pub trait DatabaseConnect<T> {
    /// Establishes a new connection to the given database string.
    fn connect(database_url: &str) -> Result<DatabaseConnection<T>, String>;
}

/// Defines the relation of the model to the database.
pub trait DatabaseModel {
    /// Defines the primary key type on the database table.
    type PrimaryKey;

    /// Returns the primary key value.
    fn primary_key(&self) -> Self::PrimaryKey;
    /// Sets the primary key value.
    fn set_primary_key(&mut self, value: &Self::PrimaryKey);
    /// Gets the default primary key value.
    fn default_primary_key() -> Self::PrimaryKey;
    /// Returns the table name related to the model.
    fn table_name() -> &'static str;
}

/// Defines functions to modify the stored model instance on the database.
pub trait DatabaseModelModifier<T>
where
    Self: Clone,
{
    /// Inserts `self` to the given database.
    /// *Updates the `primary_key` to the one that has been assigned by the database*.
    fn insert(&mut self, conn: &DatabaseConnection<T>) -> bool;
    /// Removes `self` from the database, selects by `id`.
    fn remove(self, conn: &DatabaseConnection<T>) -> bool;
    /// Updates `self` on the given database.
    /// *Updates the `updated_at` member if available before updating the database.*.
    fn update(&mut self, conn: &DatabaseConnection<T>) -> bool;
}
