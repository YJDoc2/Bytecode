use quote::__private::TokenStream;
use quote::quote;

pub fn compile_instr(v: usize) -> TokenStream {
    if v < 1 << 7 {
        // if the iter number is less than 1<<7, we can use it as it is
        let v = v as u8;
        quote! {
            vec![#v]
        }
    } else {
        // else we need to split is in two u8s
        // as we restrict number of variants to << 15, we can directly take
        // higher byte and lower byte of u16
        // just need to set the MSB for the first byte to indicate that this is
        // a 2 byte spanning instruction
        let [higher_byte, lower_byte] = split_into_instr_bytes(v);
        quote! {
            vec![#higher_byte , #lower_byte]
        }
    }
}

pub fn split_into_instr_bytes(v: usize) -> [u8; 2] {
    let higher_byte = ((v >> 8) as u8) | 1 << 7;
    let lower_byte = v as u8 & u8::MAX;
    [higher_byte, lower_byte]
}
