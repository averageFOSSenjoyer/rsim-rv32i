
void _start() {
    unsigned char* vga_ptr = (unsigned char*)0x000B8000;

    for (int i = 0; i < 2000; ++i) {
        vga_ptr[2 * i] = (unsigned char)i;
    }

    __asm__(
        "slti x0, x0, -256"
    );
}