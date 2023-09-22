	.text
	.attribute	4, 16
	.attribute	5, "rv64i2p0"
	.file	"fib.c"
	.globl	main
	.p2align	2
	.type	main,@function
main:
	addi	sp, sp, -32
	sd	ra, 24(sp)
	sd	s0, 16(sp)
	addi	s0, sp, 32
	li	a0, 0
	sw	a0, -20(s0)
	li	a0, 10
	call	fib
	ld	ra, 24(sp)
	ld	s0, 16(sp)
	addi	sp, sp, 32
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
	addi	s0, sp, 32
	sw	a0, -24(s0)
	lw	a0, -24(s0)
	li	a1, 0
	beq	a0, a1, .LBB1_2
	j	.LBB1_1
.LBB1_1:
	lw	a0, -24(s0)
	li	a1, 1
	bne	a0, a1, .LBB1_3
	j	.LBB1_2
.LBB1_2:
	lw	a0, -24(s0)
	sw	a0, -20(s0)
	j	.LBB1_4
.LBB1_3:
	lw	a0, -24(s0)
	addiw	a0, a0, -1
	call	fib
	sd	a0, -32(s0)
	lw	a0, -24(s0)
	addiw	a0, a0, -2
	call	fib
	mv	a1, a0
	ld	a0, -32(s0)
	addw	a0, a0, a1
	sw	a0, -20(s0)
	j	.LBB1_4
.LBB1_4:
	lw	a0, -20(s0)
	ld	ra, 24(sp)
	ld	s0, 16(sp)
	addi	sp, sp, 32
	ret
.Lfunc_end1:
	.size	fib, .Lfunc_end1-fib

	.ident	"Ubuntu clang version 14.0.0-1ubuntu1.1"
	.section	".note.GNU-stack","",@progbits
	.addrsig
	.addrsig_sym fib
