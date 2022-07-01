// You can run this example on different databases (Docker files provided in
// the repository).
//
// Select the database by compiling this example with the corresponding features:
// * sqlite and barrel-sqlite
// * mysql and barrel-mysql
// * pg and barrel-pg

#[macro_use]
extern crate diesel;

#[cfg(any(
    feature = "barrel-sqlite",
    feature = "barrel-mysql",
    feature = "barrel-pg"
))]
use naphtha::barrel::{
    types,
    DatabaseSqlMigration,
    DatabaseSqlMigrationExecutor,
    Migration,
};

use {
    chrono::prelude::NaiveDateTime,
    naphtha::{
        diesel::prelude::*,
        model,
        DatabaseConnect,
        DatabaseConnection,
        DatabaseInsertHandler,
        DatabaseModel,
        DatabaseModelModifier,
        DatabaseRemoveHandler,
        DatabaseUpdateHandler,
    },
};

const DATABASE_URL: &'static str = if cfg!(feature = "sqlite") {
    ":memory:"
} else if cfg!(feature = "mysql") {
    "mysql://naphtha:naphtha@127.0.0.1:3306/naphtha"
} else if cfg!(feature = "pg") {
    "postgres://naphtha:naphtha@127.0.0.1:5432/naphtha"
} else {
    "not supported"
};

// USE THE ACCORDING FEATURE FOR DATABASE TYPE SELECTION
#[cfg(feature = "sqlite")]
type DbBackend = diesel::SqliteConnection;
#[cfg(feature = "mysql")]
type DbBackend = diesel::MysqlConnection;
#[cfg(feature = "pg")]
type DbBackend = diesel::PgConnection;

// To see what the model attribute adds to your source code, use
// the 'cargo expand' command.
#[model(table_name = "persons", primary_key = "entity_id")]
pub struct Person {
    entity_id: i32,
    pub description: Option<String>,
    pub updated_at: NaiveDateTime,
}

pub mod schema {
    table! {
        persons (entity_id) {
            entity_id -> Int4,
            description -> Nullable<Varchar>,
            updated_at -> Timestamp,
        }
    }
}

impl naphtha::DatabaseModel for Person {
    type PrimaryKey = i32;

    fn primary_key(&self) -> Self::PrimaryKey {
        self.entity_id
    }

    fn set_primary_key(&mut self, value: &Self::PrimaryKey) {
        self.entity_id = *value;
    }

    fn default_primary_key() -> Self::PrimaryKey {
        0
    }

    fn table_name() -> &'static str {
        "persons"
    }
}

impl<T> DatabaseUpdateHandler<T> for Person {
    fn pre_update(&mut self, _conn: &DatabaseConnection<T>) {
        self.updated_at = chrono::Utc::now().naive_utc();
    }

    fn post_update(&mut self, _conn: &DatabaseConnection<T>) {}
}

impl<T> DatabaseInsertHandler<T> for Person {}
impl<T> DatabaseRemoveHandler<T> for Person {}

#[cfg(any(
    feature = "barrel-sqlite",
    feature = "barrel-mysql",
    feature = "barrel-pg"
))]
impl DatabaseSqlMigration for Person {
    fn migration_up(migration: &mut Migration) {
        migration.create_table_if_not_exists(Self::table_name(), |t| {
            t.add_column("entity_id", types::primary());
            t.add_column("description", types::text().nullable(true));
            t.add_column("updated_at", types::custom("timestamp"));
        });
    }

    fn migration_down(migration: &mut Migration) {
        migration.drop_table_if_exists(Self::table_name());
    }
}

fn main() {
    println!("Using {}", DATABASE_URL);
    let db = DatabaseConnection::connect(DATABASE_URL).unwrap();

    // create the table if not existent
    // This method can be used on startup of your application to make sure
    // your database schema is always up to date.
    #[cfg(any(
        feature = "barrel-sqlite",
        feature = "barrel-mysql",
        feature = "barrel-pg"
    ))]
    match Person::execute_migration_up(&db) {
        Ok(_) => (),
        Err(msg) => println!("Could not create table: {}", msg),
    };

    let mut p = Person {
        entity_id: Person::default_primary_key(),
        description: Some("The new person is registered".into()),
        updated_at: chrono::Utc::now().naive_utc(),
    };

    p.insert(&db);
    // id member is set to the correct number given by the database.

    // do a custom query to the database
    let res =
        db.custom::<diesel::result::QueryResult<Person>, _>(|c: &DbBackend| {
            use schema::persons::dsl::*;
            persons.filter(entity_id.eq(1)).first(c)
        });
    let queried_by_id = Person::query_by_entity_id(&db, &1);
    println!("{:#?}", res);
    println!("{:#?}", queried_by_id);

    p.remove(&db);
    // p not available anymore

    #[cfg(any(
        feature = "barrel-sqlite",
        feature = "barrel-mysql",
        feature = "barrel-pg"
    ))]
    match Person::execute_migration_down(&db) {
        Ok(_) => (),
        Err(msg) => println!("Could not drop table: {}", msg),
    };
}
