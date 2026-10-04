use super::cpu::*;
use super::instruction::Instruction;

#[test]
fn decode_add() {
    assert_eq!(
        Instruction::decode(&[0x20, 1, 2, 3]).unwrap(),
        Instruction::Add(1, 2, 3)
    );
}

#[test]
fn decode_immediate() {
    assert_eq!(
        Instruction::decode(&[0x10, 1, 2, 42, 0, 0, 0]).unwrap(),
        Instruction::AddI(1, 2, 42)
    );
}

#[test]
fn register_zero_is_hardwired() {
    let mut cpu = Cpu::new(1024);
    cpu.load_program(&[0x10, 1, 0, 100, 0, 0, 0, 0x00]);
    cpu.run(100);
    assert_eq!(cpu.registers()[0], 0);
    assert_eq!(cpu.registers()[1], 100);
}

#[test]
fn arithmetic() {
    let mut cpu = Cpu::new(1024);

    cpu.load_program(&[
        0x10, 1, 0, 20, 0, 0, 0, 0x10, 2, 0, 5, 0, 0, 0, 0x20, 3, 1, 2, 0x21, 4, 1, 2, 0x22, 5, 1,
        2, 0x00,
    ]);

    cpu.run(100);

    assert_eq!(cpu.registers()[3], 25);
    assert_eq!(cpu.registers()[4], 15);
    assert_eq!(cpu.registers()[5], 100);
}

#[test]
fn division() {
    let mut cpu = Cpu::new(1024);

    cpu.load_program(&[
        0x10, 1, 0, 100, 0, 0, 0, 0x10, 2, 0, 4, 0, 0, 0, 0x24, 3, 1, 2, 0x00,
    ]);

    cpu.run(100);

    assert_eq!(cpu.registers()[3], 25);
}

#[test]
fn branches() {
    let mut cpu = Cpu::new(1024);

    cpu.load_program(&[
        0x10, 1, 0, 1, 0, 0, 0, 0x10, 2, 0, 1, 0, 0, 0, 0x27, 1, 2, 0x61, 0x1c, 0, 0, 0, 0x10, 3,
        0, 99, 0, 0, 0, 0x00,
    ]);

    cpu.run(100);

    assert_eq!(cpu.registers()[3], 0);
}

#[test]
fn stack_call_return() {
    let mut cpu = Cpu::new(4096);

    cpu.load_program(&[0x72, 0x06, 0, 0, 0, 0x00, 0x10, 1, 0, 42, 0, 0, 0, 0x73]);

    cpu.run(100);

    assert_eq!(cpu.registers()[1], 42);
}

#[test]
fn rotate() {
    let mut cpu = Cpu::new(1024);

    cpu.load_program(&[
        0x10, 1, 0, 1, 0, 0, 0, 0x10, 2, 0, 4, 0, 0, 0, 0x43, 3, 1, 2, 0x00,
    ]);

    cpu.run(100);

    assert_eq!(cpu.registers()[3], 16);
}
