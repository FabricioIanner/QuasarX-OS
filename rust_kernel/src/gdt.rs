use lazy_static::lazy_static;
use x86_64::structures::gdt::{GlobalDescriptorTable, Descriptor, SegmentSelector};
use x86_64::structures::tss::TaskStateSegment;
use x86_64::VirtAddr;
use spin::Once;

pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;

static mut STACK: [u8; 4096 * 5] = [0; 4096 * 5];

lazy_static! {
    static ref TSS: TaskStateSegment = {
        let mut tss = TaskStateSegment::new();

        // Define uma pilha dedicada para double fault
        let stack_start = VirtAddr::from_ptr(unsafe { &STACK });
        let stack_end = stack_start + STACK.len();
        tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = stack_end;

        tss
    };
}

lazy_static! {
    static ref GDT: (GlobalDescriptorTable, Selectors) = {
        let mut gdt = GlobalDescriptorTable::new();

        let code_selector = gdt.add_entry(Descriptor::kernel_code_segment());
        let tss_selector = gdt.add_entry(Descriptor::tss_segment(&TSS));

        (gdt, Selectors { code_selector, tss_selector })
    };
}

pub struct Selectors {
    pub code_selector: SegmentSelector,
    pub tss_selector: SegmentSelector,
}

pub fn init() {
    use x86_64::instructions::segmentation::set_cs;
    use x86_64::instructions::tables::load_tss;

    GDT.0.load();
    unsafe {
        set_cs(GDT.1.code_selector);
        load_tss(GDT.1.tss_selector);
    }
}
