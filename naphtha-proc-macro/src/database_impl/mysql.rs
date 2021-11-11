use {
    quote::quote,
    syn::{Data::Struct, DeriveInput},
};

pub(crate) fn impl_mysql(
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
    assert!(
        crate::helper::has_id(ast),
        "No `id` member found in model `{}`. Currently only models having an `id` column of type `i32` are supported.",
        name
        );

    quote! {
        impl ::naphtha::DatabaseModelModifier<MysqlConnection> for #name
        where
            Self: ::naphtha::DatabaseUpdateHandler<MysqlConnection>
            + ::naphtha::DatabaseInsertHandler<MysqlConnection>
            + ::naphtha::DatabaseRemoveHandler<MysqlConnection>,
        {
            fn insert(&mut self, conn: &::naphtha::DatabaseConnection<MysqlConnection>) -> bool {
                use {
                    ::naphtha::{log, DatabaseModel, diesel::{Connection, RunQueryDsl, ExpressionMethods, Table, QueryDsl}},
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
                        log::error!("Could not aquire lock on DatabaseModifier::insert for model:\n{:#?}", self);
                        return false;
                    }
                };
                self.pre_insert(conn);
                let res_id = match c.transaction::<_, ::naphtha::diesel::result::Error, _>(|| {
                    ::naphtha::diesel::insert_into(#table_name)
                        .values((#insert_properties))
                        .execute(&*c)?;
                    #table_name.select(#table_name.primary_key())
                        .order(#table_name.primary_key().desc())
                        .first(&*c)
                }) {
                    Ok(v) => v,
                    Err(msg) => {
                        log::error!("Failed inserting entity:\n{:#?}", self);
                        return false;
                    }
                };
                self.set_primary_key(&res_id);
                self.post_insert(conn);
                true
            }

            fn update(&mut self, conn: &::naphtha::DatabaseConnection<MysqlConnection>) -> bool {
                use ::naphtha::{diesel::SaveChangesDsl, log};
                let c = match conn.lock() {
                    Ok(c) => c,
                    Err(msg) => {
                        log::error!("Could not aquire lock on DatabaseModifier::update for model:\n{:#?}", self);
                        return false;
                    }
                };
                self.pre_update(conn);
                let update_result = match self.save_changes::<Self>(&*c) {
                    Ok(_) => true,
                    Err(msg) => {
                        log::error!("Failed updating entity:\n{:#?}", self);
                        return false;
                    },
                };
                self.post_update(conn);
                update_result
            }

            fn remove(&mut self, conn: &::naphtha::DatabaseConnection<MysqlConnection>) -> bool {
                use {
                    ::naphtha::{log::{self, info}, DatabaseModel, diesel::{ExpressionMethods, RunQueryDsl, QueryDsl, Table}},
                    schema::{#table_name, #table_name::dsl::*},
                };
                let c = match conn.lock() {
                    Ok(c) => c,
                    Err(msg) => {
                        log::error!("Could not aquire lock on DatabaseModifier::remove for model:\n{:#?}", self);
                        return false;
                    }
                };
                self.pre_remove(conn);
                let num_deleted = ::naphtha::diesel::delete(
                    #table_name.filter(
                        #table_name.primary_key().eq(self.primary_key())
                    )
                );
                let num_deleted = match num_deleted.execute(&*c) {
                    Ok(_) => {
                        #[cfg(debug_assertions)]
                        info!("Removed entity with primary key {} from database!", self.primary_key());
                        true
                    },
                    Err(msg) => {
                        log::error!("Could not aquire lock on DatabaseModifier::remove for model:\n{:#?}", self);
                        false
                    }
                };
                self.post_remove(conn);
                num_deleted
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
                fn #function_name(conn: &::naphtha::DatabaseConnection<MysqlConnection>, property: &#fieldtype)
                    -> ::naphtha::diesel::result::QueryResult<#return_type> {
                    use schema::{#table_name, #table_name::dsl::*};
                    use ::naphtha::diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};
                    conn.custom::<::naphtha::diesel::result::QueryResult<#return_type>, _>(|c| {
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
        impl QueryByProperties<MysqlConnection> for #name {
            type Error = ::naphtha::diesel::result::Error;
            #queries
            #query_by_ids
        }
    }
}

fn impl_query_by_ids(
    ast: &::syn::DeriveInput,
    table_name_attr: &::proc_macro2::TokenStream,
) -> ::proc_macro2::TokenStream {
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
                fn query_by_ids(conn: &::naphtha::DatabaseConnection<MysqlConnection>, ids: &[#fieldtype])
                    -> ::naphtha::diesel::result::QueryResult<Vec<Self>> {
                    use {
                        schema::{#table_name, #table_name::dsl::*},
                        ::naphtha::diesel::{ExpressionMethods, QueryDsl, RunQueryDsl},
                    };
                    conn.custom::<::naphtha::diesel::result::QueryResult<Vec<Self>>, _>(|c| {
                        #table_name.filter(#fieldname.eq_any(ids)).load::<Self>(&*c)
                    })
                }
        };
        break;
    }

    query
}
