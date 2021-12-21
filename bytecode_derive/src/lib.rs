mod enum_derive;
mod internal;
mod struct_derive;
mod util;
use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(Bytecode)]
pub fn derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    internal::derive(input)
}
