# System Calls in PollOS

Registers:
    - ebx: System Call ID
    - ecx: Argument A
    - esi: Argument B
    - edi: Argument C
    - ebp: Argument D

## Function Signatures

| function | id | arg a | arg b | arg c | arg d | description |
|----------|----|------|------|------|------|-------------|
| print | 1 | *u8: buffer | u8: buffer len | | | Writes buffer to stdout |

