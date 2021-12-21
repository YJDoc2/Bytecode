use proc_macro::TokenStream;
use proc_macro2::Ident;
use quote::quote;

pub fn derive_struct(name: &syn::Ident, input_struct: &syn::DataStruct) -> TokenStream {
    let fields = &input_struct.fields;
    match fields {
        syn::Fields::Unit => panic!("Bytecode is not supported for Unit type structs"),
        syn::Fields::Unnamed(unnamed) => {
            let fields = &unnamed.unnamed;
            let parse_fn_param_name = syn::Ident::new("__bytes", name.span());

            let field_list = fields
                .iter()
                .enumerate()
                .map(|(i, _)| Ident::new(&format!("v{}", i), name.span()));
            let fields_compiled = fields.iter().enumerate().map(|(i, f)| {
                let idx = syn::Index::from(i);
                quote! {
                    self.#idx.compile()
                }
            });
            let field_list_bracketed = quote! {
                (#(#field_list),*)
            };
            let compiled = quote! {
                let mut _i = std::vec::Vec::new();
                #(_i.extend(&#fields_compiled);)*
                return _i;
            };

            let output = quote! {
                impl bytecode_trait::Bytecodable for #name{
                    fn compile(&self)->Vec<u8>{
                        #compiled
                    }

                    fn parse(#parse_fn_param_name:&[u8])->std::result::Result<(#name,usize),bytecode_trait::BytecodeError>{
                        unimplemented!();
                        // if #parse_fn_param_name.len() < 1 {
                        //     return std::result::Result::Err(bytecode_trait::BytecodeError::EmptyInstruction);
                        // }
                        // #parse_check_logic

                        // match #parse_fn_param_name[0]{
                        //     #(#single_byte_parse_logic ),*
                        //     #(#two_byte_parse_logic),*
                        //     _ => {unreachable!();}
                        // }
                    }
                }
            };
            output.into()
        }
        _ => unimplemented!(),
    }
}
