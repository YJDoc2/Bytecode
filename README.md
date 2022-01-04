# Bytecode

A simple way to derive bytecode for you Enums and Structs.

## What is this

This is a crate that provides a proc macro which will derive bytecode representation of your enums and structs, and provides compile and parse functions to convert to and from the bytecode. This also provides necessary traits to do so, in case you want to do it manually.

Note : The values of the fields are compiled as little-endian values, so in the bytecode the smallest byte is at smallest location. The bytecode itself is Big-endian for certain reasons.

## Example

Cargo.toml

```toml
...
[dependencies]
...
bytecode = {git = "https://github.com/YJDoc2/Bytecode" }
...

```

Code

```rust
use bytecode::{Bytecodable, Bytecode};

#[derive(Bytecode, Debug, PartialEq, Eq)]
pub enum Register {
    AX,
    BX,
    CX,
    DX,
}

#[derive(Bytecode, Debug, PartialEq, Eq)]
pub struct Mem {
    segment: Register,
    offset: Register,
    imOffset: u16,
}

#[derive(Bytecode, Debug, PartialEq, Eq)]
pub enum Opcode {
    Hlt,
    Nop,
    Add(Register, Register),
    AddI(Register, u16),
    AddM(Register, Mem),
}

fn main() {
    let op1 = Opcode::AddI(Register::AX, 57);
    let compiled = op1.compile();

    // This is for the example,
    // actually you might use parse on a already compiled
    // values, to parse them back into the enum variant
    let bytes = [4, 2, 1, 3, 0x75, 0x00];
    let op2 = Opcode::parse(&bytes);
    let op2_test = Opcode::AddM(
        Register::CX,
        Mem {
            segment: Register::BX,
            offset: Register::DX,
            imOffset: 0x0075,
        },
    );
    assert!(op2 == Ok((op2_test, 6)));
}


```

## Bytecodable Trait

This crate also exposes the `bytecodable` trait, which is used to implement the compile and parse functions. In case it is required, you can implement this by yourself for your own structs/enums.

```rust
pub trait Bytecodable{
    /// This function compiles the value to its bytecode representation.
    /// It should return a Vec<u8> containing the bytecode of the value.
    fn compile(&self)->Vec<u8>;

    /// This function parses a u8 slice reference to the value, and
    /// returns a result :
    /// Ok with the parsed value and how many bytes were consumed to parse it
    /// Err with the BytecodeError
    fn parse(&[u8])-> Result<(Self,usize),BytecodeError>;
}
```

## Use case

##### This shows _why_ and _when_ you would use this. To see _how_ to use this, see the example section.

Consider that you are writing a VM, or an interpreter, and you want to have a opcode-like representation of your instructions. A good way to do this would be using an Enum to represent the instructions, with the parameters of the instruction as the enum fields. So it would be something like this :

```rust

pub enum Register{
    AX,
    BX,
    ...
}

pub enum Opcode{
    Nop,
    Hlt,
    Add(Register,Register),
    AddI(Register,u16),
    ...
}
```

Now that you have done the representation, you can directly use it as an IR, where you target your language to this Enum variants, and the iterate over the list, and taking actions as per the variant.

But if you are making a low level emulator, where you need to store the opcodes as values in memory, or you want to store this representation as a compiled file, you will need to make a bytecode representation of this Enum. In the simplest way, this would mean assigning a value to each enum variant, and to each of enum field, and write functions to manually compile the enum values to `u8` array, and parse from `u8` array back to enum field. It would be something like this

```rust
impl Register{
    ...
    fn compile(&self)->Vec<u8>{
        match self{
            Register::AX => vec![0],
            Register::BX => vec![1],
            ...
        }
    }
    fn parse(bytes:&[u8])->Result<Self,&str>{
        match bytes[0]{
            1 => Ok(Register::AX),
            2 => Ok(Register::BX),
            ...
            _ => Err("Invalid opcode")
        }
    }
    ...
}

impl Opcode{
    ...
    fn compile(&self)->Vec<u8>{
        match self{
            Opcode::Nop => vec![0],
            Opcode::Hlt => vec![1],
            Opcode::Add(r1,r2) => {
                let mut v = Vec::with_capacity(2);
                v.extend(&r1.compile());
                v.extend(&r2.compile());
                v
            }
            Opcode::AddI(r1,v1) =>{
                let mut v = Vec::with_capacity(3);
                v.extend(&r1.compile());
                v.extend(&v1.to_le_bytes());
                v
            }
            ...
        }
    }
    fn parse(bytes:&[u8])->Result<Self,&str>{
        match bytes[0]{
            1 => Ok(Opcode::Nop),
            2 => Ok(Opcode::Hlt),
            3 =>{
                let r1 = Register::parse(&bytes[1..])?;
                ...
            }
            ...
            _ => Err("Invalid opcode")
        }
    }
}
```

