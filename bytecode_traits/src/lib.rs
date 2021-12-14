pub enum BytecodeError {
    Other(&'static str),
    InvalidInstruction,
}

pub trait Bytecodable: Sized {
    fn compile(&self) -> Vec<u8>;
    fn parse(bytes: &[u8]) -> Result<Self, BytecodeError>;
}
