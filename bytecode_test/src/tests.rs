use bytecode::{Bytecodable, Bytecode, BytecodeError};
use seq_macro::seq;

#[derive(Bytecode, Debug, PartialEq, Eq)]
enum SimpleEnum {
    T0,
    T1(u8, u8),
    T2 { x: u8, y: u8 },
}

#[derive(Bytecode, Debug, PartialEq, Eq)]
enum CompositeEnum {
    T0,
    T1(SimpleEnum),
    T2 { x: SimpleEnum, y: u8 },
}

seq!(N in 0..128{
#[derive(Bytecode, Debug, PartialEq, Eq)]
enum TestBoundaryFromBelow{
    #(T~N(u8,u8),)*
}
});

seq!(N in 0..150{
#[derive(Bytecode, Debug, PartialEq, Eq)]
enum TestBoundaryFromAbove{
    #(T~N(u8,u8),)*
}
});

#[derive(Bytecode, Debug, PartialEq, Eq)]
struct SimpleTupleStruct(u8, u16, i16);

#[derive(Bytecode, Debug, PartialEq, Eq)]
struct CompositeTupleStruct(SimpleEnum, u16);

#[derive(Bytecode, Debug, PartialEq, Eq)]
struct SimpleNamedStruct {
    x: u16,
    y: u32,
    z: u8,
}
#[derive(Bytecode, Debug, PartialEq, Eq)]
struct CompositeNamedStruct {
    x: SimpleNamedStruct,
    y: u16,
    z: SimpleEnum,
}

#[test]
fn test_simple_enum_compile() {
    let t1 = SimpleEnum::T0;
    let t2 = SimpleEnum::T1(5, 10);
    let t3 = SimpleEnum::T2 { x: 7, y: 14 };

    assert_eq!(&t1.compile(), &[0]);
    assert_eq!(&t2.compile(), &[1, 5, 10]);
    assert_eq!(&t3.compile(), &[2, 7, 14]);
}

#[test]
fn test_simple_enum_parse() {
    let t1 = [0];
    let t2 = [1, 10, 5];
    let t3 = [2, 14, 7];

    // should give invalid instruction
    let err1 = [3, 5];
    // should give incomplete instruction
    let err2 = [1, 2];

    let t1 = SimpleEnum::parse(&t1);
    assert_eq!(t1, Ok((SimpleEnum::T0, 1)));

    let t2 = SimpleEnum::parse(&t2);
    assert_eq!(t2, Ok((SimpleEnum::T1(10, 5), 3)));

    let t3 = SimpleEnum::parse(&t3);
    assert_eq!(t3, Ok((SimpleEnum::T2 { x: 14, y: 7 }, 3)));

    let err1 = SimpleEnum::parse(&err1);
    assert_eq!(err1, Err(BytecodeError::InvalidInstruction));

    let err2 = SimpleEnum::parse(&err2);
    assert_eq!(err2, Err(BytecodeError::IncompleteInstruction));

    let err3 = SimpleEnum::parse(&[]);
    assert_eq!(err3, Err(BytecodeError::IncompleteInstruction));
}

#[test]
fn test_composite_enum_compile() {
    let t1 = CompositeEnum::T0;
    let t2 = CompositeEnum::T1(SimpleEnum::T1(5, 7));
    let t3 = CompositeEnum::T2 {
        x: SimpleEnum::T2 { x: 7, y: 5 },
        y: 12,
    };

    assert_eq!(t1.compile(), &[0]);

    assert_eq!(t2.compile(), &[1, 1, 5, 7]);

    assert_eq!(t3.compile(), &[2, 2, 7, 5, 12]);
}

#[test]
fn test_composite_enum_parse() {
    let t1 = [0];
    let t2 = [1, 2, 7, 5];
    let t3 = [2, 1, 8, 16, 2];

    let t1 = CompositeEnum::parse(&t1);
    assert_eq!(t1, Ok((CompositeEnum::T0, 1)));

    let t2 = CompositeEnum::parse(&t2);
    assert_eq!(
        t2,
        Ok((CompositeEnum::T1(SimpleEnum::T2 { x: 7, y: 5 }), 4))
    );

    let t3 = CompositeEnum::parse(&t3);
    assert_eq!(
        t3,
        Ok((
            CompositeEnum::T2 {
                x: SimpleEnum::T1(8, 16),
                y: 2
            },
            5
        ))
    );

    // should give invalid instruction
    let err1 = [5, 8, 7];
    // should give incomplete instruction
    let err2 = [1];
    // should give incomplete instruction
    let err3 = [1, 2];
    // should give incomplete instruction
    let err4 = [2, 2, 5, 7];

    let err1 = CompositeEnum::parse(&err1);
    assert_eq!(err1, Err(BytecodeError::InvalidInstruction));

    let err2 = CompositeEnum::parse(&err2);
    assert_eq!(err2, Err(BytecodeError::IncompleteInstruction));

    let err3 = CompositeEnum::parse(&err3);
    assert_eq!(err3, Err(BytecodeError::IncompleteInstruction));

    let err4 = CompositeEnum::parse(&err4);
    assert_eq!(err4, Err(BytecodeError::IncompleteInstruction));
}