Now consider doing this for even 25-ish opcodes, which is roughly the minimum amount you might need for a small instruction set. For a more complex instruction set, you will need to do this for about 100+ instructions, and then for each field value of individual variant, like `Register` in this example. This will get tedious, potentially error prone, and quite boring, moving focus from building a VM / interpreter which is your original intention to writing these functions.

The Bytecode macro will derive these functions for you, for Enum and Structs, without having you to manually do anything.

Also imagine trying to remove a variant from middle in the manual implementation :-| You will either :

- need to remove that variant from parse and compile functions and shift rest of them accordingly
- keep the variant in the functions, but never output in the IR building, making one dead spot
- remove the variant, but keep the values in compile and parse function same, treating the value of removed variant as dead spot

None of these is particularly good, and thus delegating this manual work to a proc-macro makes that much more sense!

## Why did I make this

- The primary reason behind making this was to move the IR of my [8086 emulator](https://github.com/YJDoc2/8086-Emulator/) from Text-based to opcode based. Currently the emulator uses a Text-based IR, which works and gives about 90-95 % of 8086 functionality, but still I would like to make is as similar to an actual 8086 hardware emulator as possible. One of the improvements needed for this is to make the IR bytecode-based, which can be stored in the VM's memory. I didn't want to make the bytecode for about 100 odd instruction opcodes, so instead I made this!
- I wanted to experiment with proc-macros for sometime, and this seemed a good opportunity to try it.
- In the long term, I want to make a general Hardware Emulation framework, which will allow connecting different emulated hardware components together, eg: connecting my 8086 emulator with someone's DMA controller. That will not only allow making hardware emulators a bit more approachable, but it will also allow developing modular emulators (at least that's what I think). I feel having such a macro to easily write the bytecode of different devices will be a step in that direction.

## Restrictions

This macro has some restrictions :

- For enums, there can be at most 1 << 15 = 32768 variants. In case you need more variants than this, this cannot help you.
- By default this provides Bytecodable implementation for bool and all `u_` and `i_` numeric types except `usize` and `isize`. In case you need some other types as enum fields, you need to make sure that they also implements the Bytecodable trait, wither using the derive macro or manually (like the `Register` enum in the example).
- This crate does not necessarily create an efficient representation of the values. For example

  ```rust
  enum Register{
      AX,
      BX,
      CX,
      DX,
  }
  enum Opcode{
      ...
      Add(Register,Register),
      ...
  }
  ```

  Ideally the `Add` instruction can be compiled into two bytes :

  - 1 byte for denoting the `Add` variant of `Opcode` Enum
  - 1 byte whose higher 4 bits denote first register, and lower 4 bits denote second register.
    This can be done, as we know that `Register` enum will have at max 4 values, and thus its bytecode representation will have at most thevalue 3, which can be represented in 4 bits.

  This bytecode macro will not create this efficient representation, but will instead allocate 1 byte for each register field, thus taking 3 bytes total for `Add` Opcode Variant.
  In case you need this kind of compression, the only way to do it currently is to use u8 instead of two Register i.e. `Add(u8)` and then manually extract and code the Registers in the `u8`.

- This macro will not allow to manually set value of the bytecode for a specific field or variant. Currently The bytecode starts at `0` for first variant of any enum, and goes on from there. But this is not a guarantee, and ideally the bytecode generated should be treated as a black box and should be interacted only with the `compile` and `parse` functions (_Ideally_) . That said currently there is not way to set specific value for a specific variant, and the values will be allocated as generated. In case you need a specific value for specific opcode, such as `0` for `Hlt` (because if you are storing the compiled bytecode in VM's memory, treating 0 as Hlt is a good idea to stop random, potentially endless execution), then you will need to make sure `Hlt` is the first variant in the enum.

---

## License

Licensed under either of

- Apache License, Version 2.0
  ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license
  ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
