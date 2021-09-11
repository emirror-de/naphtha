use quote::quote;

pub fn impl_trait_query_by_properties(
    ast: &::syn::DeriveInput,
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
                /// Queries the database with by the given property. It only returns
                /// those with an exact match.
                fn #function_name(conn: &DatabaseConnection<DB>, property: &#fieldtype)
                    -> Result<#return_type, Self::Error>;
        };
        queries = quote! {
            #queries
            #query
        };
    }

    let query_by_ids = if crate::helper::has_id(ast) {
        impl_trait_query_by_ids(ast)
    } else {
        quote! {}
    };

    quote! {
        /// Queries the model by the given property. Returns only those with an
        /// exact match.
        pub trait QueryByProperties<DB>
            where
                Self: Sized
        {
            type Error;
            #queries
            #query_by_ids
        }
    }
}

fn impl_trait_query_by_ids(
    ast: &::syn::DeriveInput,
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
        match &fieldname.to_string()[..] {
            "id" => (),
            _ => continue,
        };
        let fieldtype = &field.ty;
        query = quote! {
                fn query_by_ids(conn: &DatabaseConnection<DB>, ids: &[#fieldtype])
                    -> Result<Vec<Self>, Self::Error>;
        };
        break;
    }

    query
}
