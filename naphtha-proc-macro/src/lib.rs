extern crate proc_macro;
extern crate quote;

use {
    quote::quote,
    syn::{parse, DeriveInput},
};

#[cfg(any(feature = "barrel-full", feature = "barrel-sqlite",))]
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
    #[cfg(not(any(feature = "full", feature = "sqlite")))]
    let impl_trait_query_by_properties = quote! {};
    #[cfg(any(feature = "full", feature = "sqlite"))]
    let impl_trait_query_by_properties =
        database_traits::impl_trait_query_by_properties(&ast);

    // SQLITE
    #[cfg(not(feature = "sqlite"))]
    let impl_sqlite = quote! {};
    #[cfg(feature = "sqlite")]
    let impl_sqlite = database_impl::sqlite::impl_sqlite(&ast, &attr);

    #[cfg(not(any(feature = "barrel-full", feature = "barrel-sqlite")))]
    let impl_barrel_sqlite = quote! {};
    #[cfg(any(feature = "barrel-full", feature = "barrel-sqlite"))]
    let impl_barrel_sqlite = barrel_impl::sqlite::impl_sqlite();

    // MYSQL
    let impl_mysql = if cfg!(feature = "mysql") {
        database_impl::mysql::impl_mysql(&ast, &attr)
    } else {
        quote! {}
    };

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
    };

    ::proc_macro::TokenStream::from(output)
}
