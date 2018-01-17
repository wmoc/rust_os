use memory::PAGE_SIZE;

const ENTRY_COUNT: usize = 512;


mod entry;
mod table;

pub type PAddr = usize; // physical address
pub type VAddr = usize; // virtual address

pub struct Page{
    number: usize,
}

