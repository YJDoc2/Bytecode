use bytecode::Bytecode;
use seq_macro::seq;

#[derive(Bytecode)]
pub enum Reg {
    A,
    B,
    C,
}

#[derive(Bytecode)]
#[allow(non_camel_case_types)]
pub enum cpu_bytecode {
    I0,
    I1(u8, Reg, u8),
    I2(bytes, u8),
    I3 { x: bytes, y: u8, z: Reg },
}

#[derive(Bytecode)]
#[allow(non_camel_case_types)]
pub struct bytes {
    x: u8,
    y: Reg,
    z: u8,
}

fn main() {}
