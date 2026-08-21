	.file	"wunzip.c"
# GNU C23 (GCC) version 16.1.1 20260625 (x86_64-pc-linux-gnu)
#	compiled by GNU C version 16.1.1 20260625, GMP version 6.3.0, MPFR version 4.2.2, MPC version 1.4.1, isl version isl-0.28-GMP

# GGC heuristics: --param ggc-min-expand=100 --param ggc-min-heapsize=131072
# options passed: -mtune=generic -march=x86-64 -O1
	.text
	.globl	print_block
	.type	print_block, @function
print_block:
.LFB22:
	.cfi_startproc
	leaq	-24(%rsp), %rsp	#,
	.cfi_def_cfa_offset 32
	movq	%rbp, 8(%rsp)	#,
	.cfi_offset 6, -24
# wunzip.c:14:   int count = *(int *) bytes;
	movl	(%rdi), %ebp	# MEM[(int *)bytes_5(D)], count
# wunzip.c:17:   for (int i = 0; i < count; i++) {
	testl	%ebp, %ebp	# count
	jle	.L1	#,
	movq	%rbx, (%rsp)	#,
	movq	%r12, 16(%rsp)	#,
	.cfi_offset 3, -32
	.cfi_offset 12, -16
# wunzip.c:15:   int letter = bytes[sizeof(int)];
	movsbl	4(%rdi), %r12d	# MEM[(char *)bytes_5(D) + 4B], letter
# wunzip.c:17:   for (int i = 0; i < count; i++) {
	movl	$0, %ebx	#, i
.L3:
# wunzip.c:18:     printf("%c", letter);
	movl	%r12d, %edi	# letter,
	call	putchar@PLT	#
# wunzip.c:17:   for (int i = 0; i < count; i++) {
	addl	$1, %ebx	#, i
# wunzip.c:17:   for (int i = 0; i < count; i++) {
	cmpl	%ebx, %ebp	# i, count
	jne	.L3	#,
	movq	(%rsp), %rbx	#,
	.cfi_restore 3
	movq	16(%rsp), %r12	#,
	.cfi_restore 12
.L1:
# wunzip.c:20: }
	movq	8(%rsp), %rbp	#,
	leaq	24(%rsp), %rsp	#,
	.cfi_def_cfa_offset 8
	ret	
	.cfi_endproc
.LFE22:
	.size	print_block, .-print_block
	.section	.rodata.str1.1,"aMS",@progbits,1
.LC0:
	.string	"wunzip: file1 [file2 ...]"
.LC1:
	.string	"malloc failed\n"
.LC2:
	.string	"open: "
.LC3:
	.string	"failed to open file\n"
.LC4:
	.string	"invalid file size\n"
.LC5:
	.string	"read: "
.LC6:
	.string	"failed to read file\n"
	.text
	.globl	main
	.type	main, @function
main:
.LFB23:
	.cfi_startproc
	pushq	%r15	#
	.cfi_def_cfa_offset 16
	.cfi_offset 15, -16
	pushq	%r14	#
	.cfi_def_cfa_offset 24
	.cfi_offset 14, -24
	pushq	%r13	#
	.cfi_def_cfa_offset 32
	.cfi_offset 13, -32
	pushq	%r12	#
	.cfi_def_cfa_offset 40
	.cfi_offset 12, -40
	pushq	%rbp	#
	.cfi_def_cfa_offset 48
	.cfi_offset 6, -48
	pushq	%rbx	#
	.cfi_def_cfa_offset 56
	.cfi_offset 3, -56
	subq	$24, %rsp	#,
	.cfi_def_cfa_offset 80
# wunzip.c:23:   if (argc < 2) {
	cmpl	$1, %edi	#, argc
	jle	.L20	#,
	movl	%edi, %r12d	# argc, argc
	movq	%rsi, %rbx	# argv, argv
# wunzip.c:29:   char *buf = malloc(buf_len * sizeof(char));
	movl	$5120, %edi	#,
	call	malloc@PLT	#
	movq	%rax, %rbp	# buf, buf
# wunzip.c:30:   if (!buf) {
	testq	%rax, %rax	# buf
	je	.L21	#,
	leaq	8(%rbx), %r15	#, ivtmp.20
	leal	-2(%r12), %eax	#, _55
	leaq	16(%rbx,%rax,8), %rax	#, _59
	movq	%rax, 8(%rsp)	# _59, %sfp
# wunzip.c:46:       if (nread % BLOCK_SIZE != 0) {
	movabsq	$-3689348814741910323, %r14	#, tmp147
	jmp	.L13	#
.L20:
# wunzip.c:24:     printf("wunzip: file1 [file2 ...]\n");
	leaq	.LC0(%rip), %rdi	#,
	call	puts@PLT	#
# wunzip.c:25:     exit(EXIT_FAILURE);
	movl	$1, %edi	#,
	call	exit@PLT	#
