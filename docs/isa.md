# StatusCPU v1 Architecture

## Core

StatusCPU is a 32-bit little-endian CPU.

General-purpose register count:

16

Register width:

32 bits

Address width:

32 bits

Instruction width:

Variable length.

The architectural program counter is 32 bits.

## Registers

R0 is permanently zero.

R1-R12 are general purpose registers.

R13 is a temporary register.

R14 is the frame pointer.

R15 is the stack pointer.

Writing R0 has no effect.

## Program counter

PC contains the address of the next instruction.

Instructions update PC before execution. Taken branches replace the already advanced PC.

## Flags

FLAGS contains:

bit 0 Z
bit 1 N
bit 2 C
bit 3 V

Z indicates a zero result.

N indicates bit 31 is set.

C indicates carry or borrow according to the operation.

V indicates signed overflow.

## Privilege

Three privilege modes exist:

0 USER
1 SUPERVISOR
2 MACHINE

Reset begins in MACHINE mode.

Privileged operations are rejected outside their required privilege level.

## Exceptions

Exceptions save:

EPC
CAUSE

The processor then jumps to TVEC.

## Interrupts

Interrupts are maskable through STATUS.

Timer interrupts are supported.

External interrupts are supported by the interrupt controller interface.

## CSR registers

0x000 STATUS
0x001 CAUSE
0x002 EPC
0x003 TVEC
0x004 SCRATCH
0x005 CYCLE
0x006 INSTRET
0x007 MODE
0x008 IE
0x009 IP
0x00A CPU_ID

## Reset

PC = 0xFFFF0000

SP = top of RAM

MODE = MACHINE

STATUS = 0

TVEC = 0xFFFF0100

## Instruction execution

The reference emulator executes instructions sequentially.

The RTL uses the same architectural semantics.

## Memory

All memory addresses are 32-bit.

Memory is little endian.

Misaligned halfword and word accesses generate an alignment exception.

## MMIO

0x80000000 UART
0x80000100 TIMER
0x80000200 INTERRUPT CONTROLLER
0x80000300 CPU CONTROL

## Boot ROM

0xFFFF0000-0xFFFFFFFF is reserved for boot ROM.