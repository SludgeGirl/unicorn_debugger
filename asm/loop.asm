
section .text
	global _start

_start:
    mov rsi, 0
_loop:
    inc rsi
    cmp rsi, 5
    jne _loop

_end:
	; Here we are call the sys_exit that would look like this
	; sys_exit(int error_code);
	mov rax, 60 ; id for sys_exit
	mov rdi, rsi; error code (0 is for successuful)
	syscall
