; data section contains the data defined before compilation
section .data
	; db stands for define bytes
	; so here we are defining bytes (or string)
	; to "variable" called text (10 is char value for new line)
	text db "Hello, World!",10

; section .text contains the actual code
section .text
	global _start

_start:
	; Here making a syscall with loading the appropriate values to 64-bit registers
	; We could also load them in 32-bit registers (eax, edi etc)
	; syscalls are made by loading the approriate values to register and calling the syscall
	; see syscall-register-table.txt for more info
	; this is the sys_write call that would look like this in c:
	; sys_write(int fd, char* buffer, int buff_len)
	mov rax, 1 ; id for sys_write
	mov rdi, 1 ; filedescriptor (0 stdin, 1 stdout, 2 stderr)
	mov rsi, text ; the data buffer (kinda like char*)
	mov rdx, 14 ; the length of the databuffer
	syscall

	; Here we are call the sys_exit that would look like this
	; sys_exit(int error_code);
	mov rax, 60 ; id for sys_exit
	mov rdi, 0 ; error code (0 is for successuful)
	syscall
