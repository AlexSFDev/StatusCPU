# StatusCPU v1 Instruction Set Architecture

## Overview

StatusCPU is a 32-bit general-purpose processor architecture designed as the CPU foundation for StatusOS.

## Architecture

- Word size: 32-bit
- General-purpose registers: 16
- Register width: 32-bit
- Address width: 32-bit
- Address space: 4 GiB
- Endianness: little-endian
- Register R0: hardwired to zero
- Program counter: PC
- Stack pointer: SP
- Status/flags register: FLAGS
- Control and status registers: CSR
- Privilege modes: User, Supervisor, Machine

## Registers

| Register | Name | Description |
|---|---|---|
| R0 | ZERO | Constant zero |
| R1-R12 | GPR | General-purpose registers |
| R13 | SP | Stack pointer |
| R14 | LR | Link/return register |
| R15 | TMP | General-purpose temporary register |

R0 always reads as zero. Writes to R0 are discarded.

## Flags

| Flag | Description |
|---|---|
| Z | Zero |
| C | Carry |
| N | Negative |
| V | Signed overflow |

The flags are stored in the processor status register.

## Instruction Encoding

Instructions are variable length.

### Register ALU

    +--------+------+------+------+
    | Opcode | Rd   | Rs1  | Rs2  |
    +--------+------+------+------+
    | 1 byte | 1    | 1    | 1    |
    +--------+------+------+------+

### Immediate

    +--------+------+------+----------------+
    | Opcode | Rd   | Rs1  | Immediate      |
    +--------+------+------+----------------+
    | 1 byte | 1    | 1    | 4 bytes        |
    +--------+------+------+----------------+

### Memory

    +--------+------+------+----------------+
    | Opcode | Rd   | Base | Offset         |
    +--------+------+------+----------------+
    | 1 byte | 1    | 1    | 4 bytes        |
    +--------+------+------+----------------+

### Branch

    +--------+------------------------------+
    | Opcode | Address                      |
    +--------+------------------------------+
    | 1 byte | 4 bytes                      |
    +--------+------------------------------+

## System Instructions

| Opcode | Instruction | Size |
|---|---|---:|
| `0x00` | HALT | 1 |
| `0x01` | NOP | 1 |
| `0x02` | WFI | 1 |
| `0x03` | ERET | 1 |
| `0x04` | BREAK | 1 |

## Immediate Arithmetic

| Opcode | Instruction |
|---|---|
| `0x10` | ADDI |
| `0x11` | SUBI |
| `0x12` | ANDI |
| `0x13` | ORI |
| `0x14` | XORI |
| `0x15` | MULI |
| `0x16` | SLTI |
| `0x17` | SLTIU |

Format:

    OP Rd, Rs1, Immediate

Example:

    ADDI R1, R0, 100

## Integer Arithmetic

| Opcode | Instruction |
|---|---|
| `0x20` | ADD |
| `0x21` | SUB |
| `0x22` | MUL |
| `0x23` | DIV |
| `0x24` | DIVU |
| `0x25` | REM |
| `0x26` | REMU |
| `0x27` | CMP |
| `0x28` | MIN |
| `0x29` | MAX |

Format:

    OP Rd, Rs1, Rs2

`CMP` compares two registers and updates FLAGS without storing a result.

## Logical Operations

| Opcode | Instruction |
|---|---|
| `0x30` | AND |
| `0x31` | OR |
| `0x32` | XOR |
| `0x33` | NOT |
| `0x34` | CLZ |
| `0x35` | CTZ |
| `0x36` | POPCNT |

## Shift and Rotate

| Opcode | Instruction |
|---|---|
| `0x40` | SHL |
| `0x41` | SHR |
| `0x42` | SAR |
| `0x43` | ROL |
| `0x44` | ROR |

Format:

    OP Rd, Rs1, Rs2

`SHL` performs a logical left shift.

`SHR` performs a logical right shift.

`SAR` performs an arithmetic right shift.

`ROL` rotates left.

`ROR` rotates right.

## Memory Operations

| Opcode | Instruction | Size |
|---|---|---:|
| `0x50` | LB | 7 |
| `0x51` | LBU | 7 |
| `0x52` | LH | 7 |
| `0x53` | LHU | 7 |
| `0x54` | LW | 7 |
| `0x55` | SB | 7 |
| `0x56` | SH | 7 |
| `0x57` | SW | 7 |

Format:

    LW Rd, Base, Offset
    SW Rs, Base, Offset

Effective address:

    address = Base + Offset

### Load Instructions

`LB` loads a signed byte.

`LBU` loads an unsigned byte.

`LH` loads a signed 16-bit halfword.

`LHU` loads an unsigned 16-bit halfword.

`LW` loads a 32-bit word.

### Store Instructions

`SB` stores one byte.

`SH` stores one 16-bit halfword.

`SW` stores one 32-bit word.

## Atomic Operations

| Opcode | Instruction |
|---|---|
| `0x58` | LRW |
| `0x59` | SCW |

### LRW

Load-reserved word.

    LRW R1, R2

Loads the 32-bit value from the address in R2 and establishes a reservation.

### SCW

Store-conditional word.

    SCW R1, R2

Attempts to store the value in R1 to the address in R2.

The result indicates whether the store succeeded.

These instructions are intended to support synchronization primitives and operating-system locks.

## Branch Instructions

| Opcode | Instruction |
|---|---|
| `0x60` | JMP |
| `0x61` | JZ |
| `0x62` | JNZ |
| `0x63` | JC |
| `0x64` | JN |
| `0x65` | JO |
| `0x66` | JGE |
| `0x67` | JL |
| `0x68` | JGT |
| `0x69` | JLE |
| `0x6A` | JCS |
| `0x6B` | JCC |

