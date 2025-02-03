#!/bin/bash
riscv32-unknown-elf-gcc -mcmodel=medany -static -fno-common -ffreestanding -nostartfiles \
  -lm -static-libgcc -lgcc -lc -Wl,--no-relax \
  -march=rv32i -mabi=ilp32 -Ofast -flto -Wall -Wextra -Wno-unused -Tlink.ld $1 -o $1.elf

riscv32-unknown-elf-objdump -D $1.elf > $1.disas