use core::arch::global_asm;
use cpio_reader::iter_files;
use xmas_elf::ElfFile;

use crate::println;
global_asm!(include_str!("archive.S"));

pub const PAGE_BITS: usize = 12;

#[macro_export]
macro_rules! round_up {
    ($n: expr, $b: expr) => {
        ((($n - 1) >> $b) + 1) << $b
    };
}

#[macro_export]
macro_rules! round_down {
    ($n: expr, $b: expr) => {
        $n >> $b
    };
}

#[macro_export]
macro_rules! mask {
    ($b: expr) => {
        (1 << $b) - 1
    };
}

#[macro_export]
macro_rules! is_aligned {
    ($n: expr, $b: expr) => {
        ($n & mask!($b)) == 0
    };
}

fn elf_get_memory_bounds(elf: &ElfFile, is_phys: bool) -> (u64, u64) {
    let mut mem_min = u64::max_value();
    let mut mem_max = 0u64;
    for header in elf.program_iter() {
        if header.get_type().unwrap() == xmas_elf::program::Type::Load {
            let sect_min = if is_phys {
                header.physical_addr()
            } else {
                header.virtual_addr()
            };
            let sect_max = sect_min + header.mem_size();
            mem_min = mem_min.min(sect_min);
            mem_max = mem_max.max(sect_max);
        }
    }
    (mem_min, mem_max)
}

fn elf_get_entry_point(elf: &ElfFile) -> u64 {
    elf.header.pt2.entry_point()
}

fn unpack_elf(elf: &ElfFile) {
    for header in elf.program_iter() {
        if header.get_type().unwrap() == xmas_elf::program::Type::Load {
            let seg_dest_paddr = header.physical_addr();
            let seg_size = header.file_size();
            let seg_offset = header.offset();
            unsafe {
                core::slice::from_raw_parts_mut(seg_dest_paddr as *mut u8, seg_size as usize)
                    .copy_from_slice(core::slice::from_raw_parts(
                        elf.input.as_ptr().add(seg_offset as usize),
                        seg_size as usize,
                    ));
            }
        }
    }
}

fn load_elf(name: &str, elf: &ElfFile, dest_paddr: u64) {
    println!("ELF-loading image {:#x?} to {:#x?}", name, dest_paddr);
    let (min_vaddr, mut max_vaddr) = elf_get_memory_bounds(elf, false);
    max_vaddr = round_up!(max_vaddr, PAGE_BITS);
    let image_size = max_vaddr - min_vaddr;

    assert!(is_aligned!(dest_paddr, PAGE_BITS));
    println!(
        "  paddr=[{:#x?}..{:#x?}]",
        dest_paddr,
        dest_paddr + image_size - 1
    );
    println!("  vaddr=[{:#x?}..{:#x?}]", min_vaddr, max_vaddr - 1);
    println!("  virt_entry={:#x?}", elf_get_entry_point(elf));

    unpack_elf(elf);
}

pub fn load_images(max_user_images: usize, bootloader_dtb: *const u64) {
    extern "C" {
        fn _archive_start();
        fn _archive_end();
    }

    let cpio = unsafe {
        core::slice::from_raw_parts(
            _archive_start as usize as *mut u8,
            _archive_end as usize - _archive_start as usize,
        )
    };

    for entry in iter_files(cpio) {
        if entry.name().eq("kernel") {
            let kernel_elf_blob = entry.file();
            let kernel_elf = ElfFile::new(kernel_elf_blob).unwrap();
            let (kernel_phys_start, kernel_phys_end) = elf_get_memory_bounds(&kernel_elf, true);
            load_elf("kernel", &kernel_elf, kernel_phys_start);
        }
    }
}
