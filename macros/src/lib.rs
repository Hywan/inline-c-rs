use proc_macro2::TokenStream;
use quote::quote;

#[proc_macro]
pub fn assert_c(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = TokenStream::from(input);
    let input_as_string = reconstruct(input);

    quote!(
        inline_c::run(inline_c::Language::C, #input_as_string).map_err(|e| panic!(e.to_string())).unwrap()
    )
    .into()
}

#[proc_macro]
pub fn assert_cxx(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = TokenStream::from(input);
    let input_as_string = reconstruct(input);

    quote!(
        inline_c::run(inline_c::Language::CXX, #input_as_string).map_err(|e| panic!(e.to_string())).unwrap()
    )
    .into()
}

fn reconstruct(input: TokenStream) -> String {
    use proc_macro2::{Delimiter, Spacing, TokenTree::*};

    let mut output = String::new();
    let mut iterator = input.into_iter().peekable();

    loop {
        match iterator.next() {
            Some(Punct(token)) => {
                let token_value = token.as_char();

                match token_value {
                    '#' => {
                        output.push('\n');
                        output.push(token_value);

                        match iterator.peek() {
                            Some(Ident(include))
                                if include.to_string() == "include".to_string() =>
                            {
                                iterator.next();

                                let opening;
                                let closing;

                                match iterator.next() {
                                    Some(Punct(punct)) => {
                                        opening = punct.as_char();

                                        if opening == '"' {
                                            closing = '"';
                                        } else {
                                            closing = '>';
                                        }
                                    }

                                    Some(token) => panic!(
                                        "Invalid opening token after `#include`, received `{:?}`.",
                                        token
                                    ),

                                    None => panic!("`#include` must be followed by `<` or `\"`."),
                                }

                                output.push_str("include");
                                output.push(' ');
                                output.push(opening);

                                loop {
                                    match iterator.next() {
                                        Some(Punct(punct)) => {
                                            let punct = punct.as_char();

                                            if punct == closing {
                                                break;
                                            }

                                            output.push(punct)
                                        }
                                        Some(Ident(ident)) => output.push_str(&ident.to_string()),
                                        token => panic!(
                                            "Invalid token in `#include` value, with `{:?}`.",
                                            token
                                        ),
                                    }
                                }

                                output.push(closing);
                                output.push('\n');
                            }

                            _ => (),
                        }
                    }

                    ';' => {
                        output.push(token_value);
                        output.push('\n');
                    }

                    _ => {
                        output.push(token_value);

                        if token.spacing() == Spacing::Alone {
                            output.push(' ');
                        }
                    }
                }
            }

            Some(Ident(ident)) => {
                output.push_str(&ident.to_string());
                output.push(' ');
            }

            Some(Group(group)) => {
                let group_output = reconstruct(group.stream());

                match group.delimiter() {
                    Delimiter::Parenthesis => {
                        output.push('(');
                        output.push_str(&group_output);
                        output.push(')');
                    }

                    Delimiter::Brace => {
                        output.push('{');
                        output.push('\n');
                        output.push_str(&group_output);
                        output.push('\n');
                        output.push('}');
                    }

                    Delimiter::Bracket => {
                        output.push('[');
                        output.push_str(&group_output);
                        output.push(']');
                    }

                    Delimiter::None => {
                        output.push_str(&group_output);
                    }
                }
            }

            Some(token) => {
                output.push_str(&token.to_string());
            }

            None => break,
        }
    }

    output
}
