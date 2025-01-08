.globl _start
_start:
    li x1, 0x000A0000
    lb x2, 0(x1)
    li x1, 0x000A0001
    lb x3, 0(x1)
    slti x0, x0, -256
