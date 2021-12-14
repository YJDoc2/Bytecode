use bytecode::Bytecode;
use seq_macro::seq;

seq!(N in 0..150{
    #[derive(Bytecode)]
    #[allow(non_camel_case_types)]
    pub enum cpu_bytecode {
        #(
            I#N,
        )*
    }
});

fn main() {}
