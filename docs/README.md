# StatusCPU

StatusCPU is the custom 32-bit processor architecture used by StatusProjects.

StatusCPU v1 provides:

- 32-bit little-endian execution
- 16 general-purpose registers
- Hardwired zero register
- Arithmetic and logical operations
- Multiplication and division
- Shifts and rotates
- Byte, halfword and word memory operations
- Conditional branches
- Calls and returns
- Stack operations
- Atomic load/store operations
- Control and status registers
- User, supervisor and machine privilege modes
- Exceptions
- Interrupts
- Timer
- UART MMIO
- Reference emulator
- Native assembler
- SystemVerilog RTL
- RTL verification

The emulator is the architectural reference implementation.

## Project structure

- `docs/` architecture documentation
- `emulator/` reference implementation
- `assembler/` assembler
- `rtl/` SystemVerilog implementation
- `firmware/` low-level programs
- `test/` verification programs
- `tools/` development tools