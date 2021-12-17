pub enum BytecodeError {
    EmptyInstruction,
    InvalidInstruction,
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
        return vec![*self];
    }
    fn parse(bytes: &[u8]) -> Result<(Self, usize), BytecodeError> {
        if bytes.len() < 1 {
            return Err(BytecodeError::InvalidInstruction);
        }
        Ok((bytes[0], 1))
    }
}

impl Bytecodable for u16 {
    fn compile(&self) -> Vec<u8> {
        let lower = *self as u8;
        let higher = (*self >> 8) as u8;
        return vec![lower, higher];
    }
    fn parse(bytes: &[u8]) -> Result<(Self, usize), BytecodeError> {
        if bytes.len() < 2 {
            return Err(BytecodeError::InvalidInstruction);
        }
        let lower = bytes[0] as u16;
        let higher = bytes[1] as u16;
        Ok((higher << 8 | lower, 2))
    }
}

impl Bytecodable for u32 {
    fn compile(&self) -> Vec<u8> {
        let lower1 = *self as u8;
        let lower2 = (*self >> 8) as u8;
        let higher1 = (*self >> 16) as u8;
        let higher2 = (*self >> 24) as u8;
        return vec![lower1, lower2, higher1, higher2];
    }
    fn parse(bytes: &[u8]) -> Result<(Self, usize), BytecodeError> {
        if bytes.len() < 4 {
            return Err(BytecodeError::InvalidInstruction);
        }
        let lower1 = bytes[0] as u32;
        let lower2 = (bytes[1] as u32) << 8;
        let higher1 = (bytes[2] as u32) << 16;
        let higher2 = (bytes[3] as u32) << 24;
        Ok((lower1 | lower2 | higher1 | higher2, 4))
    }
}

impl Bytecodable for u64 {
    fn compile(&self) -> Vec<u8> {
        let lower1 = *self as u8;
        let lower2 = (*self >> 8) as u8;
        let lower3 = (*self >> 16) as u8;
        let lower4 = (*self >> 24) as u8;
        let higher1 = (*self >> 32) as u8;
        let higher2 = (*self >> 40) as u8;
        let higher3 = (*self >> 48) as u8;
        let higher4 = (*self >> 56) as u8;
        return vec![
            lower1, lower2, lower3, lower4, higher1, higher2, higher3, higher4,
        ];
    }
    fn parse(bytes: &[u8]) -> Result<(Self, usize), BytecodeError> {
        if bytes.len() < 8 {
            return Err(BytecodeError::InvalidInstruction);
        }
        let lower1 = bytes[0] as u64;
        let lower2 = (bytes[1] as u64) << 8;
        let lower3 = (bytes[2] as u64) << 16;
        let lower4 = (bytes[3] as u64) << 24;
        let higher1 = (bytes[0] as u64) << 32;
        let higher2 = (bytes[1] as u64) << 40;
        let higher3 = (bytes[2] as u64) << 48;
        let higher4 = (bytes[3] as u64) << 56;
        Ok((
            lower1 | lower2 | lower3 | lower4 | higher1 | higher2 | higher3 | higher4,
            8,
        ))
    }
}

impl Bytecodable for i8 {
    fn compile(&self) -> Vec<u8> {
        (*self as u8).compile()
    }
    fn parse(bytes: &[u8]) -> Result<(Self, usize), BytecodeError> {
        u8::parse(bytes).map(|(x, s)| (x as i8, s))
    }
}

impl Bytecodable for i16 {
    fn compile(&self) -> Vec<u8> {
        (*self as u16).compile()
    }
    fn parse(bytes: &[u8]) -> Result<(Self, usize), BytecodeError> {
        u16::parse(bytes).map(|(x, s)| (x as i16, s))
    }
}

impl Bytecodable for i32 {
    fn compile(&self) -> Vec<u8> {
        (*self as u32).compile()
    }
    fn parse(bytes: &[u8]) -> Result<(Self, usize), BytecodeError> {
        u32::parse(bytes).map(|(x, s)| (x as i32, s))
    }
}

impl Bytecodable for i64 {
    fn compile(&self) -> Vec<u8> {
        (*self as u64).compile()
    }
    fn parse(bytes: &[u8]) -> Result<(Self, usize), BytecodeError> {
        u64::parse(bytes).map(|(x, s)| (x as i64, s))
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

// using usize should be avoided as the compiled bytecode will not be portable across arch
// fixed size type should be used instead, These implementations are given as to
// complete all numerical primitive types

#[cfg(target_pointer_width = "16")]
impl Bytecodable for usize {
    fn compile(&self) -> Vec<u8> {
        (*self as u16).compile()
    }
    // this is dicey, as this essentially mimics
    // C style truth/false value
    fn parse(bytes: &[u8]) -> Result<(Self, usize), BytecodeError> {
        u6::parse(bytes).map(|(x, s)| (x as usize, s))
    }
}

#[cfg(target_pointer_width = "32")]
impl Bytecodable for usize {
    fn compile(&self) -> Vec<u8> {
        (*self as u32).compile()
    }
    // this is dicey, as this essentially mimics
    // C style truth/false value
    fn parse(bytes: &[u8]) -> Result<(Self, usize), BytecodeError> {
        u32::parse(bytes).map(|(x, s)| (x as usize, s))
    }
}

#[cfg(target_pointer_width = "64")]
impl Bytecodable for usize {
    fn compile(&self) -> Vec<u8> {
        (*self as u64).compile()
    }
    // this is dicey, as this essentially mimics
    // C style truth/false value
    fn parse(bytes: &[u8]) -> Result<(Self, usize), BytecodeError> {
        u64::parse(bytes).map(|(x, s)| (x as usize, s))
    }
}
