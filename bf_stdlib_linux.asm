global print
global zero_mem
global exit
section .text

;Prints a given character to stdout
;Params:
;	rdi: A pointer to the character to print in ascii form
print:
	push 	rax
	push 	rsi
	push 	rdx
		
  	mov 	rax, 	1
  	mov 	rsi, 	rdi
	mov 	rdi, 	1
	mov 	rdx,	1

  	syscall

	pop 	rdx
	pop 	rsi
	pop 	rax
	ret

;Zeroes a region of memory
;Params:
;	rdi: The pointer to the region. This should be the lowest location.
;	rsi: The amount of bytes to zero
zero_mem:
	cmp 	rsi, 	0
	jne 	zero_mem_1
	ret
zero_mem_1:
	mov byte [rdi+rsi], 0
	inc rdi
	dec rsi
	jmp zero_mem

;Exits the programm
exit:
  	mov 	rax, 	60
  	mov 	rdi, 	0 
  	syscall
