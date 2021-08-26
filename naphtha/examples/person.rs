#[macro_use]
extern crate diesel;

use chrono::prelude::NaiveDateTime;
use naphtha::{
    barrel::{types, DatabaseSqlMigration, Migration},
    model,
    DatabaseUpdateHandler,
};

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

#[cfg(any(
    feature = "barrel-full",
    feature = "barrel-sqlite",
))]
impl DatabaseSqlMigration for Person {
    fn migration_up(migration: &mut Migration) {
        use naphtha::DatabaseModel;
        migration.create_table_if_not_exists(Self::table_name(), |t| {
            t.add_column("id", types::primary());
            t.add_column("description", types::text().nullable(true));
            t.add_column("updated_at", types::custom("timestamp"));
        });
    }

    fn migration_down(migration: &mut Migration) {
        use naphtha::DatabaseModel;
        migration.drop_table_if_exists(Self::table_name());
    }
}

fn main() {
    use naphtha::{
        barrel::DatabaseSqlMigrationExecutor,
        DatabaseConnect,
        DatabaseConnection,
        DatabaseModel,
    };

    let db = DatabaseConnection::connect(":memory:").unwrap();

    // create the table if not existent
    // This method can be used on startup of your application to make sure
    // your database schema is always up to date.
    Person::execute_migration_up(&db);

    let mut p = Person {
        id: Person::default_primary_key(),
        description: Some("The new person is registered".into()),
        updated_at: chrono::Utc::now().naive_utc(),
    };

    p.insert(&db);
    // id member is set to the correct number given by the database.

    // do a custom query to the database
    let res = db.custom::<diesel::result::QueryResult<Person>>(|c| {
        use schema::persons::dsl::*;
        persons.filter(id.eq(1)).first(c)
    });
    println!("{:#?}", res);

    p.remove(&db);
    // p not available anymore
}
