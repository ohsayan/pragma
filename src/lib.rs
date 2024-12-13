use {
    proc_macro::TokenStream,
    syn::{parse::Result as ParseResult, parse_macro_input},
};

mod grammar;
mod parse;

#[proc_macro]
pub fn pragma(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as parse::PragmaInput);
    let output = parse::process_pragma_input(input);
    output.into()
}
