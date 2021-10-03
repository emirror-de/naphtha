extern crate proc_macro;
extern crate quote;

use {
    quote::quote,
    syn::{parse, DeriveInput},
};

#[cfg(any(feature = "barrel-sqlite", feature = "barrel-mysql"))]
mod barrel_impl;
mod database_impl;
mod database_traits;
#[allow(dead_code)]
mod helper;

#[proc_macro_attribute]
pub fn model(
    attr: ::proc_macro::TokenStream,
    item: ::proc_macro::TokenStream,
) -> ::proc_macro::TokenStream {
    let ast: DeriveInput = parse(item).expect(
        "proc_macro_attribute model: Could not parse TokenStream input!",
    );
    let attr = format!("#[{}]", attr);
    let attr: ::proc_macro2::TokenStream = attr.parse().unwrap();

    // QUERY BY PROPERTY TRAIT
    #[cfg(not(any(feature = "sqlite", feature = "mysql")))]
    let impl_trait_query_by_properties = quote! {};
    #[cfg(any(feature = "sqlite", feature = "mysql"))]
    let impl_trait_query_by_properties =
        database_traits::impl_trait_query_by_properties(&ast);

    // SQLITE
    #[cfg(not(feature = "sqlite"))]
    let impl_sqlite = quote! {};
    #[cfg(feature = "sqlite")]
    let impl_sqlite = database_impl::sqlite::impl_sqlite(&ast, &attr);

    #[cfg(not(feature = "barrel-sqlite"))]
    let impl_barrel_sqlite = quote! {};
    #[cfg(feature = "barrel-sqlite")]
    let impl_barrel_sqlite = barrel_impl::sqlite::impl_sqlite();

    // MYSQL
    #[cfg(not(feature = "mysql"))]
    let impl_mysql = quote! {};
    #[cfg(feature = "mysql")]
    let impl_mysql = database_impl::mysql::impl_mysql(&ast, &attr);

    #[cfg(not(feature = "barrel-mysql"))]
    let impl_barrel_mysql = quote! {};
    #[cfg(feature = "barrel-mysql")]
    let impl_barrel_mysql = barrel_impl::mysql::impl_mysql();

    let output = quote! {
        use schema::*;
        #[cfg(any(feature = "sqlite", feature = "mysql"))]
        use {
            ::diesel::{backend::Backend, prelude::*},
        };

        #[derive(Debug, Queryable, Identifiable, AsChangeset, Associations)]
        #attr
        #ast

        #impl_trait_query_by_properties

        #impl_sqlite
        #impl_barrel_sqlite

        #impl_mysql
        #impl_barrel_mysql
    };

    ::proc_macro::TokenStream::from(output)
}
