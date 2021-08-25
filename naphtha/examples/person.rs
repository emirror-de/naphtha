#[macro_use]
extern crate diesel;

use chrono::prelude::NaiveDateTime;
use naphtha::{model, DatabaseUpdateHandler};

// The model attribute automatically adds:
//
// use schema::*;
// #[derive(Debug, Queryable, Identifiable, AsChangeset, Associations)]
// #[table_name = "persons"]
#[model(table_name = "persons")]
pub struct Person {
    id: i32,
    pub description: Option<String>,
    pub updated_at: NaiveDateTime,
}

pub mod schema {
    table! {
        persons (id) {
            id -> Int4,
            description -> Nullable<Varchar>,
            updated_at -> Timestamp,
        }
    }
}

impl naphtha::DatabaseModel for Person {
    type PrimaryKey = i32;
    fn primary_key(&self) -> Self::PrimaryKey {
        self.id
    }

    fn set_primary_key(&mut self, value: &Self::PrimaryKey) {
        self.id = *value;
    }

    fn default_primary_key() -> Self::PrimaryKey {
        0
    }

    fn table_name() -> &'static str {
        "persons"
    }
}

impl DatabaseUpdateHandler for Person {
    fn before_update(&mut self) {
        self.updated_at = chrono::Utc::now().naive_utc();
    }

    fn after_update(&mut self) {}
}

fn main() {
    println!("done!");
}
