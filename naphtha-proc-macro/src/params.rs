/// The input parameter of the attribute.
#[derive(Debug)]
pub struct Params {
    pub table_name: String,
    pub primary_key: String,
}

impl Params {
    const TABLE_NAME: &'static str = "table_name";
    const PRIMARY_KEY: &'static str = "primary_key";
}

#[derive(PartialEq, Eq)]
enum ParseState {
    Identifier,
    Punct,
    Literal,
}

impl From<::proc_macro::TokenStream> for Params {
    fn from(attr: ::proc_macro::TokenStream) -> Self {
        use proc_macro::TokenTree::*;
        let mut table_name = None;
        let mut primary_key = None;

        // The expected token
        let mut parse_state = ParseState::Identifier;
        // the current identifier (name of the parameter)
        let mut current_ident: Option<::proc_macro::Ident> = None;

        for a in attr.into_iter() {
            match a {
                Ident(ident) => {
                    if parse_state != ParseState::Identifier {
                        panic!("Syntax error in parsing Params struct. Expected identifier!");
                    }
                    current_ident = Some(ident);
                    parse_state = ParseState::Punct;
                }
                Punct(punct) => {
                    if parse_state != ParseState::Punct {
                        panic!("Syntax error in parsing Params struct. Expected Punctuation like , or =");
                    }
                    parse_state = match &punct.as_char().to_string()[..] {
                        "=" => ParseState::Literal,
                        "," => ParseState::Identifier,
                        _ => panic!("Syntax error. Unknown punctuation given: {}\nPossible values are: , or =", punct.as_char()),
                    };
                }
                Literal(literal) => {
                    if parse_state != ParseState::Literal {
                        panic!("Syntax error in parsing Params struct. Expected Punctuation like , or =");
                    }
                    let literal = literal.to_string();
                    match current_ident {
                        Some(i) => {
                            match &i.to_string()[..] {
                                Self::TABLE_NAME => table_name = Some(literal.replace('\"', "")),
                                Self::PRIMARY_KEY => primary_key = Some(literal.replace('\"', "")),
                                _ => panic!("Unknown parameter '{}' given!", i),
                            }
                        }
                        None => panic!("Syntax error in parsing Params struct. Identifier expected."),
                    }
                    current_ident = None;
                    parse_state = ParseState::Punct;
                }
                _ => continue,
            }
        }

        let table_name = if let Some(t) = table_name {
            t
        } else {
            panic!("Missing parameter table_name. Please add it to the model attribute, e.g. table_name = \"my_new_table\"!");
        };

        let primary_key = if let Some(k) = primary_key {
            k
        } else {
            panic!("Missing parameter primary_key. Please add it to the model attribute, e.g. primary_key = \"id\"!");
        };

        Params {
            table_name,
            primary_key,
        }
    }
}
