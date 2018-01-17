pub use self::area_frame_allocator::AreaFrameAllocator;
use self::paging::PAddr;

mod area_frame_allocator;
mod paging;


pub const PAGE_SIZE: usize = 4096;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Frame {
    number: usize,
}

pub trait FrameAllocator {
    fn allocate_frame(&mut self) -> Option<Frame>;
    fn deallocate_frame(&mut self, frame: Frame);
}


impl Frame {

    fn containing_address(address: usize) -> Frame {
        Frame { number: address / PAGE_SIZE }
    }

    fn start_address(&self) -> PAddr {
        self.number * PAGE_SIZE
    }
}