#[test]
fn test_enum_boundary() {
    let t1 = TestBoundaryFromBelow::T0(5, 7);
    let t2 = TestBoundaryFromBelow::T126(7, 5);
    let t3 = TestBoundaryFromBelow::T127(7, 5);

    assert_eq!(t1.compile(), &[0, 5, 7]);
    assert_eq!(t2.compile(), &[126, 7, 5]);
    assert_eq!(t3.compile(), &[127, 7, 5]);

    let t1 = TestBoundaryFromAbove::T0(0, 0);
    let t2 = TestBoundaryFromAbove::T126(8, 50);
    let t3 = TestBoundaryFromAbove::T127(50, 80);
    let t4 = TestBoundaryFromAbove::T128(55, 77);
    let t5 = TestBoundaryFromAbove::T129(77, 55);

    assert_eq!(t1.compile(), &[0, 0, 0]);
    assert_eq!(t2.compile(), &[126, 8, 50]);
    assert_eq!(t3.compile(), &[127, 50, 80]);
    assert_eq!(t4.compile(), &[128, 128, 55, 77]);
    assert_eq!(t5.compile(), &[128, 129, 77, 55]);

    let r1 = [0, 5, 7];
    let r2 = [126, 7, 5];
    let r3 = [127, 5, 5];

    assert_eq!(
        TestBoundaryFromBelow::parse(&r1),
        Ok((TestBoundaryFromBelow::T0(5, 7), 3))
    );

    assert_eq!(
        TestBoundaryFromBelow::parse(&r2),
        Ok((TestBoundaryFromBelow::T126(7, 5), 3))
    );

    assert_eq!(
        TestBoundaryFromBelow::parse(&r3),
        Ok((TestBoundaryFromBelow::T127(5, 5), 3))
    );

    let err1 = [128, 5, 5];
    assert_eq!(
        TestBoundaryFromBelow::parse(&err1),
        Err(BytecodeError::InvalidInstruction)
    );

    let err2 = [150, 5, 5];
    assert_eq!(
        TestBoundaryFromBelow::parse(&err2),
        Err(BytecodeError::InvalidInstruction)
    );

    let r1 = [0, 8, 55];
    let r2 = [126, 12, 15];
    let r3 = [127, 8, 7];
    let r4 = [128, 128, 2, 3];
    let r5 = [128, 129, 5, 7];
    let r6 = [128, 149, 5, 7];
    assert_eq!(
        TestBoundaryFromAbove::parse(&r1),
        Ok((TestBoundaryFromAbove::T0(8, 55), 3))
    );
    assert_eq!(
        TestBoundaryFromAbove::parse(&r2),
        Ok((TestBoundaryFromAbove::T126(12, 15), 3))
    );
    assert_eq!(
        TestBoundaryFromAbove::parse(&r3),
        Ok((TestBoundaryFromAbove::T127(8, 7), 3))
    );
    assert_eq!(
        TestBoundaryFromAbove::parse(&r4),
        Ok((TestBoundaryFromAbove::T128(2, 3), 4))
    );
    assert_eq!(
        TestBoundaryFromAbove::parse(&r5),
        Ok((TestBoundaryFromAbove::T129(5, 7), 4))
    );
    assert_eq!(
        TestBoundaryFromAbove::parse(&r6),
        Ok((TestBoundaryFromAbove::T149(5, 7), 4))
    );

    // incomplete instruction
    let err1 = [127];
    // incomplete instruction
    let err2 = [128];
    // incomplete instruction
    let err3 = [128, 128, 3];
    // invalid instruction
    let err4 = [128, 151, 0, 0];
    // invalid instruction
    let err5 = [128, 155, 0, 0];
    assert_eq!(
        TestBoundaryFromAbove::parse(&err1),
        Err(BytecodeError::IncompleteInstruction)
    );
    assert_eq!(
        TestBoundaryFromAbove::parse(&err2),
        Err(BytecodeError::IncompleteInstruction)
    );
    assert_eq!(
        TestBoundaryFromAbove::parse(&err3),
        Err(BytecodeError::IncompleteInstruction)
    );
    assert_eq!(
        TestBoundaryFromAbove::parse(&err4),
        Err(BytecodeError::InvalidInstruction)
    );
    assert_eq!(
        TestBoundaryFromAbove::parse(&err5),
        Err(BytecodeError::InvalidInstruction)
    );
}

