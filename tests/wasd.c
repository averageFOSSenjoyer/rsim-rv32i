#define VGA_NUM_ROWS 25
#define VGA_NUM_COLS 80

__attribute__((noinline))
int mul(int a, int b) {
    int ret = 0;
    for (int i = 0; i < a; ++i) {
        ret += b;
    }
    return ret;
}

void _start() {
    volatile unsigned char* vga_ptr = (unsigned char*)0x000B8000;
    volatile unsigned char* kb_status_ptr = (unsigned char*)0x000A0000;
    volatile char* kb_value_ptr = (char*)0x000A0001;

    char key;
    unsigned int old_x = 0;
    unsigned int old_y = 0;
    unsigned int new_x = 0;
    unsigned int new_y = 0;

    vga_ptr[mul(2, (mul(old_y, VGA_NUM_COLS) + old_x))] = '*';

    while (1) {
        if (*kb_status_ptr == 0x1) {
            key = *kb_value_ptr;
            if (key == 'w') {
                if (old_y == 0) {
                    new_y = VGA_NUM_ROWS - 1;
                } else {
                    new_y = old_y - 1;
                }
            } else if (key == 'a') {
                if (old_x == 0) {
                    new_x = VGA_NUM_COLS - 1;
                } else {
                    new_x = old_x - 1;
                }
            } else if (key == 's') {
                if (old_y == VGA_NUM_ROWS - 1) {
                    new_y = 0;
                } else {
                    new_y = old_y + 1;
                }
            } else if (key == 'd') {
                if (old_x == VGA_NUM_COLS - 1) {
                    new_x = 0;
                } else {
                    new_x = old_x + 1;
                }
            } else if (key == 'q') {
                break;
            }
            vga_ptr[mul(2, (mul(old_y, VGA_NUM_COLS) + old_x))] = ' ';
            vga_ptr[mul(2, (mul(new_y, VGA_NUM_COLS) + new_x))] = '*';
            old_x = new_x;
            old_y = new_y;
        }
    }

    __asm__(
        "slti x0, x0, -256"
    );
}