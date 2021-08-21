extern crate proc_macro;
extern crate quote;

use {
    quote::quote,
    syn::{parse, DeriveInput},
};

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
    let output = quote! {
        use schema::*;
        #[derive(Debug, Clone, Queryable, Identifiable, AsChangeset, Associations)]
        #attr
        #ast
    };
    ::proc_macro::TokenStream::from(output)
}
