use syn::{Data::Struct, DeriveInput, Ident};

pub fn extract_table_name(attr: &::proc_macro2::TokenStream) -> Ident {
    for t in attr.clone().into_iter() {
        match t {
            ::proc_macro2::TokenTree::Group(g) => {
                let mut is_next = false;
                for t in g.stream().into_iter() {
                    match t {
                        ::proc_macro2::TokenTree::Ident(i) => {
                            let ident = ::proc_macro2::Ident::new(
                                "table_name",
                                ::proc_macro2::Span::call_site(),
                            );
                            is_next = ident == i;
                        }
                        ::proc_macro2::TokenTree::Literal(l) => {
                            if is_next {
                                let l = l.to_string();
                                return Ident::new(
                                    &l[1..l.len() - 1],
                                    ::proc_macro2::Span::call_site(),
                                );
                            }
                        }
                        _ => continue,
                    }
                }
            }
            _ => continue,
        }
    }

    panic!("Attribute table_name has not been found in {}", attr);
}

pub fn has_id(ast: &DeriveInput) -> bool {
    let data = match &ast.data {
        Struct(data) => data,
        _ => {
            return false;
        }
    };
    for field in data.fields.iter() {
        if field.ident.is_none() {
            continue;
        }
        let fieldname = field.ident.as_ref().unwrap();
        match &fieldname.to_string()[..] {
            "id" => return true,
            _ => continue,
        };
    }

    false
}
