
#![recursion_limit="128"]

extern crate proc_macro;

mod inspect;
mod optionize;

use crate::proc_macro::TokenStream;


#[proc_macro_derive(Inspect, attributes(inspect))]
pub fn inspect_macro_derive(input: TokenStream) -> TokenStream {
    // Construct a representation of Rust code as a syntax tree
    // that we can manipulate
    let ast = syn::parse(input).unwrap();

    // Build the trait implementation
    inspect::impl_inspect_macro(&ast)
}

#[proc_macro_derive(Optionize, attributes(optionize))]
pub fn optionize_macro_derive(input: TokenStream) -> TokenStream {
    // Construct a representation of Rust code as a syntax tree
    // that we can manipulate
    let ast = syn::parse(input).unwrap();

    // Build the trait implementation
    optionize::impl_optionize_macro(&ast)
}
