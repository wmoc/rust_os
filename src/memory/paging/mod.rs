use core::ptr::Unique;
use memory::{PAGE_SIZE, FrameAllocator, Frame};
use self::table::*;
pub use self::entry::*;

const ENTRY_COUNT: usize = 512;

mod entry;
mod table;

pub type PAddr = usize;
// physical address
pub type VAddr = usize; // virtual address

pub struct Page {
    number: usize,
}

impl Page {
    pub fn containing_address(addr: VAddr) -> Page {
        assert!(addr < 0x0000_8000_0000_0000 || addr > 0xffff_8000_0000_0000,
                "Invalid address: 0x{:x}", addr);

        Page { number: addr / PAGE_SIZE }
    }

    fn start_address(&self) -> usize {
        self.number * PAGE_SIZE
    }

    fn p4_index(&self) -> usize {
        (self.number >> 27) & 0x1FF
    }

    fn p3_index(&self) -> usize {
        (self.number >> 18) & 0x1FF
    }

    fn p2_index(&self) -> usize {
        (self.number >> 9) & 0x1FF
    }

    fn p1_index(&self) -> usize {
        (self.number >> 0) & 0x1FF
    }
}

pub struct ActivePageTable {
    p4: Unique<Table<Level4>>,
}


impl ActivePageTable {
    pub unsafe fn new() -> ActivePageTable {
        ActivePageTable {
            p4: Unique::new_unchecked(table::P4),
        }
    }

    fn p4(&self) -> &Table<Level4> {
        unsafe { self.p4.as_ref() }
    }

    fn p4_mut(&mut self) -> &mut Table<Level4> {
        unsafe { self.p4.as_mut() }
    }

    pub fn map_to<A: FrameAllocator>(&mut self, page: Page, frame: Frame, flags: EntryFlags, allocator: &mut A) {
        let p4 = self.p4_mut();
        let p3 = p4.next_table_create(page.p4_index(), allocator);
        let p2 = p3.next_table_create(page.p3_index(), allocator);
        let p1 = p2.next_table_create(page.p2_index(), allocator);

        assert!(p1[page.p1_index()].is_unused());
        p1[page.p1_index()].set(frame, flags | PRESENT);
    }

    pub fn map<A: FrameAllocator>(&mut self, page: Page, flags: EntryFlags, allocator: &mut A) {
        let frame = allocator.allocate_frame().expect("Out of memory");
        self.map_to(page, frame, flags, allocator)
    }

    pub fn identity_map<A: FrameAllocator>(&mut self, frame: Frame, flags: EntryFlags, allocator: &mut A) {
        let page = Page::containing_address(frame.start_address());
        self.map_to(page, frame, flags, allocator)
    }

    pub fn translate(&self, v_addr: VAddr) -> Option<PAddr> {
        let offset = v_addr % PAGE_SIZE;
        self.translate_page(Page::containing_address(v_addr))
            .map(|frame| frame.number * PAGE_SIZE + offset)
    }

    fn unmap<A: FrameAllocator>(&mut self, page: Page, allocator: &mut A){
        assert!(self.translate(page.start_address()).is_some());

        let p1 = self.p4_mut()
                .next_table_mut(page.p4_index())
                .and_then(|p3| p3.next_table_mut(page.p3_index()))
                .and_then(|p2| p2.next_table_mut(page.p2_index()))
                .expect("mapping does not support huge pages");

        let frame = p1[page.p1_index()].pointed_frame().unwrap();
        p1[page.p1_index()].set_unused();

        use x86_64::instructions::tlb;
        use x86_64::VirtualAddress;

        tlb::flush(VirtualAddress(page.start_address()))
        // todo free immediate tables if empty
        // allocator.deallocate_frame(frame);
    }

    fn translate_page(&self, page: Page) -> Option<Frame> {
        use self::entry::HUGE_PAGE;

        let p3 = self.p4().next_table(page.p4_index());

        // returns frame if address belongs to a huge page
        let huge_page = || {
            p3.and_then(|p3| {
                let p3_entry = &p3[page.p3_index()];

                if let Some(start_frame) = p3_entry.pointed_frame() {
                    if p3_entry.flags().contains(HUGE_PAGE) {
                        // address must be 1GiB aligned
                        assert!(start_frame.number % (ENTRY_COUNT * ENTRY_COUNT) == 0);

                        return Some(Frame {
                            number: start_frame.number + page.p2_index() * ENTRY_COUNT + page.p1_index(),
                        });
                    }
                }

                if let Some(p2) = p3.next_table(page.p3_index()) {
                    let p2_entry = &p2[page.p2_index()];

                    if let Some(start_frame) = p2_entry.pointed_frame() {
                        if p2_entry.flags().contains(HUGE_PAGE){
                            // 2MiB page
                            assert!(start_frame.number % ENTRY_COUNT == 0);
                            return Some(Frame {
                                number: start_frame.number + page.p1_index(),
                            });
                        }
                    }
                }

                None
            })
        };

        p3.and_then(|p3| p3.next_table(page.p3_index()))
            .and_then(|p2| p2.next_table(page.p2_index()))
            .and_then(|p1| p1[page.p1_index()].pointed_frame())
            .or_else(huge_page)
    }
}


pub fn test_paging<A: FrameAllocator>(allocator: &mut A) {
    let mut page_table = unsafe { ActivePageTable::new() };

    let addr = 42 * 512 * 512 * 4096;
    let page = Page::containing_address(addr);
    let frame = allocator.allocate_frame().unwrap();
    println!("None = {:?}, map to {:?}", page_table.translate(addr), frame);
    page_table.map_to(page, frame, EntryFlags::empty(), allocator);
    println!("Some = {:?}", page_table.translate(addr));
    page_table.unmap(Page::containing_address(addr), allocator);
    println!("None = {:?}", page_table.translate(addr));

    println!("Next free frame: {:?}", allocator.allocate_frame());
    println!("Next free frame: {:?}", allocator.allocate_frame());

}