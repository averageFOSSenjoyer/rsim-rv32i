# x1 is vga ptr
# x2 is value to write
# x3 is loop max 25*80*2=4000

.globl _start
_start:
    li x1, 0x000B8000
    li x2, 0
    li x3, 4000

loop:
    bgeu x2, x3, halt
    sb x2, 0(x1)
    addi x1, x1, 2
    addi x2, x2, 2
    j loop

halt:
    slti x0, x0, -256

