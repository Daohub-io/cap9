# System Calls

## Necessary System Calls

First we identify which opcodes we need to use via system calls. These are all
the "state-changing" opcodes.

- SSTORE
- LOG0-4
- SELFDESTRUCT
- CREATE
- CREATE2
- CALL
- CALLCODE
- DELEGATECALL

Second we identify non-state-changing opcodes we may also want to control.

- SLOAD

Thirdly we make this into system calls. Here are the first three fundamental
system calls:

- Read
- Write
- LOG0-4

The rest will come.


