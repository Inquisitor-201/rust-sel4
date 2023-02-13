pub fn sel4_cnode_copy(
    _service: usize,
    dest_index: usize,
    dest_depth: usize,
    src_root: usize,
    src_index: usize,
    src_depth: usize,
    rights: usize,
) {
    // 	seL4_Error result;
    // 	seL4_MessageInfo_t tag = seL4_MessageInfo_new(CNodeCopy, 0, 1, 5);
    // 	seL4_MessageInfo_t output_tag;
    // 	seL4_Word mr0;
    // 	seL4_Word mr1;
    // 	seL4_Word mr2;
    // 	seL4_Word mr3;

    // 	/* Setup input capabilities. */
    // 	seL4_SetCap(0, src_root);

    // 	/* Marshal and initialise parameters. */
    // 	mr0 = dest_index;
    // 	mr1 = (dest_depth & 0xffull);
    // 	mr2 = src_index;
    // 	mr3 = (src_depth & 0xffull);
    // 	seL4_SetMR(4, rights.words[0]);

    // 	/* Perform the call, passing in-register arguments directly. */
    // 	output_tag = seL4_CallWithMRs(_service, tag,
    // 		&mr0, &mr1, &mr2, &mr3);
    // 	result = (seL4_Error) seL4_MessageInfo_get_label(output_tag);

    // 	/* Unmarshal registers into IPC buffer on error. */
    // 	if (result != seL4_NoError) {
    // 		seL4_SetMR(0, mr0);
    // 		seL4_SetMR(1, mr1);
    // 		seL4_SetMR(2, mr2);
    // 		seL4_SetMR(3, mr3);
    // #ifdef CONFIG_KERNEL_INVOCATION_REPORT_ERROR_IPC
    // 		if (seL4_CanPrintError()) {
    // 			seL4_DebugPutString(seL4_GetDebugError());
    // 		}
    // #endif
    // 	}

    // 	return result;
}
