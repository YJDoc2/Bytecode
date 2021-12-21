use proc_macro::TokenStream;
use proc_macro2::Ident;
use quote::quote;

pub fn derive_struct(name: &syn::Ident, input_struct: &syn::DataStruct) -> TokenStream {
    let fields = &input_struct.fields;
    match fields {
        syn::Fields::Unit => panic!("Bytecode is not supported for Unit type structs"),
        syn::Fields::Unnamed(unnamed) => derive_unnamed(name, unnamed),
        _ => unimplemented!(),
    }
}

fn derive_unnamed(
    struct_name: &syn::Ident,
    fields: &syn::FieldsUnnamed,
) -> proc_macro::TokenStream {
    let fields = &fields.unnamed;
    let parse_fn_param_name = syn::Ident::new("__bytes", struct_name.span());

    let field_list = fields
        .iter()
        .enumerate()
        .map(|(i, _)| Ident::new(&format!("v{}", i), struct_name.span()));
    let fields_compiled = fields.iter().enumerate().map(|(i, _)| {
        let idx = syn::Index::from(i);
        quote! {
            self.#idx.compile()
        }
    });

    let compiled = quote! {
        let mut _i = std::vec::Vec::new();
        #(_i.extend(&#fields_compiled);)*
        return _i;
    };

    let size_counter_var = syn::Ident::new("_count", struct_name.span());

    let parsed = fields.iter().zip(field_list.clone()).map(|(f, v)| {
        quote! {
            let (#v,size) = #f::parse(&#parse_fn_param_name[#size_counter_var..])?;
            #size_counter_var += size;
        }
    });

    let field_list_bracketed = quote! {
        (#(#field_list),*)
    };

    let output = quote! {
        impl bytecode_trait::Bytecodable for #struct_name{
            fn compile(&self)->Vec<u8>{
                #compiled
            }

            fn parse(#parse_fn_param_name:&[u8])->std::result::Result<(#struct_name,usize),bytecode_trait::BytecodeError>{
                let mut #size_counter_var = 0;
                #(#parsed)*
                std::result::Result::Ok((Self #field_list_bracketed,#size_counter_var))
            }
        }
    };
    output.into()
}