.L21:
# wunzip.c:31:     err(EXIT_FAILURE, "malloc failed\n");
	leaq	.LC1(%rip), %rsi	#,
	movl	$1, %edi	#,
	movl	$0, %eax	#,
	call	err@PLT	#
.L11:
# wunzip.c:46:       if (nread % BLOCK_SIZE != 0) {
	movslq	%eax, %rcx	# _11, _5
	movq	%rcx, %rax	# _5, tmp153
	mulq	%r14	# tmp147
	movq	%rdx, %rax	# tmp134, tmp133
	shrq	$2, %rax	#, tmp133
	andq	$-4, %rdx	#, tmp137
	addq	%rax, %rdx	# tmp133, tmp138
# wunzip.c:46:       if (nread % BLOCK_SIZE != 0) {
	cmpq	%rdx, %rcx	# tmp138, _5
	jne	.L22	#,
# wunzip.c:50:       idx = 0;
	movl	$0, %ebx	#, idx
.L10:
# wunzip.c:54:         print_block(&buf[idx]);
	movslq	%ebx, %rdi	# idx, _7
	addq	%rbp, %rdi	# buf, _8
	call	print_block	#
	leal	5(%rbx), %eax	#, _41
# wunzip.c:55:         idx += BLOCK_SIZE;
	movl	%eax, %ebx	# _41, idx
# wunzip.c:53:       while (idx < nread) {
	cmpl	%eax, %r12d	# _41, nread
	jg	.L10	#,
.L9:
# wunzip.c:45:     while ((nread = read(fd, buf, buf_len)) > 0) {
	movl	$5120, %edx	#,
	movq	%rbp, %rsi	# buf,
	movl	%r13d, %edi	# fd,
	call	read@PLT	#
# wunzip.c:45:     while ((nread = read(fd, buf, buf_len)) > 0) {
	movl	%eax, %r12d	# _11, nread
# wunzip.c:45:     while ((nread = read(fd, buf, buf_len)) > 0) {
	testl	%eax, %eax	# _11
	jg	.L11	#,
# wunzip.c:59:     if (nread < 0) {
	js	.L23	#,
# wunzip.c:63:     close(fd);
	movl	%r13d, %edi	# fd,
	call	close@PLT	#
# wunzip.c:38:   for (int i = 1; i < argc; i++) {
	addq	$8, %r15	#, ivtmp.20
	movq	8(%rsp), %rax	# %sfp, _59
	cmpq	%rax, %r15	# _59, ivtmp.20
	je	.L24	#,
.L13:
# wunzip.c:39:     fd = open(argv[i], O_RDONLY);
	movq	(%r15), %rdi	# MEM[(char * *)_52], MEM[(char * *)_52]
	movl	$0, %esi	#,
	movl	$0, %eax	#,
	call	open@PLT	#
	movl	%eax, %r13d	# fd, fd
# wunzip.c:40:     if (fd < 0) {
	testl	%eax, %eax	# fd
	jns	.L9	#,
# wunzip.c:41:       perror("open: ");
	leaq	.LC2(%rip), %rdi	#,
	call	perror@PLT	#
# wunzip.c:42:       err(EXIT_FAILURE, "failed to open file\n");
	leaq	.LC3(%rip), %rsi	#,
	movl	$1, %edi	#,
	movl	$0, %eax	#,
	call	err@PLT	#
.L22:
# wunzip.c:47:         err(EXIT_FAILURE, "invalid file size\n");
	leaq	.LC4(%rip), %rsi	#,
	movl	$1, %edi	#,
	movl	$0, %eax	#,
	call	err@PLT	#
.L23:
# wunzip.c:60:       perror("read: ");
	leaq	.LC5(%rip), %rdi	#,
	call	perror@PLT	#
# wunzip.c:61:       err(EXIT_FAILURE, "failed to read file\n");
	leaq	.LC6(%rip), %rsi	#,
	movl	$1, %edi	#,
	movl	$0, %eax	#,
	call	err@PLT	#
.L24:
# wunzip.c:66:   free(buf);
	movq	%rbp, %rdi	# buf,
	call	free@PLT	#
# wunzip.c:68: }
	movl	$0, %eax	#,
	addq	$24, %rsp	#,
	.cfi_def_cfa_offset 56
	popq	%rbx	#
	.cfi_def_cfa_offset 48
	popq	%rbp	#
	.cfi_def_cfa_offset 40
	popq	%r12	#
	.cfi_def_cfa_offset 32
	popq	%r13	#
	.cfi_def_cfa_offset 24
	popq	%r14	#
	.cfi_def_cfa_offset 16
	popq	%r15	#
	.cfi_def_cfa_offset 8
	ret	
	.cfi_endproc
.LFE23:
	.size	main, .-main
	.ident	"GCC: (GNU) 16.1.1 20260625"
	.section	.note.GNU-stack,"",@progbits
