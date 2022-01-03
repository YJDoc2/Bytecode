use std::convert::TryFrom;
#[derive(Debug, PartialEq, Eq, Hash)]
/// This specifies the errors that might occur in parsing the bytecode
pub enum BytecodeError {
    /// The instruction is invalid, that is no instruction compiles to this specific instruction byte(s)
    InvalidInstruction,
    /// The instruction is incomplete, more bytes are expected to complete the parsing
    IncompleteInstruction,
    /// Some other error
    Other(&'static str),
}

pub trait Bytecodable: Sized {
    fn compile(&self) -> Vec<u8>;
    fn parse(bytes: &[u8]) -> Result<(Self, usize), BytecodeError>;
}

// Note all primitive follow little enadian encoding
// shouldn't be a big problem, as when using Bytecodable,
// users aren't meant to manually work with the bytecode,
// instead treat it as blackbox, and use the trait as interface

impl Bytecodable for u8 {
    fn compile(&self) -> Vec<u8> {
        Vec::from(self.to_le_bytes())
    }
    fn parse(bytes: &[u8]) -> Result<(Self, usize), BytecodeError> {
        #[allow(clippy::len_zero)]
        if bytes.len() < 1 {
            return Err(BytecodeError::IncompleteInstruction);
        }
        let bytes: [u8; 1] = [bytes[0]];
        Ok((u8::from_le_bytes(bytes), 1))
    }
}

impl Bytecodable for u16 {
    fn compile(&self) -> Vec<u8> {
        Vec::from(self.to_le_bytes())
    }
    fn parse(bytes: &[u8]) -> Result<(Self, usize), BytecodeError> {
        if bytes.len() < 2 {
            return Err(BytecodeError::IncompleteInstruction);
        }
        let bytes: [u8; 2] = <[u8; 2]>::try_from(&bytes[0..2]).unwrap();
        Ok((u16::from_le_bytes(bytes), 2))
    }
}

impl Bytecodable for u32 {
    fn compile(&self) -> Vec<u8> {
        Vec::from(self.to_le_bytes())
    }
    fn parse(bytes: &[u8]) -> Result<(Self, usize), BytecodeError> {
        if bytes.len() < 4 {
            return Err(BytecodeError::IncompleteInstruction);
        }
        let bytes: [u8; 4] = <[u8; 4]>::try_from(&bytes[0..4]).unwrap();
        Ok((u32::from_le_bytes(bytes), 4))
    }
}

impl Bytecodable for u64 {
    fn compile(&self) -> Vec<u8> {
        Vec::from(self.to_le_bytes())
    }
    fn parse(bytes: &[u8]) -> Result<(Self, usize), BytecodeError> {
        if bytes.len() < 8 {
            return Err(BytecodeError::IncompleteInstruction);
        }
        let bytes: [u8; 8] = <[u8; 8]>::try_from(&bytes[0..8]).unwrap();
        Ok((u64::from_le_bytes(bytes), 8))
    }
}

impl Bytecodable for i8 {
    fn compile(&self) -> Vec<u8> {
        Vec::from(self.to_le_bytes())
    }
    fn parse(bytes: &[u8]) -> Result<(Self, usize), BytecodeError> {
        #[allow(clippy::len_zero)]
        if bytes.len() < 1 {
            return Err(BytecodeError::IncompleteInstruction);
        }
        let bytes: [u8; 1] = [bytes[0]];
        Ok((i8::from_le_bytes(bytes), 1))
    }
}

impl Bytecodable for i16 {
    fn compile(&self) -> Vec<u8> {
        Vec::from(self.to_le_bytes())
    }
    fn parse(bytes: &[u8]) -> Result<(Self, usize), BytecodeError> {
        if bytes.len() < 2 {
            return Err(BytecodeError::IncompleteInstruction);
        }
        let bytes: [u8; 2] = <[u8; 2]>::try_from(&bytes[0..2]).unwrap();
        Ok((i16::from_le_bytes(bytes), 2))
    }
}

impl Bytecodable for i32 {
    fn compile(&self) -> Vec<u8> {
        Vec::from(self.to_le_bytes())
    }
    fn parse(bytes: &[u8]) -> Result<(Self, usize), BytecodeError> {
        if bytes.len() < 4 {
            return Err(BytecodeError::IncompleteInstruction);
        }
        let bytes: [u8; 4] = <[u8; 4]>::try_from(&bytes[0..4]).unwrap();
        Ok((i32::from_le_bytes(bytes), 4))
    }
}

impl Bytecodable for i64 {
    fn compile(&self) -> Vec<u8> {
        Vec::from(self.to_le_bytes())
    }
    fn parse(bytes: &[u8]) -> Result<(Self, usize), BytecodeError> {
        if bytes.len() < 8 {
            return Err(BytecodeError::IncompleteInstruction);
        }
        let bytes: [u8; 8] = <[u8; 8]>::try_from(&bytes[0..8]).unwrap();
        Ok((i64::from_le_bytes(bytes), 8))
    }
}

impl Bytecodable for bool {
    fn compile(&self) -> Vec<u8> {
        (*self as u8).compile()
    }
    // this is dicey, as this essentially mimics
    // C style truth/false value
    fn parse(bytes: &[u8]) -> Result<(Self, usize), BytecodeError> {
        u8::parse(bytes).map(|(x, s)| (x != 0, s))
    }
}
