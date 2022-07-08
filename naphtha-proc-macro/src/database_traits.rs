use quote::quote;

pub fn impl_trait_query_by_properties(
    ast: &::syn::DeriveInput,
    params: &crate::params::Params,
) -> ::proc_macro2::TokenStream {
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
        let return_type = match &fieldname.to_string()[..] {
            "updated_at" => continue,
            _ => quote! { Vec<Self> },
        };
        let return_type = if &fieldname.to_string()[..] == params.primary_key {
            quote! { Self }
        } else {
            return_type
        };

        let function_name = ::proc_macro2::Ident::new(
            &format!("query_by_{}", fieldname).to_lowercase(),
            ::proc_macro2::Span::call_site(),
        );
        let fieldtype = &field.ty;
        let query = quote! {
                /// Queries the database with by the given property. It only returns
                /// those with an exact match.
                fn #function_name(conn: &::naphtha::DatabaseConnection<DB>, property: &#fieldtype)
                    -> Result<#return_type, Self::Error>;
        };
        queries = quote! {
            #queries
            #query
        };
    }

    let query_by_primary_keys = impl_trait_query_by_primary_keys(ast, params);

    quote! {
        /// Queries the model by the given property. Returns only those with an
        /// exact match.
        pub trait QueryByProperties<DB>
            where
                Self: Sized
        {
            /// The error type for this implementation.
            type Error;
            #queries
            #query_by_primary_keys
        }
    }
}

fn impl_trait_query_by_primary_keys(
    ast: &::syn::DeriveInput,
    params: &crate::params::Params,
) -> ::proc_macro2::TokenStream {
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
        if &fieldname.to_string()[..] != params.primary_key {
            continue;
        }
        let fieldtype = &field.ty;
        let function_name = ::proc_macro2::Ident::new(
            &format!("query_by_{}s", &params.primary_key.to_lowercase()),
            ::proc_macro2::Span::call_site(),
        );
        query = quote! {
                /// Queries the database by the given #fieldname.
                fn #function_name(conn: &::naphtha::DatabaseConnection<DB>, primary_keys: &[#fieldtype])
                    -> Result<Vec<Self>, Self::Error>;
        };
        break;
    }

    query
}
