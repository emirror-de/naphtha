use {quote::quote, syn::DeriveInput};

pub(crate) fn impl_mysql(ast: &DeriveInput) -> ::proc_macro2::TokenStream {
    let name = &ast.ident;

    quote! {
        impl ::naphtha::barrel::DatabaseSqlMigrationExecutor<::naphtha::diesel::MysqlConnection, usize> for #name
        where
            Self: ::naphtha::barrel::DatabaseSqlMigration,
        {
            fn execute_migration_up(conn: &::naphtha::DatabaseConnection<::naphtha::diesel::MysqlConnection>) -> Result<usize, String> {
                use {
                    ::naphtha::{barrel::Migration, DatabaseConnection, log::error, diesel::RunQueryDsl},
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

                match ::naphtha::diesel::sql_query(m).execute(&*c) {
                    Ok(u) => Ok(u),
                    Err(msg) => Err(msg.to_string()),
                }
            }

            fn execute_migration_down(conn: &::naphtha::DatabaseConnection<::naphtha::diesel::MysqlConnection>) -> Result<usize, String> {
                use {
                    ::naphtha::{barrel::Migration, DatabaseConnection, log::error, diesel::RunQueryDsl},
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

                match ::naphtha::diesel::sql_query(m).execute(&*c) {
                    Ok(u) => Ok(u),
                    Err(msg) => Err(msg.to_string()),
                }
            }
        }
    }
}
