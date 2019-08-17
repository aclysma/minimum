extern crate proc_macro;

mod inspect;

use crate::proc_macro::TokenStream;

#[proc_macro_derive(Inspect, attributes(inspect, inspect_slider))]
pub fn inspect_macro_derive(input: TokenStream) -> TokenStream {
    inspect::impl_inspect_macro(input)
}
