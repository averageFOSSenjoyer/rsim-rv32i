# x1 is vga ptr
# x2 is keyboard status ptr
# x3 is keyboard value ptr
# x4 reads status and value from keyboard
# x5 holds 1 for comparison

.globl _start
_start:
    li x1, 0x000B8000
    li x2, 0x000A0000
    li x3, 0x000A0001
    li x5, 0x00000001

query:
    lb x4, 0(x2)
    bltu x4, x5, query

    lb x4, 0(x3)
    sb x4, 0(x1)

halt:
    slti x0, x0, -256
