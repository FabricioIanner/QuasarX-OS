#![no_main]
#![no_std]

use rust_kernel::kernel_main;
use core::panic::PanicInfo;
use fat12::Fat12Volume;
use vfs::VFS_INSTANCE;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    kernel_main();
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

let fat = Box::leak(Box::new(Fat12Volume::new()));
VFS_INSTANCE.lock().mount(fat);

if let Some(mut file) = VFS_INSTANCE.lock().open("HELLOTXT") {
    vga_println!("Arquivo: {}", file.name);
    while let Some(b) = file.read_byte() {
        use crate::vga_buffer::vga_print;
        if b == b'\n' {
            vga_println!();
        } else {
            vga_print!("{}", b as char);
        }
    }
} else {
    vga_println!("Arquivo não encontrado");
}

vga_println!("Conteúdo do diretório '/':");

if let Some(dir) = VFS_INSTANCE.lock().list_dir("/") {
    for entry in dir.entries {
        if entry.is_dir {
            vga_println!("<DIR> {}", entry.name);
        } else {
            vga_println!("     {} ({} bytes)", entry.name, entry.size);
        }
    }
}

#![no_std]
#![no_main]

use core::panic::PanicInfo;

#[no_mangle]
pub extern "C" fn main() {
    let vga = 0xb8000 as *mut u8;
    let msg = b"Hello from exec!";

    for (i, &b) in msg.iter().enumerate() {
        unsafe {
            *vga.add(i * 2) = b;
            *vga.add(i * 2 + 1) = 0x0F;
        }
    }
}

#[panic_handler]
fn panic(_: &PanicInfo) -> ! {
    loop {}
}
