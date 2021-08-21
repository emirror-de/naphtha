#[macro_use]
extern crate diesel;

use chrono::prelude::NaiveDateTime;
use naphtha_proc_macro::model;

// The model attribute automatically adds:
//
// use schema::*;
// #[derive(Debug, Clone, Queryable, Identifiable, AsChangeset, Associations)]
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

fn main() {
}
