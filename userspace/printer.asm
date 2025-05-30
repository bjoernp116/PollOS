section .text

global _start

_start:
    mov ebx, 1
    mov ecx, HELLO_WORLD
    mov esi, HELLO_WORLD_LEN
    int 0x80

section .data
HELLO_WORLD db "Hello, World!", 0
HELLO_WORLD_LEN equ $ - HELLO_WORLD
