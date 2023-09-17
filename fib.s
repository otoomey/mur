	.text
	.attribute	4, 16
	.attribute	5, "rv64i2p0"
	.file	"fib.c"
	.globl	main
	.p2align	2
	.type	main,@function
main:
	addi	sp, sp, -16
	sd	ra, 8(sp)
	li	a0, 10
	call	fib
	ld	ra, 8(sp)
	addi	sp, sp, 16
	ret
.Lfunc_end0:
	.size	main, .Lfunc_end0-main

	.globl	fib
	.p2align	2
	.type	fib,@function
fib:
	addi	sp, sp, -32
	sd	ra, 24(sp)
	sd	s0, 16(sp)
	sd	s1, 8(sp)
	li	a1, 2
	mv	s0, a0
	bltu	a0, a1, .LBB1_2
	addiw	a0, s0, -1
	call	fib
	mv	s1, a0
	addiw	a0, s0, -2
	call	fib
	addw	s0, a0, s1
.LBB1_2:
	mv	a0, s0
	ld	ra, 24(sp)
	ld	s0, 16(sp)
	ld	s1, 8(sp)
	addi	sp, sp, 32
	ret
.Lfunc_end1:
	.size	fib, .Lfunc_end1-fib

	.ident	"Ubuntu clang version 14.0.0-1ubuntu1.1"
	.section	".note.GNU-stack","",@progbits
	.addrsig
