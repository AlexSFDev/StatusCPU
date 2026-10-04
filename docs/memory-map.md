# StatusCPU Memory Map

## RAM

0x00000000-0x7FFFFFFF

Normal RAM.

## MMIO

0x80000000-0x800000FF

UART.

0x80000100-0x800001FF

Timer.

0x80000200-0x800002FF

Interrupt controller.

0x80000300-0x800003FF

CPU control.

## Boot ROM

0xFFFF0000-0xFFFFFFFF

Boot ROM.

## UART

0x80000000 DATA

Write a byte to transmit it.

0x80000004 STATUS

bit 0 TX ready.

## TIMER

0x80000100 COUNTER

64-bit counter represented by two 32-bit registers.

0x80000108 COMPARE

64-bit timer compare value.

0x80000110 CONTROL

bit 0 enable.

bit 1 interrupt enable.

## Interrupt controller

0x80000200 PENDING

0x80000204 ENABLE

0x80000208 ACK

## CPU control

0x80000300 HALT

Writing 1 halts the processor.

0x80000304 CPU ID

Returns the architectural CPU identifier.