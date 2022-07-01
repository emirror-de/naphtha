use {
    quote::quote,
    syn::{Data::Struct, DeriveInput},
};

pub(crate) fn impl_pg(
    ast: &DeriveInput,
    params: &crate::params::Params,
) -> ::proc_macro2::TokenStream {
    let database_modifier = impl_database_modifier(ast, &params);
    let query_by_property = impl_query_by_property(ast, &params);
    quote! {
        #database_modifier
        #query_by_property
    }
}

fn impl_database_modifier(
    ast: &DeriveInput,
    params: &crate::params::Params,
) -> ::proc_macro2::TokenStream {
    let name = &ast.ident;

    let insert_properties = generate_insert_properties(ast, params);

    let table_name = ::proc_macro2::Ident::new(
        &params.table_name,
        ::proc_macro2::Span::call_site(),
    );
    quote! {
        impl ::naphtha::DatabaseModelModifier<::naphtha::diesel::PgConnection> for #name
        where
            Self: ::naphtha::DatabaseUpdateHandler<::naphtha::diesel::PgConnection>
            + ::naphtha::DatabaseInsertHandler<::naphtha::diesel::PgConnection>
            + ::naphtha::DatabaseRemoveHandler<::naphtha::diesel::PgConnection>,
        {
            fn insert(
                &mut self,
                conn: &::naphtha::DatabaseConnection<::naphtha::diesel::PgConnection>
                ) -> ::anyhow::Result<()> {
                use {
                    ::anyhow::Context,
                    ::naphtha::{log, DatabaseModel, diesel::{Connection, RunQueryDsl, ExpressionMethods, Table, QueryDsl}},
                    schema::{#table_name, #table_name::dsl::*},
                };
                // preventing duplicate insertion if default primary key gets
                // changed on database insertion.
                if self.primary_key() != Self::default_primary_key() {
                    return Err(::anyhow::anyhow!("Could not update primary key!"));
                }
                let c = match conn.lock() {
                    Ok(c) => c,
                    Err(msg) => {
                        let message = format!(
                            "Could not aquire lock on DatabaseModifier::insert for model:\nError: {}\nModel:\n{:#?}",
                            msg,
                            self
                            );
                        log::error!("{}", message);
                        return Err(::anyhow::anyhow!("{}", message));
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
                        let message = format!(
                            "Failed inserting entity:\nError: {}\nModel: {:#?}",
                            msg,
                            self
                            );
                        log::error!("{}", message);
                        return Err(::anyhow::anyhow!("{}", message));
                    }
                };
                self.set_primary_key(&res_id);
                self.post_insert(conn);
                Ok(())
            }

            fn update(&mut self, conn: &::naphtha::DatabaseConnection<::naphtha::diesel::PgConnection>) -> ::anyhow::Result<()> {
                use ::naphtha::{diesel::SaveChangesDsl, log};
                let c = match conn.lock() {
                    Ok(c) => c,
                    Err(msg) => {
                        let message = format!(
                            "Could not aquire lock on DatabaseModifier::update for model:\nError: {}\nModel:\n{:#?}",
                            msg,
                            self
                            );
                        log::error!("{}", message);
                        return Err(::anyhow::anyhow!("{}", message));
                    }
                };
                self.pre_update(conn);
                match self.save_changes::<Self>(&*c) {
                    Ok(_) => (),
                    Err(msg) => {
                        let message = format!(
                            "Failed updating entity:\nError: {}\nModel:\n{:#?}",
                            msg,
                            self
                            );
                        log::error!("{}", message);
                        return Err(::anyhow::anyhow!("{}", message));
                    },
                };
                self.post_update(conn);
                Ok(())
            }

            fn remove(&mut self, conn: &::naphtha::DatabaseConnection<::naphtha::diesel::PgConnection>) -> ::anyhow::Result<()> {
                use {
                    ::naphtha::{log::{self, info}, DatabaseModel, diesel::{ExpressionMethods, RunQueryDsl, QueryDsl, Table}},
                    schema::{#table_name, #table_name::dsl::*},
                };
                let c = match conn.lock() {
                    Ok(c) => c,
                    Err(msg) => {
                        let message = format!(
                            "Could not aquire lock on DatabaseModifier::remove for model:\nError: {}\nModel:\n{:#?}",
                            msg,
                            self
                            );
                        log::error!("{}", message);
                        return Err(::anyhow::anyhow!("{}", message));
                    }
                };
                self.pre_remove(conn);
                let num_deleted = ::naphtha::diesel::delete(
                    #table_name.filter(
                        #table_name.primary_key().eq(self.primary_key())
                    )
                );
                match num_deleted.execute(&*c) {
                    Ok(_) => {
                        #[cfg(debug_assertions)]
                        info!("Removed entity with primary key {} from database!", self.primary_key());
                    },
                    Err(msg) => {
                        let message = format!(
                            "Could not remove model from database:\nError: {}\nModel:\n{:#?}",
                            msg,
                            self
                            );
                        log::error!("{}", message);
                        return Err(::anyhow::anyhow!("{}", message));
                    }
                };
                self.post_remove(conn);
                Ok(())
            }
        }
    }
}

fn generate_insert_properties(
    ast: &DeriveInput,
    params: &crate::params::Params,
) -> ::proc_macro2::TokenStream {
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
        if fieldname.to_string() == params.primary_key {
            // Primary must not be set during insertion.
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
    params: &crate::params::Params,
) -> ::proc_macro2::TokenStream {
    let name = &ast.ident;
    let table_name = ::proc_macro2::Ident::new(
        &params.table_name,
        ::proc_macro2::Span::call_site(),
    );

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
            _ => (quote! { Vec<Self> }, quote! { load }),
        };
        let (return_type, diesel_query_fn) =
            if fieldname.to_string() == params.primary_key {
                (quote! { Self }, quote! { first })
            } else {
                (return_type, diesel_query_fn)
            };
        let function_name = ::proc_macro2::Ident::new(
            &format!("query_by_{}", fieldname).to_lowercase(),
            ::proc_macro2::Span::call_site(),
        );
        let fieldtype = &field.ty;
        let query = quote! {
                fn #function_name(conn: &::naphtha::DatabaseConnection<::naphtha::diesel::PgConnection>, property: &#fieldtype)
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

    let query_by_primary_keys = impl_query_by_primary_keys(ast, params);

    quote! {
        impl QueryByProperties<::naphtha::diesel::PgConnection> for #name {
            type Error = ::naphtha::diesel::result::Error;
            #queries
            #query_by_primary_keys
        }
    }
}

fn impl_query_by_primary_keys(
    ast: &::syn::DeriveInput,
    params: &crate::params::Params,
) -> ::proc_macro2::TokenStream {
    let table_name = ::proc_macro2::Ident::new(
        &params.table_name,
        ::proc_macro2::Span::call_site(),
    );

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
        if fieldname.to_string() != params.primary_key {
            continue;
        }
        let fieldtype = &field.ty;
        let function_name = ::proc_macro2::Ident::new(
            &format!("query_by_{}s", fieldname).to_lowercase(),
            ::proc_macro2::Span::call_site(),
        );
        query = quote! {
                fn #function_name(conn: &::naphtha::DatabaseConnection<::naphtha::diesel::PgConnection>, primary_keys: &[#fieldtype])
                    -> ::naphtha::diesel::result::QueryResult<Vec<Self>> {
                    use {
                        schema::{#table_name, #table_name::dsl::*},
                        ::naphtha::diesel::{ExpressionMethods, QueryDsl, RunQueryDsl, Table},
                    };
                    conn.custom::<::naphtha::diesel::result::QueryResult<Vec<Self>>, _>(|c| {
                        #table_name.filter(#table_name.primary_key().eq_any(primary_keys)).load::<Self>(&*c)
                    })
                }
        };
        break;
    }

    query
}
