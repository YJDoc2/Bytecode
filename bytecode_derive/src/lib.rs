mod util;
use proc_macro::TokenStream;
use proc_macro2::Ident;
use quote::quote;
use std::collections::BTreeMap;
use syn::{parse_macro_input, DeriveInput};

// we use at max 2 bytes for instructions, and thus 1 bit is reserved to show length of instruction
// we might need to add more constants in future to indicate this, rather than hardcoding directly
const MAX_VARIANTS_ALLOWED: usize = 1 << 15;

// TODO add better error reporting

#[proc_macro_derive(Bytecode)]
pub fn derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let input_enum = if let syn::Data::Enum(ref s) = input.data {
        s
    } else {
        panic!("Bytecode derive only supported for enums");
    };

    let name = input.ident;

    let max_possible_instructions = input_enum.variants.len();

    if max_possible_instructions > MAX_VARIANTS_ALLOWED {
        panic!(
            "Currently at max {} enum variants are supported, found {}",
            MAX_VARIANTS_ALLOWED,
            input_enum.variants.len()
        );
    }

    let compiled = input_enum.variants.iter().enumerate().map(|(i, v)| {
        let ident = &v.ident;
        let v = util::compile_instr(i);
        quote! {
            #name::#ident => {#v}
        }
    });

    let parse_fn_param_name = Ident::new("bytes", name.span());
    let parse_check_logic = if max_possible_instructions <= 1 << 7 {
        let _max = max_possible_instructions as u8;
        quote! {
            if #parse_fn_param_name[0] > #_max{
                return std::result::Result::Err(bytecode_trait::BytecodeError::InvalidInstruction);
            }
        }
    } else {
        let _max = max_possible_instructions as u16;
        quote! {
            if #parse_fn_param_name[0] > 1<< 7 && #parse_fn_param_name.len() < 2{
                return std::result::Result::Err(bytecode_trait::BytecodeError::InvalidInstruction);
            }
            if #parse_fn_param_name[0] > 1<< 7{
                let higher_byte:u8 = #parse_fn_param_name[0] & (1<<7 -1);
                let lower_byte:u8 = #parse_fn_param_name[1];
                let instr:u16 = (higher_byte as u16) << 8 | lower_byte as u16;
                if instr > #_max {
                    return std::result::Result::Err(bytecode_trait::BytecodeError::InvalidInstruction);
                }
            }
        }
    };

    // this handles only single byte length instruction
    // parsing logic, the two-byte instruction parsing logic
    // is in separate variable
    let mut single_byte_parse_logic = Vec::new();
    for i in 0..max_possible_instructions {
        if i < 1 << 7 {
            let var = &input_enum.variants[i];
            let i = i as u8;

            single_byte_parse_logic.push(quote! {
                #i => {
                    return std::result::Result::Ok(#name::#var);
                }
            });
        } else {
            // we're beyond the single byte length as per encoding scheme
            // so we can stop the loop
            break;
        }
    }
    let single_byte_parse_logic = single_byte_parse_logic.iter();
    let mut two_byte_parse_logic = Vec::new();
    if max_possible_instructions > 1 << 7 {
        // we can use hashmap here, but then the order of output is dependent on
        // how the higher byte is hashed, BTreeMap allows the inter to be in order of numerical
        // values, so the output is ordered, and easier to debug
        let mut hm: BTreeMap<u8, Vec<_>> = BTreeMap::new();
        for i in 1 << 7..max_possible_instructions {
            let [higher_byte, lower_byte] = util::split_into_bytes(i);
            let entry = hm.entry(higher_byte).or_default();
            entry.push((lower_byte, &input_enum.variants[i]));
        }
        for (higher_byte, list) in hm.into_iter() {
            let lower_byte_matches = list.into_iter().map(|(lower_byte, v)| {
                quote! {
                    #lower_byte =>{return std::result::Result::Ok(#name::#v)}
                }
            });
            two_byte_parse_logic.push(quote! {
                #higher_byte => {
                    match #parse_fn_param_name[1] {
                        #(#lower_byte_matches),*
                        _ => unreachable!()
                    }
                }
            })
        }
    }
    let two_byte_parse_logic = two_byte_parse_logic.iter();

    let output = quote! {
        impl bytecode_trait::Bytecodable for #name{
            fn compile(&self)->Vec<u8>{
                match self {
                    #(#compiled ),*
                }
            }

            fn parse(#parse_fn_param_name:&[u8])->std::result::Result<#name,bytecode_trait::BytecodeError>{
                if #parse_fn_param_name.len() < 1 {
                    return std::result::Result::Err(bytecode_trait::BytecodeError::Other("Slice length must be atleast 1"));
                }
                #parse_check_logic

                match #parse_fn_param_name[0]{
                    #(#single_byte_parse_logic ),*
                    #(#two_byte_parse_logic),*
                    _ => {unreachable!();}
                }
            }
        }
    };
    output.into()
}
