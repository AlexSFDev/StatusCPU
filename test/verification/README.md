# StatusCPU Verification

Verification has two architectural targets.

The Rust emulator is the reference model.

The SystemVerilog implementation is the hardware model.

Programs under `test/programs` are assembled and executed against the emulator.

The same programs are loaded into RTL testbenches.

Architectural state is compared after execution.

The minimum comparison set is:

- PC
- R0-R15
- FLAGS
- STATUS
- CAUSE
- EPC
- MODE
- CYCLE
- INSTRET

A program passes only when the architectural results match.