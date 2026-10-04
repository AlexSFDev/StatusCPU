# Privilege

StatusCPU has three modes.

0 USER
1 SUPERVISOR
2 MACHINE

MODE is stored in CSR 0x007.

Machine mode may access every CSR.

Supervisor mode may access supervisor and user state.

User mode may only access user-visible CSRs.

An illegal privileged access generates CAUSE_PRIVILEGE.

MRET restores machine execution from EPC.

SRET restores supervisor execution from EPC.