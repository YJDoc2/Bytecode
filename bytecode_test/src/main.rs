use bytecode::*;
use seq_macro::seq;

#[derive(Bytecode)]
pub enum Reg {
    A,
    B,
    C,
}

seq!(N in 0..150 {
    #[derive(Bytecode)]
    #[allow(non_camel_case_types)]
    pub enum cpu_bytecode {
        #(I~N { x: u8, y: u8 },)*
    }
});

// #[derive(Bytecode)]
// #[allow(non_camel_case_types)]
// struct bytes {
//     x: u8,
//     y: Reg,
//     z: u8,
// }

fn main() {}
