use crate::enum_derive::derive_enum;
use crate::struct_derive::derive_struct;
use proc_macro::TokenStream;

// TODO add better error reporting, instead of manual panics use macro derive errors

pub fn derive(input: syn::DeriveInput) -> TokenStream {
    match input.data {
        syn::Data::Enum(ref input_enum) => derive_enum(&input.ident, input_enum),
        syn::Data::Struct(ref input_struct) => derive_struct(&input.ident, input_struct),
        syn::Data::Union(_) => panic!("Bytecode is not supported for Unions"),
    }
}
