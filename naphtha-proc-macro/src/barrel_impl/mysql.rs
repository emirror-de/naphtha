use quote::quote;

pub(crate) fn impl_mysql() -> ::proc_macro2::TokenStream {
    quote! {
        impl ::naphtha::barrel::DatabaseSqlMigrationExecutor<::diesel::MysqlConnection, usize> for Person
        where
            Self: ::naphtha::barrel::DatabaseSqlMigration,
        {
            fn execute_migration_up(conn: &::naphtha::DatabaseConnection<::diesel::MysqlConnection>) -> Result<usize, String> {
                use {
                    ::log::error,
                    ::naphtha::{barrel::Migration, DatabaseConnection},
                    crate::diesel::RunQueryDsl,
                };
                let mut m = Migration::new();
                Self::migration_up(&mut m);
                let m = m.make::<::naphtha::barrel::backend::MySql>();

                let c = match conn.lock() {
                    Ok(c) => c,
                    Err(msg) => {
                        error!("Could not aquire lock on DatabaseSqlMigrationExecutor::execute_migration_up: {}", msg.to_string());
                        return Err(msg.to_string());
                    }
                };

                match ::diesel::sql_query(m).execute(&*c) {
                    Ok(u) => Ok(u),
                    Err(msg) => Err(msg.to_string()),
                }
            }

            fn execute_migration_down(conn: &::naphtha::DatabaseConnection<::diesel::MysqlConnection>) -> Result<usize, String> {
                use {
                    ::log::error,
                    ::naphtha::{barrel::Migration, DatabaseConnection},
                    crate::diesel::RunQueryDsl,
                };
                let mut m = Migration::new();
                Self::migration_down(&mut m);
                let m = m.make::<::naphtha::barrel::backend::MySql>();

                let c = match conn.lock() {
                    Ok(c) => c,
                    Err(msg) => {
                        error!("Could not aquire lock on DatabaseSqlMigrationExecutor::execute_migration_down for model: {}", msg.to_string());
                        return Err(msg.to_string());
                    }
                };

                match ::diesel::sql_query(m).execute(&*c) {
                    Ok(u) => Ok(u),
                    Err(msg) => Err(msg.to_string()),
                }
            }
        }
    }
}
