extern crate proc_macro;
extern crate quote;

use {
    quote::quote,
    syn::{parse, DeriveInput},
};

mod database_impl;
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

    #[cfg(feature = "sqlite")]
    let impl_sqlite = database_impl::sqlite::impl_sqlite(&ast, &attr);

    let output = quote! {
        use schema::*;
        #[derive(Debug, Clone, Queryable, Identifiable, AsChangeset, Associations)]
        #attr
        #ast
        #[cfg(feature = "sqlite")]
        #impl_sqlite
    };

    ::proc_macro::TokenStream::from(output)
}
