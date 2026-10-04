# Interrupts

StatusCPU supports timer and external interrupts.

Interrupt enable state is stored in IE.

Pending interrupts are stored in IP.

When an enabled interrupt is accepted:

EPC receives the interrupted PC.

CAUSE receives the interrupt cause.

PC becomes TVEC.

The interrupt is serviced in MACHINE mode.

Timer interrupt cause:

0x80000001

External interrupt cause:

0x80000002

Exceptions have bit 31 clear.

Interrupt causes have bit 31 set.