#[test]
fn test_tuple_struct_compile() {
    let t1 = SimpleTupleStruct(5, 0x1234, -568);
    let lower = -568_i16 as u16 as u8;
    let higher = ((-568_i16 as u16) >> 8) as u8;
    assert_eq!(t1.compile(), &[5, 0x34, 0x12, lower, higher]);

    let t2 = CompositeTupleStruct(SimpleEnum::T1(57, 78), 555);
    let lower = 555_u16 as u8;
    let higher = (555 >> 8) as u8;
    assert_eq!(t2.compile(), &[1, 57, 78, lower, higher]);
}

#[test]
fn test_tuple_struct_parse() {
    let r1 = [12, 58, 0, 0x77, 0x55];
    let t1 = SimpleTupleStruct(12, 58, 0x5577);
    assert_eq!(SimpleTupleStruct::parse(&r1), Ok((t1, 5)));

    let err1 = [12, 5, 7, 3];
    assert_eq!(
        SimpleTupleStruct::parse(&err1),
        Err(BytecodeError::IncompleteInstruction)
    );

    let r2 = [2, 97, 95, 72, 0];
    let t2 = CompositeTupleStruct(SimpleEnum::T2 { x: 97, y: 95 }, 72);
    assert_eq!(CompositeTupleStruct::parse(&r2), Ok((t2, 5)));

    let err2 = [1, 5];
    assert_eq!(
        CompositeTupleStruct::parse(&err2),
        Err(BytecodeError::IncompleteInstruction)
    );
    let err3 = [7, 5];
    assert_eq!(
        CompositeTupleStruct::parse(&err3),
        Err(BytecodeError::InvalidInstruction)
    );
}

#[test]
fn test_named_struct_compile() {
    let t1 = SimpleNamedStruct {
        x: 0xABCD,
        y: 0x55ABCDEF,
        z: 0xAB,
    };
    assert_eq!(t1.compile(), &[0xCD, 0xAB, 0xEF, 0xCD, 0xAB, 0x55, 0xAB]);

    let t2 = CompositeNamedStruct {
        x: SimpleNamedStruct {
            x: 0x0012,
            y: 0x00120034,
            z: 0x01,
        },
        y: 0x0102,
        z: SimpleEnum::T1(5, 7),
    };
    assert_eq!(
        t2.compile(),
        &[0x12, 0, 0x34, 0, 0x12, 0, 0x1, 0x2, 0x1, 1, 5, 7]
    );
}

#[test]
fn test_named_struct_parse() {
    let r1 = [0x2, 0x1, 0x4, 0x3, 0x2, 0x1, 0x1];
    let t1 = SimpleNamedStruct {
        x: 0x102,
        y: 0x1020304,
        z: 0x1,
    };
    assert_eq!(SimpleNamedStruct::parse(&r1), Ok((t1, 7)));

    let err1 = [0x1, 0x1, 0x1, 0x1];
    assert_eq!(
        SimpleNamedStruct::parse(&err1),
        Err(BytecodeError::IncompleteInstruction)
    );

    let r2 = [0, 0x12, 0, 0, 0x34, 0x12, 0, 0x55, 0x77, 2, 7, 5];
    let t2 = CompositeNamedStruct {
        x: SimpleNamedStruct {
            x: 0x1200,
            y: 0x12340000,
            z: 0,
        },
        y: 0x07755,
        z: SimpleEnum::T2 { x: 7, y: 5 },
    };
    assert_eq!(CompositeNamedStruct::parse(&r2), Ok((t2, 12)));

    let err1 = [0, 0x12, 0, 0, 0x34, 0x12, 0, 0x55, 0x77, 2, 7];
    assert_eq!(
        CompositeNamedStruct::parse(&err1),
        Err(BytecodeError::IncompleteInstruction)
    );

    let err2 = [0, 0x12, 0, 0, 0x34, 0x12, 0, 0x55, 0x77, 3, 7];
    assert_eq!(
        CompositeNamedStruct::parse(&err2),
        Err(BytecodeError::InvalidInstruction)
    );
}
