.align 2
.global trap_entry
trap_entry:
	csrrw	t0, sscratch, t0
	sd	ra, 0(t0)
	sd	sp, 8(t0)
	sd	gp, 16(t0)
	sd	tp, 24(t0)
	sd	t1, 40(t0)
	sd	t2, 48(t0)
	sd	s0, 56(t0)
	sd	s1, 64(t0)
	sd	a0, 72(t0)
	sd	a1, 80(t0)
	sd	a2, 88(t0)
	sd	a3, 96(t0)
	sd	a4, 104(t0)
	sd	a5, 112(t0)
	sd	a6, 120(t0)
	sd	a7, 128(t0)
	sd	s2, 136(t0)
	sd	s3, 144(t0)
	sd	s4, 152(t0)
	sd	s5, 160(t0)
	sd	s6, 168(t0)
	sd	s7, 176(t0)
	sd	s8, 184(t0)
	sd	s9, 192(t0)
	sd	s10, 200(t0)
	sd	s11, 208(t0)
	sd	t3, 216(t0)
	sd	t4, 224(t0)
	sd	t5, 232(t0)
	sd	t6, 240(t0)
	
	/* Store t0, sstatus, scause */
	csrr	ra, sscratch
	sd	ra, 32(t0)
	csrr	ra, sstatus
	sd	ra, 256(t0)
	csrr	s0, scause
	sd	s0, 248(t0)

	/* Load kernel's stack address */
	la  sp, boot_stack_top

	/* Store sepc */
	csrr	x1, sepc
	sd	x1, 264(t0)

	/* Check if it's an interrupt */
	bltz s0, interrupt

	/* ratified priv has value 8 for ecall from U-mode exception */
	li   s4, 8
	bne  s0, s4, exception

syscall:
	addi x1, x1, 4
	sd x1, 272(t0)
	mv a2, a7
	j handle_syscall

interrupt:
	sd x1, 272(t0)
	j handle_interrupt

exception:
	sd x1, 272(t0)
	j handle_exception

.align 2
.globl __restore
__restore:
    mv	t0, a0
    ld	ra, 0(t0)
	ld	sp, 8(t0)
	ld	gp, 16(t0)

    /* skip tp */
    /* skip x5/t0 */
    /* no-op store conditional to clear monitor state */
    /* this may succeed in implementations with very large reservations, but the saved ra is dead */

	sc.w	zero, zero, (t0)
	ld	t2, 48(t0)
	ld	s0, 56(t0)
	ld	s1, 64(t0)
	ld	a0, 72(t0)
	ld	a1, 80(t0)
	ld	a2, 88(t0)
	ld	a3, 96(t0)
	ld	a4, 104(t0)
	ld	a5, 112(t0)
	ld	a6, 120(t0)
	ld	a7, 128(t0)
	ld	s2, 136(t0)
	ld	s3, 144(t0)
	ld	s4, 152(t0)
	ld	s5, 160(t0)
	ld	s6, 168(t0)
	ld	s7, 176(t0)
	ld	s8, 184(t0)
	ld	s9, 192(t0)
	ld	s10, 200(t0)
	ld	s11, 208(t0)
	ld	t3, 216(t0)
	ld	t4, 224(t0)
	ld	t5, 232(t0)
	ld	t6, 240(t0)

    /* Get next restored tp */
	ld	t1, 24(t0)
    /* get restored tp */
	add	tp, t1, zero
    /* get sepc */
	ld	t1, 272(t0)
	csrw	sepc, t1
	csrw	sscratch, t0
    /* get sstatus */
	ld	t1, 256(t0)
	csrw	sstatus, t1
    /* restore t0 & t1 */
	ld	t1, 40(t0)
	ld	t0, 32(t0)
	sret