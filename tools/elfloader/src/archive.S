.section ._archive_cpio
.globl _cpio_archive, _cpio_archive_end
_cpio_archive:
.incbin "target/archive.archive.o.cpio"
_cpio_archive_end: