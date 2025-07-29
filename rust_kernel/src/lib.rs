#![no_std]
#![no_main]

pub fn kernel_main() {
    // Aqui começa a execução do seu kernel.
    // Pode iniciar inicialização de hardware, terminal etc.
}

extern crate bootloader;
extern crate alloc;

use bootloader::{entry_point, BootInfo};

mod vga_buffer;
mod interrupts;
mod timer;
mod keyboard;
mod memory;
mod allocator;
mod gdt;
mod fat12;
mod vfs;
mod shell;

use core::panic::PanicInfo;
use x86_64::instructions::interrupts as x86_interrupts;
use x86_64::VirtAddr;
use x86_64::structures::paging::Page;
use x86_64::structures::paging::OffsetPageTable;
use x86_64::registers::control::Cr3;
use memory::{init_heap, BootInfoFrameAllocator, HEAP_START, HEAP_SIZE};
use allocator::init_heap_allocator;
use crate::gdt::init as init_gdt;
use crate::interrupts::{init_idt, PICS};
use crate::timer::init_pit;
use crate::vga_buffer::vga_println;
use fat12::Fat12Volume;
use shell::run_shell;

#[no_mangle]
#pub extern "C" fn _start() -> ! {
#    vga_println!("Kernel em Rust - Timer Init");
#
#      init_idt();
#    unsafe { PICS.lock().initialize() };
#
#    init_pit();
#    x86_interrupts::enable();
#
#    loop {}
#
#   vga_println!("Terminal VGA funcionando.");
#    loop {}
#        vga_println!("Inicializando IDT...");
#    init_idt();
#    
#    // Disparando int3 manualmente para testar
#    x86_64::instructions::interrupts::int3();
#
#    loop {}
#}

entry_point!(kernel_main);

/// Retorna a tabela de páginas a partir do endereço físico base
fn init_mapper(physical_memory_offset: VirtAddr) -> OffsetPageTable<'static> {
    use x86_64::structures::paging::PageTable;

    let (level_4_page_table_frame, _) = Cr3::read();

    let phys = level_4_page_table_frame.start_address();
    let virt = physical_memory_offset + phys.as_u64();
    let page_table_ptr: *mut PageTable = virt.as_mut_ptr();

    unsafe { OffsetPageTable::new(&mut *page_table_ptr, physical_memory_offset) }
}

fn kernel_main(boot_info: &'static BootInfo) -> ! {
    vga_println!("Kernel iniciado!");
    vga_println!("Endereço do BootInfo: {:?}", boot_info);

    init_gdt();
    init_idt();

    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { init_mapper(phys_mem_offset) };


    use memory::BootInfoFrameAllocator;

    let mut frame_allocator = unsafe {
        BootInfoFrameAllocator::init(&boot_info.memory_map)
    };

    // Aloca alguns frames só para testar
    for _ in 0..5 {
    if let Some(frame) = frame_allocator.allocate_frame() {
    vga_println!("Frame alocado: {:?}", frame.start_address());

    let fat = Fat12Volume::new();
    vga_println!("Listando root dir:");
    fat.list_root_dir();

    vga_println!("---");
    fat.read_file_contents("HELLOTXT");

    loop {}   
    }

    init_heap(&mut mapper, &mut frame_allocator)
    .expect("Falha ao mapear heap");

    init_heap_allocator(HEAP_START as usize, HEAP_SIZE);

    use alloc::{boxed::Box, vec::Vec};

    let heap_val = Box::new(42);
    vga_println!("Box alocado: {}", heap_val);

    let mut vec = Vec::new();
    for i in 0..10 {
        vec.push(i * i);
    }
    vga_println!("Vec: {:?}", vec);

    loop {}
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    vga_println!("Panic: {}", info);
    loop {}
}

vga_println!("Iniciando shell:");
run_shell();