### Unconditional

    JMP target

### Conditional

    JZ target
    JNZ target
    JC target
    JN target
    JO target
    JGE target
    JL target
    JGT target
    JLE target
    JCS target
    JCC target

Conditions are evaluated using FLAGS.

## Stack Operations

| Opcode | Instruction |
|---|---|
| `0x70` | PUSH |
| `0x71` | POP |

Examples:

    PUSH R1
    POP R1

The stack grows downward.

The stack pointer is maintained by the CPU.

## Subroutines

| Opcode | Instruction |
|---|---|
| `0x72` | CALL |
| `0x73` | RET |

### CALL

    CALL function

CALL saves the return address on the stack and transfers execution to the target address.

### RET

    RET

RET retrieves the saved return address and resumes execution.

Example:

    CALL function
    HALT

    function:
    ADDI R1, R0, 42
    RET

## Control and Status Registers

| Opcode | Instruction |
|---|---|
| `0x90` | CSRR |
| `0x91` | CSRW |
| `0x92` | CSRS |
| `0x93` | CSRC |

### CSRR

Read a CSR:

    CSRR R1, CSR

### CSRW

Write a CSR:

    CSRW R1, CSR

### CSRS

Set bits in a CSR:

    CSRS R1, CSR

### CSRC

Clear bits in a CSR:

    CSRC R1, CSR

## Privilege and Return Instructions

| Opcode | Instruction |
|---|---|
| `0xA0` | MRET |
| `0xA1` | SRET |
| `0xA2` | SETMODE |

### MRET

Returns from a machine-mode exception or interrupt.

### SRET

Returns from a supervisor-mode exception or interrupt.

### SETMODE

Changes the processor privilege mode according to the value supplied by the register.

## Interrupt Instructions

| Opcode | Instruction |
|---|---|
| `0xB0` | INT |
| `0xB1` | IRET |

### INT

Software interrupt:

    INT vector

### IRET

Returns from an interrupt handler.

## Memory-Mapped I/O

StatusCPU reserves the following address ranges for hardware devices.

| Base | Device |
|---|---|
| `0x10000000` | UART |
| `0x10001000` | Timer |
| `0x10002000` | Interrupt Controller |
| `0x10003000` | CPU Information |

## UART

Base address:

    0x10000000

| Offset | Register |
|---:|---|
| `0x00` | DATA |
| `0x04` | STATUS |

Writing a value to DATA transmits the low byte through the UART interface.

## Timer

Base address:

    0x10001000

| Offset | Register |
|---:|---|
| `0x00` | TIMER_LOW |
| `0x04` | TIMER_HIGH |
| `0x08` | COMPARE_LOW |
| `0x0C` | COMPARE_HIGH |
| `0x10` | CONTROL |

The timer is 64-bit.

Setting bit 0 of CONTROL enables timer interrupts.

## Interrupt Controller

Base address:

    0x10002000

| Offset | Register |
|---:|---|
| `0x00` | PENDING |
| `0x04` | ENABLE |
| `0x08` | CLEAR |

An interrupt is delivered when its pending bit is set and its corresponding enable bit is enabled.

## CPU Information

Base address:

    0x10003000

| Offset | Register | Value |
|---:|---|---|
| `0x00` | CPU ID | `1` |
| `0x04` | Vendor | `0x53544350` |
| `0x08` | Architecture Version | `1` |
| `0x0C` | Word Size | `32` |

## Data Directives

The assembler supports:

    .byte
    .half
    .word
    .ascii

Example:

    .data

    message:
    .word 123456

## Program Layout

StatusCPU binaries contain a fixed 32-byte header followed by code and data.

    +------------------------+
    | Header                 |
    | 32 bytes               |
    +------------------------+
    | Code                   |
    +------------------------+
    | Data                   |
    +------------------------+

### Header

| Offset | Size | Field |
|---:|---:|---|
| `0x00` | 4 | Magic |
| `0x04` | 2 | Version |
| `0x06` | 2 | Flags |
| `0x08` | 4 | Entry Point |
| `0x0C` | 4 | Code Offset |
| `0x10` | 4 | Code Size |
| `0x14` | 4 | Data Offset |
| `0x18` | 4 | Data Size |
| `0x1C` | 4 | Reserved |

Magic:

    STAT

Version:

    1

## Exceptions

The architecture reserves exception handling for:

- Invalid instructions
- Invalid memory accesses
- Privilege violations
- Division by zero
- Alignment faults
- Breakpoints
- Software traps
- External interrupts
- Timer interrupts

## Privilege Modes

StatusCPU defines three privilege levels:

    User
    Supervisor
    Machine

User mode is intended for applications.

Supervisor mode is intended for the operating system kernel and system services.

Machine mode is the highest privilege level and is intended for firmware, boot code, and low-level hardware control.

## Reset State

After reset:

    R0 = 0
    PC = reset vector
    SP = implementation-defined
    FLAGS = 0
    Privilege = Machine
    Interrupts = Disabled

All general-purpose registers have implementation-defined reset values except R0.

## Architectural Requirements

A conforming StatusCPU implementation must:

1. Implement 32-bit registers.
2. Maintain R0 as constant zero.
3. Implement the defined instruction encodings.
4. Implement little-endian memory access.
5. Implement the defined branch conditions.
6. Implement stack operations.
7. Implement CALL and RET.
8. Provide the defined CSR interface.
9. Provide the defined privilege modes.
10. Provide the defined interrupt architecture.
11. Provide the defined MMIO address space.

## Versioning

This document describes StatusCPU ISA version 1.

Future versions may add instructions, registers, architectural features, or implementation-defined extensions.

Existing instruction encodings should remain stable wherever practical.