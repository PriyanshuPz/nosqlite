pub const PAGE_SIZE: usize = 4096; // it is same i check with page_size::get()

pub struct Page {
    pub id: u64,
    pub data: [u8; PAGE_SIZE],
}

impl Page {
    pub fn new(id: u64) -> Self {
        Self {
            id,
            data: [0; PAGE_SIZE],
        }
    }
}
