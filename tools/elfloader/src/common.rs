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
macro_rules! bit {
    ($b: expr) => {
        1 << $b
    };
}

#[macro_export]
macro_rules! mask {
    ($b: expr) => {
        bit!($b) - 1
    };
}

#[macro_export]
macro_rules! is_aligned {
    ($n: expr, $b: expr) => {
        ($n & mask!($b)) == 0
    };
}

pub struct ImageInfo {
    pub phys_region_start: usize,
    pub phys_region_end: usize,

    /* Start/end byte in virtual memory the image requires to be located. */
    pub virt_region_start: usize,
    pub virt_region_end: usize,

    /* Virtual address of the user image's entry point. */
    pub virt_entry: usize,
    pub phys_virt_offset: usize,
}

fn elf_get_memory_bounds(elf: &ElfFile, is_phys: bool) -> (usize, usize) {
    let mut mem_min = usize::max_value();
    let mut mem_max = 0usize;
    for header in elf.program_iter() {
        if header.get_type().unwrap() == xmas_elf::program::Type::Load {
            let sect_min = if is_phys {
                header.physical_addr()
            } else {
                header.virtual_addr()
            };
            let sect_max = sect_min + header.mem_size();
            mem_min = mem_min.min(sect_min as _);
            mem_max = mem_max.max(sect_max as _);
        }
    }
    (mem_min, mem_max)
}

fn elf_get_entry_point(elf: &ElfFile) -> usize {
    elf.header.pt2.entry_point() as _
}

fn unpack_elf(elf: &ElfFile, elf_min_vaddr: usize, dest_paddr: usize) {
    for header in elf.program_iter() {
        if header.get_type().unwrap() == xmas_elf::program::Type::Load {
            let seg_dest_paddr = header.virtual_addr() as usize - elf_min_vaddr + dest_paddr;
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

fn load_elf(name: &str, elf: &ElfFile, dest_paddr: usize) -> ImageInfo {
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

    unpack_elf(elf, min_vaddr, dest_paddr);

    ImageInfo {
        phys_region_start: dest_paddr,
        phys_region_end: dest_paddr + image_size,
        virt_region_start: min_vaddr,
        virt_region_end: max_vaddr,
        virt_entry: elf_get_entry_point(elf),
        phys_virt_offset: dest_paddr - min_vaddr,
    }
}

pub fn load_images(max_user_images: usize, bootloader_dtb: *const usize) -> (ImageInfo, ImageInfo) {
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

    let mut kernel_info = None;
    let mut user_info = None;
    let mut app_phys_addr = None;

    for entry in iter_files(cpio) {
        match entry.name() {
            "kernel" => {
                let file = entry.file();
                let kernel_elf = ElfFile::new(file).unwrap();
                let (kernel_phys_start, kernel_phys_end) = elf_get_memory_bounds(&kernel_elf, true);
                kernel_info = Some(load_elf("kernel", &kernel_elf, kernel_phys_start));
                app_phys_addr = Some(round_up!(kernel_phys_end, PAGE_BITS));
            }
            "app" => {
                let app_phys_start = app_phys_addr.unwrap();
                let file = entry.file();
                let app_elf = ElfFile::new(file).unwrap();
                user_info = Some(load_elf("app", &app_elf, app_phys_start));
            }
            _ => panic!("Unknown cpio entry {:#x?}", entry.name()),
        }
    }

    (kernel_info.unwrap(), user_info.unwrap())
}
