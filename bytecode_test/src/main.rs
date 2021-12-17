use bytecode::*;
use seq_macro::seq;

#[derive(Bytecode)]
pub enum Reg {
    A,
    B,
    C,
}

seq!(N in 0..250{
    #[derive(Bytecode)]
    #[allow(non_camel_case_types)]
    pub enum cpu_bytecode {
        #(I#N(u8, u8,Reg),)*
    }
});

fn main() {}
