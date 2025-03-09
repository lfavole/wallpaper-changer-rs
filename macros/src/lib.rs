//! Code for the [`compile_env!`] macro.
use dotenv::dotenv;
use proc_macro::{TokenStream, TokenTree};
use quote::quote;
use std::env;

/// Returns an environment variable in the `.env` file or in the environment.
///
/// # Panics
/// Panics if there are no arguments or more than two arguments.
#[proc_macro]
pub fn compile_env(input: TokenStream) -> TokenStream {
    let tokens: Vec<_> = input.into_iter().collect();

    let (name, default): (String, String) = match tokens.as_slice() {
        [TokenTree::Literal(lit)] => (unwrap_string_literal(lit), String::new()),
        [
            TokenTree::Literal(lit),
            TokenTree::Punct(punct),
            TokenTree::Literal(default_lit),
        ] if punct.as_char() == ',' => {
            (unwrap_string_literal(lit), unwrap_string_literal(default_lit))
        },
        _ => panic!("This macro only accepts one or two arguments"),
    };

    // Read the .env file
    let _ = dotenv();

    let value = env::var(name).unwrap_or_else(|_| default.to_string());
    quote!(#value).into()
}

/// Gets a [`String`] from a [`proc_macro::Literal`].
///
/// Inspired from <https://docs.rs/include_dir_macros/0.7.4/src/include_dir_macros/lib.rs.html#31>.
///
/// # Panics
/// Panics if the [`proc_macro::Literal`] doesn't contain a single string argument.
fn unwrap_string_literal(lit: &proc_macro::Literal) -> String {
    let mut repr = lit.to_string();
    assert!(
        repr.starts_with('"') && repr.ends_with('"'),
        "This macro only accepts a single, non-empty string argument"
    );
    repr.remove(0);
    repr.pop();
    repr
}
