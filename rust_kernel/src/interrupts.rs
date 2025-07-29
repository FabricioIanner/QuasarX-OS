use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};
use lazy_static::lazy_static;
use crate::gdt::DOUBLE_FAULT_IST_INDEX;
use crate::vga_println;
use crate::timer;
use crate::keyboard::{read_scancode, scancode_to_ascii};

use pic8259::ChainedPics;
use spin::Mutex;

pub static PICS: Mutex<ChainedPics> = Mutex::new(unsafe {
    ChainedPics::new(0x20, 0x28) // PIC master/slave
});

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();

        // Handler de breakpoint (int3)
        idt.breakpoint.set_handler_fn(breakpoint_handler);

        idt.double_fault
            .set_handler_fn(double_fault_handler)
            .set_stack_index(DOUBLE_FAULT_IST_INDEX);


        // Futuro: idt[32].set_handler_fn(timer_interrupt);
        idt[32].set_handler_fn(timer_interrupt_handler); // IRQ0
        // Futuro: idt[33].set_handler_fn(keyboard_interrupt);
        idt[33].set_handler_fn(keyboard_interrupt_handler); // IRQ1
        idt
    };
}

pub fn init_idt() {
    IDT.load();
}

/// Notifica ao PIC que IRQ0 foi tratada
fn send_eoi(irq: u8) {
    unsafe { PICS.lock().notify_end_of_interrupt(irq); }
}

extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    vga_println!("Interrupção: Breakpoint");
    vga_println!("{:#?}", stack_frame);
}

static mut TICKS: u64 = 0;

extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: InterruptStackFrame) {
    unsafe {
        TICKS += 1;
        if TICKS % 100 == 0 {
            vga_println!("100 ticks (~1s)");
        }
    }
    send_eoi(32); // IRQ0
}

extern "x86-interrupt" fn keyboard_interrupt_handler(_stack_frame: InterruptStackFrame) {
    let scancode = read_scancode();

    if let Some(c) = scancode_to_ascii(scancode) {
        use crate::vga_buffer::vga_print;
        vga_print!("{}", c);
    }

    send_eoi(33); // IRQ1
}

extern "x86-interrupt" fn double_fault_handler(
    stack_frame: InterruptStackFrame,
    _error_code: u64,
) -> ! {
    vga_println!("EXCEÇÃO: DOUBLE FAULT");
    vga_println!("{:#?}", stack_frame);
    loop {}
}