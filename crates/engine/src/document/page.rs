use anyhow::{Result, anyhow};

use crate::pager::page::{PAGE_HEADER_SIZE, PAGE_SIZE, Page, PageType};

// Document page layout:
//
// +-----------------------------------+
// | Generic Page Header               |
// +-----------------------------------+
// | next_page (u64)                   |
// +-----------------------------------+
// | document_count (u32)              |
// +-----------------------------------+
// | used_space (u32)                  |
// +-----------------------------------+
// | BSON bytes...                     |
// | BSON bytes...                     |
// | BSON bytes...                     |
// +-----------------------------------+

pub const NEXT_PAGE_OFFSET: usize = PAGE_HEADER_SIZE;
pub const DOCUMENT_COUNT_OFFSET: usize = NEXT_PAGE_OFFSET + 8;
pub const USED_SPACE_OFFSET: usize = DOCUMENT_COUNT_OFFSET + 4;
pub const DOCUMENT_HEADER_SIZE: usize = USED_SPACE_OFFSET + 4;

pub struct DocumentPage;

impl DocumentPage {
    pub fn initialize(page: &mut Page) {
        page.set_page_type(PageType::CollectionData);

        // next_page = 0
        // document_count = 0
        // used_space starts after header

        page.data[NEXT_PAGE_OFFSET..NEXT_PAGE_OFFSET + 8].copy_from_slice(&0u64.to_le_bytes());

        page.data[DOCUMENT_COUNT_OFFSET..DOCUMENT_COUNT_OFFSET + 4]
            .copy_from_slice(&0u32.to_le_bytes());

        page.data[USED_SPACE_OFFSET..USED_SPACE_OFFSET + 4]
            .copy_from_slice(&(DOCUMENT_HEADER_SIZE as u32).to_le_bytes());
    }

    pub fn next_page(page: &Page) -> Result<u64> {
        Ok(u64::from_le_bytes(
            page.data[NEXT_PAGE_OFFSET..NEXT_PAGE_OFFSET + 8].try_into()?,
        ))
    }

    pub fn set_next_page(page: &mut Page, next_page: u64) {
        page.data[NEXT_PAGE_OFFSET..NEXT_PAGE_OFFSET + 8].copy_from_slice(&next_page.to_le_bytes());
    }

    pub fn document_count(page: &Page) -> Result<u32> {
        Ok(u32::from_le_bytes(
            page.data[DOCUMENT_COUNT_OFFSET..DOCUMENT_COUNT_OFFSET + 4].try_into()?,
        ))
    }

    pub fn set_document_count(page: &mut Page, count: u32) {
        page.data[DOCUMENT_COUNT_OFFSET..DOCUMENT_COUNT_OFFSET + 4]
            .copy_from_slice(&count.to_le_bytes());
    }

    pub fn used_space(page: &Page) -> Result<usize> {
        Ok(
            u32::from_le_bytes(page.data[USED_SPACE_OFFSET..USED_SPACE_OFFSET + 4].try_into()?)
                as usize,
        )
    }

    pub fn set_used_space(page: &mut Page, used: usize) {
        page.data[USED_SPACE_OFFSET..USED_SPACE_OFFSET + 4]
            .copy_from_slice(&(used as u32).to_le_bytes());
    }

    pub fn remaining_space(page: &Page) -> Result<usize> {
        let used = Self::used_space(page)?;

        Ok(PAGE_SIZE - used)
    }

    pub fn append_document(page: &mut Page, bson_bytes: &[u8]) -> Result<()> {
        // Appends raw BSON bytes.
        // BSON already contains its own
        // internal size prefix.
        let remaining = Self::remaining_space(page)?;

        if bson_bytes.len() > remaining {
            return Err(anyhow!("document page full"));
        }

        let used = Self::used_space(page)?;

        // Copy BSON bytes
        // Update used space
        // Increment document count

        page.data[used..used + bson_bytes.len()].copy_from_slice(bson_bytes);

        Self::set_used_space(page, used + bson_bytes.len());

        let count = Self::document_count(page)?;

        Self::set_document_count(page, count + 1);

        Ok(())
    }

    pub fn documents(page: &Page) -> Result<Vec<&[u8]>> {
        let mut documents = Vec::new();

        let used = Self::used_space(page)?;

        let mut cursor = DOCUMENT_HEADER_SIZE;

        while cursor < used {
            // BSON document size
            // stored in first 4 bytes
            let bson_size = i32::from_le_bytes(page.data[cursor..cursor + 4].try_into()?) as usize;

            if bson_size == 0 {
                return Err(anyhow!("invalid bson size"));
            }

            if cursor + bson_size > used {
                return Err(anyhow!("corrupted bson document"));
            }

            documents.push(&page.data[cursor..cursor + bson_size]);

            cursor += bson_size;
        }

        Ok(documents)
    }
}
