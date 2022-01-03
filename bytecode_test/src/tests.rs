use bytecode::{Bytecodable, Bytecode, BytecodeError};

#[derive(Bytecode, Debug, PartialEq, Eq)]
enum TestBasicEnum {
    T1,
    T2(u8, u8),
    T3 { x: u8, y: u8 },
}

#[derive(Bytecode, Debug, PartialEq, Eq)]
enum TestCompositeEnum {
    T1,
    T2(TestBasicEnum),
    T3 { x: TestBasicEnum, y: u8 },
}

#[test]
fn test_basic_enum_compile() {
    let t1 = TestBasicEnum::T1;
    let t2 = TestBasicEnum::T2(5, 10);
    let t3 = TestBasicEnum::T3 { x: 7, y: 14 };

    assert_eq!(&t1.compile(), &[0]);
    assert_eq!(&t2.compile(), &[1, 5, 10]);
    assert_eq!(&t3.compile(), &[2, 7, 14]);
}

#[test]
fn test_basic_enum_parse() {
    let t1 = [0];
    let t2 = [1, 10, 5];
    let t3 = [2, 14, 7];

    // should give invalid instruction
    let err1 = [3, 5];
    // should give incomplete instruction
    let err2 = [1, 2];

    let t1 = TestBasicEnum::parse(&t1);
    assert_eq!(t1, Ok((TestBasicEnum::T1, 1)));

    let t2 = TestBasicEnum::parse(&t2);
    assert_eq!(t2, Ok((TestBasicEnum::T2(10, 5), 3)));

    let t3 = TestBasicEnum::parse(&t3);
    assert_eq!(t3, Ok((TestBasicEnum::T3 { x: 14, y: 7 }, 3)));

    let err1 = TestBasicEnum::parse(&err1);
    assert_eq!(err1, Err(BytecodeError::InvalidInstruction));

    let err2 = TestBasicEnum::parse(&err2);
    assert_eq!(err2, Err(BytecodeError::InvalidInstruction));

    let err3 = TestBasicEnum::parse(&[]);
    assert_eq!(err3, Err(BytecodeError::EmptyInstruction));
}

#[test]
fn test_composite_enum_compile() {
    let t1 = TestCompositeEnum::T1;
    let t2 = TestCompositeEnum::T2(TestBasicEnum::T2(5, 7));
    let t3 = TestCompositeEnum::T3 {
        x: TestBasicEnum::T3 { x: 7, y: 5 },
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

    let t1 = TestCompositeEnum::parse(&t1);
    assert_eq!(t1, Ok((TestCompositeEnum::T1, 1)));

    let t2 = TestCompositeEnum::parse(&t2);
    assert_eq!(
        t2,
        Ok((TestCompositeEnum::T2(TestBasicEnum::T3 { x: 7, y: 5 }), 4))
    );

    let t3 = TestCompositeEnum::parse(&t3);
    assert_eq!(
        t3,
        Ok((
            TestCompositeEnum::T3 {
                x: TestBasicEnum::T2(8, 16),
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

    let err1 = TestCompositeEnum::parse(&err1);
    assert_eq!(err1, Err(BytecodeError::InvalidInstruction));

    let err2 = TestCompositeEnum::parse(&err2);
    assert_eq!(err2, Err(BytecodeError::EmptyInstruction));

    let err3 = TestCompositeEnum::parse(&err3);
    assert_eq!(err3, Err(BytecodeError::InvalidInstruction));

    let err4 = TestCompositeEnum::parse(&err4);
    assert_eq!(err4, Err(BytecodeError::InvalidInstruction));
}
