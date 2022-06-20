//! This crate is a support crate that contains the necessary macros for naphtha to compile.

extern crate proc_macro;
extern crate quote;

use {
    quote::quote,
    syn::{parse, DeriveInput},
};

#[cfg(any(
    feature = "barrel-sqlite",
    feature = "barrel-mysql",
    feature = "barrel-pg"
))]
mod barrel_impl;
mod database_impl;
mod database_traits;
#[allow(dead_code)]
mod params;

#[proc_macro_attribute]
pub fn model(
    attr: ::proc_macro::TokenStream,
    item: ::proc_macro::TokenStream,
) -> ::proc_macro::TokenStream {
    let ast: DeriveInput = parse(item).expect(
        "proc_macro_attribute model: Could not parse TokenStream input!",
    );

    let params = params::Params::from(attr.clone());
    let attribute_table_name =
        format!("#[table_name = \"{}\"]", params.table_name);
    let attribute_table_name: ::proc_macro2::TokenStream =
        attribute_table_name.parse().unwrap();
    let attribute_primary_key =
        format!("#[primary_key({})]", params.primary_key);
    let attribute_primary_key: ::proc_macro2::TokenStream =
        attribute_primary_key.parse().unwrap();

    // QUERY BY PROPERTY TRAIT
    #[cfg(not(any(feature = "sqlite", feature = "mysql", feature = "pg")))]
    let impl_trait_query_by_properties = quote! {};
    #[cfg(any(feature = "sqlite", feature = "mysql", feature = "pg"))]
    let impl_trait_query_by_properties =
        database_traits::impl_trait_query_by_properties(&ast, &params);

    // SQLITE
    #[cfg(not(feature = "sqlite"))]
    let impl_sqlite = quote! {};
    #[cfg(feature = "sqlite")]
    let impl_sqlite = database_impl::sqlite::impl_sqlite(&ast, &params);

    #[cfg(not(feature = "barrel-sqlite"))]
    let impl_barrel_sqlite = quote! {};
    #[cfg(feature = "barrel-sqlite")]
    let impl_barrel_sqlite = barrel_impl::sqlite::impl_sqlite(&ast);

    // MYSQL
    #[cfg(not(feature = "mysql"))]
    let impl_mysql = quote! {};
    #[cfg(feature = "mysql")]
    let impl_mysql = database_impl::mysql::impl_mysql(&ast, &attr);

    #[cfg(not(feature = "barrel-mysql"))]
    let impl_barrel_mysql = quote! {};
    #[cfg(feature = "barrel-mysql")]
    let impl_barrel_mysql = barrel_impl::mysql::impl_mysql(&ast);

    // PostgreSQL
    #[cfg(not(feature = "pg"))]
    let impl_pg = quote! {};
    #[cfg(feature = "pg")]
    let impl_pg = database_impl::pg::impl_pg(&ast, &attr);
    #[cfg(not(feature = "barrel-pg"))]
    let impl_barrel_pg = quote! {};
    #[cfg(feature = "barrel-pg")]
    let impl_barrel_pg = barrel_impl::pg::impl_pg(&ast);

    let output = quote! {
        use self::schema::*;
        #[cfg(any(feature = "sqlite", feature = "mysql", feature = "pg"))]
        use {
            ::naphtha::diesel::{backend::Backend, prelude::*},
        };

        #[derive(
            Debug,
            Queryable,
            Identifiable,
            AsChangeset,
            Associations
            )]
        #attribute_table_name
        #attribute_primary_key
        #ast

        #impl_trait_query_by_properties

        #impl_sqlite
        #impl_barrel_sqlite

        #impl_mysql
        #impl_barrel_mysql

        #impl_pg
        #impl_barrel_pg
    };

    ::proc_macro::TokenStream::from(output)
}
