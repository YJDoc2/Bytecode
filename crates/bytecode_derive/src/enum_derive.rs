use crate::util;
use proc_macro::TokenStream;
use proc_macro2::Ident;
use quote::quote;
use std::collections::BTreeMap;

// we use at max 2 bytes for instructions, and thus 1 bit is reserved to show length of instruction
// we might need to add more constants in future to indicate this, rather than hardcoding directly
const MAX_VARIANTS_ALLOWED: usize = 1 << 15;

pub fn derive_enum(name: &syn::Ident, input_enum: &syn::DataEnum) -> TokenStream {
    // maximum instructions possible for this enum
    // maybe change this name a bit
    let max_possible_instructions = input_enum.variants.len();

    // make sure possible instructions are less than allowed
    if max_possible_instructions > MAX_VARIANTS_ALLOWED {
        panic!(
            "Currently at max {} enum variants are supported, found {}",
            MAX_VARIANTS_ALLOWED,
            input_enum.variants.len()
        );
    }

    // this maps the variants into the code for the compile method
    // for each individual enum variant
    // we later iter over this to fill the body of compile method impl
    let compiled = input_enum
        .variants
        .iter()
        .enumerate()
        .map(|(i, v)| compile_enum_variant(name, i, v));

    // we extract this here, as multiple places need this name
    let parse_fn_param_name = Ident::new("__bytes", name.span());

    // this generates code for initial check for validity of the byte stream
    // in the parse method impl.
    // If this check fails, we can quickly exit the compile method
    // with appropriate error
    // <=, as max_possible_instruction is total number of enum variants, which start from 1
    let parse_check_logic = if max_possible_instructions <= 1 << 7 {
        // If the number of enum variants are less than 1<<7, all bytecodes generated
        // will be 1 byte length, so we have to only check the first byte
        let _max = max_possible_instructions as u8;
        // >= max, as counting starts at 0
        quote! {
            if #parse_fn_param_name[0] >= #_max{
                return std::result::Result::Err(bytecode::BytecodeError::InvalidInstruction);
            }
        }
    } else {
        // If the total variants are greater that 1<<7. we need to check first as well as second byte
        // If the 0th byte of bytestream is less than 1<<7, then it is definitely valid bytecode,
        // as 2-byte bytecode only begin after 1<<7 bytecode
        // if the 0th byte is greater than 1<<7, then there must be at least two bytes
        // and the u16 formed by both of them must be less than max possible instructions
        let _max = max_possible_instructions as u16;
        quote! {
            if #parse_fn_param_name[0] >= 1<< 7 && #parse_fn_param_name.len() < 2{
                return std::result::Result::Err(bytecode::BytecodeError::IncompleteInstruction);
            }
            if #parse_fn_param_name[0] >= 1<< 7{
                let higher_byte:u8 = #parse_fn_param_name[0] & (1<<7 -1);
                let lower_byte:u8 = #parse_fn_param_name[1];
                let instr:u16 = (higher_byte as u16) << 8 | lower_byte as u16;
                if instr > #_max {
                    return std::result::Result::Err(bytecode::BytecodeError::InvalidInstruction);
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
            let variant = &input_enum.variants[i];
            let i = i as u8;
            let parsed = parse_single_byte(name, &parse_fn_param_name, variant);
            // add conditional parse logic depending on unit type or unnamed type
            single_byte_parse_logic.push(quote! {
                #i => {
                    #parsed
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
    // if number of instructions i s less than 1<<7 then this should be empty
    if max_possible_instructions > 1 << 7 {
        // we can use std::hashmap here, but then the order of output is dependent on
        // how the higher byte is hashed, BTreeMap allows the inter to be in order of numerical
        // values, so the output is ordered, and easier to debug
        let mut hm: BTreeMap<u8, Vec<_>> = BTreeMap::new();

        // fist we split the two-byte instructions into two bytes
        // as in the byte array the higher bytes will be first, we must
        // first match on it, and then match on the next byte
        // so here we map the pair of lower byte and corresponding enum variant to
        // corresponding higher byte
        for i in 1 << 7..max_possible_instructions {
            let [higher_byte, lower_byte] = util::split_into_instr_bytes(i);
            let entry = hm.entry(higher_byte).or_default();
            entry.push((lower_byte, &input_enum.variants[i]));
        }
        // then here we get the code for parsing the individual variant, and
        // then generate the code for parsing the complete two bytes
        for (higher_byte, list) in hm.into_iter() {
            let lower_byte_matches = list.into_iter().map(|(lower_byte, var)| {
                parse_two_byte(name, &parse_fn_param_name, lower_byte, var)
            });
            two_byte_parse_logic.push(quote! {
                #higher_byte => {
                    match #parse_fn_param_name[1] {
                        #(#lower_byte_matches),*
                        _ => unreachable!()
                    }
                }
            });
        }
    }
    let two_byte_parse_logic = two_byte_parse_logic.iter();

    let output = quote! {
        impl bytecode::Bytecodable for #name{
            fn compile(&self)->Vec<u8>{
                match self {
                    #(#compiled ),*
                }
            }

            fn parse(#parse_fn_param_name:&[u8])->std::result::Result<(#name,usize),bytecode::BytecodeError>{
                if #parse_fn_param_name.len() < 1 {
                    return std::result::Result::Err(bytecode::BytecodeError::IncompleteInstruction);
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

// helper function which returns code for compiling
// individual enum variant
fn compile_enum_variant(name: &syn::Ident, i: usize, v: &syn::Variant) -> proc_macro2::TokenStream {
    let ident = &v.ident;
    let instr = util::compile_instr(i);
    match &v.fields {
        syn::Fields::Unit => {
            quote! {
                #name::#ident => {#instr}
            }
        }
        syn::Fields::Unnamed(fields) => {
            let field_list = fields
                .unnamed
                .iter()
                .enumerate()
                .map(|(i, _)| Ident::new(&format!("v{}", i), ident.span()));
            let fields_compiled = field_list.clone().map(|f| {
                quote! {
                    #f.compile()
                }
            });
            let field_list_bracketed = quote! {
                (#(#field_list),*)
            };
            quote! {
                #name::#ident #field_list_bracketed => {
                    let mut _i = #instr;
                    #(_i.extend(&#fields_compiled);)*
                    return _i;
                }
            }
        }
        syn::Fields::Named(named) => {
            let fields = &named.named;
            let field_names = fields.iter().map(|f| f.ident.as_ref().unwrap());
            let fields_compiled = fields.iter().map(|f| {
                let ident = &f.ident;
                quote! {
                    #ident.compile()
                }
            });

            let compiled = quote! {
                let mut _i = #instr;
                #(_i.extend(&#fields_compiled);)*
                return _i;
            };
            quote! {
                #name::#ident {#(#field_names),* } => {
                    #compiled
                }
            }
        }
    }
}

// helper function which returns the code for parsing
// single byte long instruction
fn parse_single_byte(
    enum_name: &syn::Ident,
    param_name: &syn::Ident,
    variant: &syn::Variant,
) -> proc_macro2::TokenStream {
    let variant_name = &variant.ident;
    match &variant.fields {
        syn::Fields::Unit => parse_unit_field(enum_name, variant_name, 1),
        syn::Fields::Unnamed(fields) => {
            parse_unnamed_field(fields, enum_name, variant_name, param_name, 1)
        }
        syn::Fields::Named(fields) => parse_named_field(fields, enum_name, variant_name, 1),
    }
}

// helper function which returns code for parsing
// two byte instruction length variants
fn parse_two_byte(
    enum_name: &syn::Ident,
    param_name: &syn::Ident,
    lower_byte: u8,
    variant: &syn::Variant,
) -> proc_macro2::TokenStream {
    let variant_name = &variant.ident;
    let parsed = match &variant.fields {
        syn::Fields::Unit => parse_unit_field(enum_name, variant_name, 2),
        syn::Fields::Unnamed(fields) => {
            parse_unnamed_field(fields, enum_name, variant_name, param_name, 2)
        }
        syn::Fields::Named(fields) => parse_named_field(fields, enum_name, variant_name, 2),
    };
    quote! {
        #lower_byte =>{#parsed}
    }
}

fn parse_unit_field(
    enum_name: &syn::Ident,
    variant: &syn::Ident,
    init_size: usize,
) -> proc_macro2::TokenStream {
    quote! {
        return std::result::Result::Ok((#enum_name::#variant,#init_size));
    }
}

fn parse_unnamed_field(
    fields: &syn::FieldsUnnamed,
    enum_name: &syn::Ident,
    variant: &syn::Ident,
    param_name: &syn::Ident,
    init_size: usize,
) -> proc_macro2::TokenStream {
    let size_counter_var = Ident::new("_count", variant.span());
    let field_list = fields
        .unnamed
        .iter()
        .enumerate()
        .map(|(i, _)| Ident::new(&format!("v{}", i), variant.span()));
    let field_parsed_list = fields.unnamed.iter().zip(field_list.clone()).map(|(f, v)| {
        quote! {
            let (#v,size) = #f::parse(&#param_name[#size_counter_var..])?;
            #size_counter_var += size;
        }
    });
    let field_list_bracketed = quote! {
        #enum_name::#variant(#(#field_list),*)
    };
    quote! {
        let mut #size_counter_var :usize = #init_size;
        #(#field_parsed_list)*
        return std::result::Result::Ok((#field_list_bracketed,#size_counter_var));
    }
}

fn parse_named_field(
    fields: &syn::FieldsNamed,
    enum_name: &syn::Ident,
    variant: &syn::Ident,
    init_size: usize,
) -> proc_macro2::TokenStream {
    let fields = &fields.named;

    let size_counter_var = syn::Ident::new("_count", enum_name.span());
    let parse_fn_param_name = syn::Ident::new("__bytes", enum_name.span());

    let parsed = fields.iter().map(|f| {
        let ident = f.ident.as_ref().unwrap();
        let ty = &f.ty;
        quote! {
            let (#ident,size) = #ty::parse(&#parse_fn_param_name[#size_counter_var..])?;
            #size_counter_var += size;
        }
    });

    let field_name_list = fields.iter().map(|f| f.ident.as_ref().unwrap());

    quote! {
        let mut #size_counter_var = #init_size;
        #(#parsed)*
        let _t = #enum_name::#variant{
            #(#field_name_list),*
        };
        std::result::Result::Ok((_t,#size_counter_var))

    }
}
