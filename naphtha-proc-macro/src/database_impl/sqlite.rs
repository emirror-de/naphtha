use {
    quote::quote,
    syn::{Data::Struct, DeriveInput},
};

pub(crate) fn impl_sqlite(
    ast: &DeriveInput,
    attr: &::proc_macro2::TokenStream,
) -> ::proc_macro2::TokenStream {
    let database_modifier = impl_database_modifier(ast, attr);
    quote! {
        #database_modifier
    }
}

fn impl_database_modifier(
    ast: &DeriveInput,
    table_name: &::proc_macro2::TokenStream,
) -> ::proc_macro2::TokenStream {
    let name = &ast.ident;
    let table_name = crate::helper::extract_table_name(table_name);
    //let table_name: syn::UsePath = syn::parse_quote! {#name::table_name()};

    let insert_properties = generate_insert_properties(ast);
    if !crate::helper::has_id(ast) {
        panic!("No `id` member found in model `{}`. Currently only models having an `id` column of type `i32` are supported.", name);
    }

    /*
    let model_has_id_member = crate::helper::has_id(ast);
                    if (#model_has_id_member) {
                        #table_name.select(#table_name.primary_key())
                            .order(#table_name.primary_key().desc())
                            .first(&*c)
                    } else {
                        #table_name.select(#table_name.primary_key())
                            .filter(#table_name.primary_key().eq(self.primary_key()))
                            .first(&*c)
                    }
    */

    quote! {
        use {
            ::diesel::{backend::Backend, prelude::*},
            ::log::error,
            ::naphtha::{DatabaseModelModifier, DatabaseConnection},
        };
        impl DatabaseModelModifier<SqliteConnection> for #name
        where
            Self: Clone,
        {
            fn insert(&mut self, conn: &DatabaseConnection<SqliteConnection>) -> bool {
                use {
                    ::naphtha::DatabaseModel,
                    schema::{#table_name, #table_name::dsl::*},
                };
                // preventing duplicate insertion if default primary key gets
                // changed on database insertion.
                if self.primary_key() != Self::default_primary_key() {
                    return false;
                }
                let c = match conn.lock() {
                    Ok(c) => c,
                    Err(msg) => {
                        error!("Could not aquire lock on DatabaseModifier::insert for model:\n{:#?}", self);
                        return false;
                    }
                };
                let res_id = match c.transaction::<_, ::diesel::result::Error, _>(|| {
                    diesel::insert_into(#table_name)
                        .values((#insert_properties))
                        .execute(&*c)?;
                    #table_name.select(#table_name.primary_key())
                        .order(#table_name.primary_key().desc())
                        .first(&*c)
                }) {
                    Ok(v) => v,
                    Err(msg) => {
                        error!("Failed inserting entity:\n{:#?}", self);
                        return false;
                    }
                };
                self.set_primary_key(&res_id);
                true
            }

            fn update(&mut self, conn: &DatabaseConnection<SqliteConnection>) -> bool {
                let c = match conn.lock() {
                    Ok(c) => c,
                    Err(msg) => {
                        error!("Could not aquire lock on DatabaseModifier::update for model:\n{:#?}", self);
                        return false;
                    }
                };
                match self.save_changes::<Self>(&*c) {
                    Ok(_) => true,
                    Err(msg) => {
                        error!("Failed updating entity:\n{:#?}", self);
                        return false;
                    },
                }
            }

            fn remove(self, conn: &DatabaseConnection<SqliteConnection>) -> bool {
                use {
                    ::log::info,
                    ::naphtha::DatabaseModel,
                    schema::{#table_name, #table_name::dsl::*},
                };
                let c = match conn.lock() {
                    Ok(c) => c,
                    Err(msg) => {
                        error!("Could not aquire lock on DatabaseModifier::remove for model:\n{:#?}", self);
                        return false;
                    }
                };
                let num_deleted = ::diesel::delete(
                    #table_name.filter(
                        #table_name.primary_key().eq(self.primary_key())
                    )
                );
                match num_deleted.execute(&*c) {
                    Ok(_) => {
                        #[cfg(debug_assertions)]
                        info!("Removed entity with primary key {} from database!", self.primary_key());
                        true
                    },
                    Err(msg) => {
                        error!("Could not aquire lock on DatabaseModifier::remove for model:\n{:#?}", self);
                        false
                    }
                }
            }
        }
    }
}

fn generate_insert_properties(ast: &DeriveInput) -> ::proc_macro2::TokenStream {
    let data = match &ast.data {
        Struct(data) => data,
        _ => panic!("Other data formats than \"struct\" is not supported yet!"),
    };
    let mut collected_properties = quote! {};
    for field in data.fields.iter() {
        if field.ident.is_none() {
            continue;
        }
        let fieldname = field.ident.as_ref().unwrap();
        if &fieldname.to_string()[..] == "id" {
            // field id is currently used as primary key and therfore generated
            // by the database, so it must not be set during insertion.
            continue;
        }
        collected_properties = quote! {
            #collected_properties
            #fieldname.eq(&self.#fieldname),
        };
    }
    collected_properties
}
