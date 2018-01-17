#![feature(lang_items)]
#![feature(const_fn)]
#![feature(unique)]
#![feature(const_unique_new)]
#![allow(dead_code)]
#![no_std]

extern crate rlibc;
extern crate volatile;
extern crate spin;
extern crate multiboot2;

#[macro_use]
mod vga;
mod memory;


#[macro_use]
extern crate bitflags;


#[no_mangle]
pub extern fn rust_main(multiboot_info_addr: usize) {
    use memory::FrameAllocator;
    // vga::clear();

    let boot_info = unsafe{ multiboot2::load(multiboot_info_addr)};
    let memory_map_tag = boot_info.memory_map_tag()
        .expect("Memory map tag required in multiboot info");

    for area in memory_map_tag.memory_areas() {
        println!("start: 0x{:x}, len: 0x{:x}", area.base_addr, area.length);
    }
    println!("a{}", {println!("xD"); 2});
    println!("sdfa{}sdf", "asd");


    let elf_sections_tag = boot_info.elf_sections_tag()
        .expect("Elf-sections tag required");

    println!("kernel sections:");
    for section in elf_sections_tag.sections() {
        println!("addr: 0x{:x}, size: 0x{:x}, flags: 0x{:x}", 
            section.addr, section.size, section.flags);
    }

    let kernel_start = elf_sections_tag.sections().map(|s| s.addr).min().unwrap();
    let kernel_end = elf_sections_tag.sections().map(|s| s.addr + s.size).max().unwrap();

    let multiboot_start = multiboot_info_addr;
    let multiboot_end = multiboot_start + (boot_info.total_size as usize);
    
    let mut frame_allocator = memory::AreaFrameAllocator::new(
        kernel_start as usize, kernel_end as usize, multiboot_start,
        multiboot_end, memory_map_tag.memory_areas());
    
    println!("{:?}", frame_allocator.allocate_frame());


    loop{}
}

#[lang = "eh_personality"] 
extern fn eh_personality() {}

#[lang = "panic_fmt"] 
#[no_mangle] 
pub extern fn panic_fmt(fmt: core::fmt::Arguments, file: &'static str, line: u32) -> ! 
{  
    println!("\n\nPANIC in {} at line {}:", file, line);
    println!("     {}", fmt);
    loop{}
}