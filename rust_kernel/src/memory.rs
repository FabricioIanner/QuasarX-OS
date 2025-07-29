use bootloader::bootinfo::{MemoryMap, MemoryRegionType};
use x86_64::{
    PhysAddr,
    VirtAddr,
    structures::paging::{
        FrameAllocator, Mapper, Page, PageTableFlags, PhysFrame, Size4KiB,
    }
};

/// Um alocador simples de quadros físicos com base no mapa de memória fornecido
pub struct BootInfoFrameAllocator {
    memory_map: &'static MemoryMap,
    next: usize,
}

impl BootInfoFrameAllocator {
    /// Cria uma instância a partir do mapa de memória
    pub unsafe fn init(memory_map: &'static MemoryMap) -> Self {
        BootInfoFrameAllocator {
            memory_map,
            next: 0,
        }
    }

    /// Retorna uma lista de todos os frames físicos usáveis (de 4KiB)
    fn usable_frames(&self) -> impl Iterator<Item = PhysFrame> {
        let regions = self.memory_map.iter();

        let usable = regions
            .filter(|r| r.region_type == MemoryRegionType::Usable);

        let addr_ranges = usable
            .map(|r| r.range.start_addr()..r.range.end_addr());

        let frame_addresses = addr_ranges.flat_map(|r| r.step_by(4096));

        frame_addresses.map(|addr| {
            PhysFrame::containing_address(PhysAddr::new(addr))
        })
    }
}

unsafe impl FrameAllocator<Size4KiB> for BootInfoFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        let frame = self.usable_frames().nth(self.next);
        self.next += 1;
        frame
    }
}

pub const HEAP_START: u64 = 0x_4444_4444_0000;
pub const HEAP_SIZE: usize = 100 * 1024; // 100 KiB

/// Mapeia a região do heap virtual para frames físicos
pub fn init_heap(
    mapper: &mut impl Mapper<Size4KiB>,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>,
) -> Result<(), &'static str> {
    let page_range = {
        let start = VirtAddr::new(HEAP_START);
        let end = start + HEAP_SIZE - 1u64;
        let start_page = Page::containing_address(start);
        let end_page = Page::containing_address(end);
        Page::range_inclusive(start_page, end_page)
    };

    for page in page_range {
        let frame = frame_allocator
            .allocate_frame()
            .ok_or("Frame físico insuficiente")?;

        let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
        unsafe {
            mapper.map_to(page, frame, flags, frame_allocator)?
                .flush();
        }
    }

    Ok(())
}