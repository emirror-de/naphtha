use {
    quote::quote,
    syn::{Data::Struct, DeriveInput},
};

pub(crate) fn impl_sqlite(
    ast: &DeriveInput,
    attr: &::proc_macro2::TokenStream,
) -> ::proc_macro2::TokenStream {
    let database_modifier = impl_database_modifier(ast, attr);
    let query_by_property = impl_query_by_property(ast, attr);
    quote! {
        #database_modifier
        #query_by_property
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
            Self: ::naphtha::DatabaseUpdateHandler,
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
                self.pre_update();
                let update_result = match self.save_changes::<Self>(&*c) {
                    Ok(_) => true,
                    Err(msg) => {
                        error!("Failed updating entity:\n{:#?}", self);
                        return false;
                    },
                };
                self.post_update();
                update_result
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

pub fn impl_query_by_property(
    ast: &::syn::DeriveInput,
    table_name_attr: &::proc_macro2::TokenStream,
) -> ::proc_macro2::TokenStream {
    let name = &ast.ident;
    let table_name = crate::helper::extract_table_name(table_name_attr);
    let data = match &ast.data {
        ::syn::Data::Struct(data) => data,
        _ => {
            // return no code if it is not a struct
            return quote! {};
        }
    };
    let mut queries = quote! {};
    for field in data.fields.iter() {
        if field.ident.is_none() {
            continue;
        }
        let fieldname = field.ident.as_ref().unwrap();
        let (return_type, diesel_query_fn) = match &fieldname.to_string()[..] {
            "updated_at" => continue,
            "id" => (quote! { Self }, quote! { first }),
            _ => (quote! { Vec<Self> }, quote! { load }),
        };
        let function_name = ::proc_macro2::Ident::new(
            &format!("query_by_{}", fieldname).to_lowercase(),
            ::proc_macro2::Span::call_site(),
        );
        let fieldtype = &field.ty;
        let query = quote! {
                fn #function_name(conn: &DatabaseConnection<SqliteConnection>, property: &#fieldtype)
                    -> ::diesel::result::QueryResult<#return_type> {
                    use schema::{#table_name, #table_name::dsl::*};
                    conn.custom::<::diesel::result::QueryResult<#return_type>, _>(|c| {
                        #table_name.filter(#fieldname.eq(property))
                            .#diesel_query_fn::<Self>(&*c)
                    })
                }
        };
        queries = quote! {
            #queries
            #query
        };
    }

    let query_by_ids = if crate::helper::has_id(ast) {
        impl_query_by_ids(ast, table_name_attr)
    } else {
        quote! {}
    };

    quote! {
        impl QueryByProperties<SqliteConnection> for #name {
            type Error = ::diesel::result::Error;
            #queries
            #query_by_ids
        }
    }
}

fn impl_query_by_ids(
    ast: &::syn::DeriveInput,
    table_name_attr: &::proc_macro2::TokenStream,
) -> ::proc_macro2::TokenStream {
    let name = &ast.ident;
    let table_name = crate::helper::extract_table_name(table_name_attr);
    let data = match &ast.data {
        ::syn::Data::Struct(data) => data,
        _ => {
            // return only defaults if it is not a struct
            return quote! {};
        }
    };
    let mut query = quote! {};
    for field in data.fields.iter() {
        if field.ident.is_none() {
            continue;
        }
        let fieldname = field.ident.as_ref().unwrap();
        match &fieldname.to_string()[..] {
            "id" => (),
            _ => continue,
        };
        let fieldtype = &field.ty;
        query = quote! {
                fn query_by_ids(conn: &DatabaseConnection<SqliteConnection>, ids: &[#fieldtype])
                    -> ::diesel::result::QueryResult<Vec<Self>> {
                    use {
                        schema::{#table_name, #table_name::dsl::*},
                    };
                    conn.custom::<::diesel::result::QueryResult<Vec<Self>>, _>(|c| {
                        #table_name.filter(#fieldname.eq_any(ids)).load::<Self>(&*c)
                    })
                }
        };
        break;
    }

    query
}
