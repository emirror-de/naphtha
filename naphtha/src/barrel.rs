pub use crate::DatabaseConnection;
use barrel::Migration;

/// Provides an interface for the migration functions of the table belonging
/// to your model.
///
/// *Requires any of the barrel features `barrel-full`, `barrel-sqlite`, `barrel-pg` or
/// `barrel-mysql`*
pub trait DatabaseSqlMigration {
    /// Defines the creation of a table.
    fn migration_up(migration: &mut Migration);
    /// Defines the deletion of a table.
    fn migration_down(migration: &mut Migration);
}

/// Gets implemented automatically when `barrel` feature is enabled.
pub trait DatabaseSqlMigrationExecutor<Conn, T>
where
    Self: DatabaseSqlMigration,
{
    /// Executes the creation of the table.
    fn execute_migration_up(
        conn: &DatabaseConnection<Conn>,
    ) -> Result<T, String>;
    /// Executes the deletion of the table.
    fn execute_migration_down(
        conn: &DatabaseConnection<Conn>,
    ) -> Result<T, String>;
}
