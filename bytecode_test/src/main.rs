use bytecode::*;
use seq_macro::seq;

#[derive(Bytecode)]
pub enum Reg {
    A,
    B,
    C,
}

// #[derive(Bytecode)]
// #[allow(non_camel_case_types)]
// pub enum cpu_bytecode {
//     I0,
//     I1(u8),
//     I2 { x: u8, y: u8 },
// }

#[derive(Bytecode)]
#[allow(non_camel_case_types)]
struct bytes(u8, Reg, u8);

fn main() {}
