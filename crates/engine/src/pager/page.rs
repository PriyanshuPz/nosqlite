pub const PAGE_SIZE: usize = 4096; // it is same i check with page_size::get()

pub const PAGE_HEADER_SIZE: usize = 16;

#[repr(u8)]
pub enum PageType {
    Free = 0,
    CollectionCatalog = 1,
    CollectionData = 2,
}

impl TryFrom<u8> for PageType {
    type Error = &'static str;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Free),
            1 => Ok(Self::CollectionCatalog),
            2 => Ok(Self::CollectionData),
            _ => Err("invalid page type"),
        }
    }
}

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

    pub fn set_page_type(&mut self, page_type: PageType) {
        self.data[0] = page_type as u8;
    }

    pub fn page_type(&self) -> Result<PageType, &'static str> {
        PageType::try_from(self.data[0])
    }

    pub fn set_next_page(&mut self, next_page: u64) {
        self.data[1..9].copy_from_slice(&next_page.to_le_bytes());
    }

    pub fn next_page(&self) -> u64 {
        u64::from_le_bytes(self.data[1..9].try_into().expect("invalid next page bytes"))
    }
}
