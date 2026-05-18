// @depreciated
use std::ffi::CStr;

use anyhow::{Ok, Result, anyhow};

use crate::pager::{
    page::{PAGE_HEADER_SIZE, PAGE_SIZE, Page, PageType},
    pager::Pager,
};

const COLLECTION_PAGE_HEADER_SIZE: usize = PAGE_HEADER_SIZE + 12;

#[derive(Debug, PartialEq, Eq)]
pub struct CollectionView<'a> {
    pub name: &'a str,
    pub root_page: u64,
}

// +-----------------------+---------------------+-------------------+------------------------+
// | num_collections (u32) | next_page_ptr (u64) | Name (\0) | Addr  | Name (\0) | Addr ...   |
// | 4 Bytes               | 8 Bytes             | Var-len + 8 Bytes | Var-len + 8 Bytes      |
// +-----------------------+---------------------+-------------------+------------------------+
// |<----------------- Header (12 Bytes) ------->|<------------------ Data Payload ---------->|

pub struct Collection;

impl Collection {
    pub fn initialize_collection_page(page: &mut Page) {
        page.set_page_type(PageType::CollectionCatalog);

        // no. of collections
        page.data[PAGE_HEADER_SIZE..PAGE_HEADER_SIZE + 4].copy_from_slice(&0u32.to_be_bytes());

        // next page ptr
        page.data[PAGE_HEADER_SIZE + 4..PAGE_HEADER_SIZE + 12].copy_from_slice(&0u64.to_be_bytes());
    }

    pub fn fetch_all_collections<'a>(page_buf: &'a [Page]) -> Result<Vec<CollectionView<'a>>> {
        let mut collections = Vec::new();

        for page in page_buf {
            let data = &page.data;
            let num_collections =
                u32::from_le_bytes(data[PAGE_HEADER_SIZE..PAGE_HEADER_SIZE + 4].try_into()?);

            let mut cursor = COLLECTION_PAGE_HEADER_SIZE;

            for _ in 0..num_collections {
                if cursor >= PAGE_SIZE {
                    return Err(anyhow!("Malformed collection page data: out of bounds"));
                }

                let c_str = CStr::from_bytes_until_nul(&data[cursor..])?;
                let name_bytes = c_str.to_bytes();
                let name = std::str::from_utf8(name_bytes)?;

                cursor += name_bytes.len() + 1;

                if cursor + 8 > PAGE_SIZE {
                    return Err(anyhow!(
                        "Malformed catalog page data: Truncated root page pointer"
                    ));
                }

                let root_page = u64::from_le_bytes(data[cursor..cursor + 8].try_into()?);
                cursor += 8;
                collections.push(CollectionView { name, root_page });
            }
        }

        Ok(collections)
    }

    pub fn load_collection_pages(pager: &mut Pager) -> Result<Vec<Page>> {
        let mut pages = Vec::new();
        let mut current_id: u64 = pager.header.catalog_page;

        while current_id != 0 {
            let page = pager.read_page(current_id)?;
            let next_page_id = u64::from_le_bytes(
                page.data[PAGE_HEADER_SIZE + 4..PAGE_HEADER_SIZE + 12].try_into()?,
            );
            pages.push(page);
            current_id = next_page_id.into();
        }

        Ok(pages)
    }

    pub fn create_collection(
        pager: &mut Pager,
        name: &str,
        documents_root_page: u64,
    ) -> Result<()> {
        if name.contains('\0') || name.is_empty() {
            return Err(anyhow!(
                "Invalid collection name, i found null term or empty"
            ));
        }

        let mut current_id = pager.header.catalog_page;
        let entry_size = name.as_bytes().len() + 1 + 8;

        loop {
            let mut page = pager.read_page(current_id)?;
            let num_collections =
                u32::from_le_bytes(page.data[PAGE_HEADER_SIZE..PAGE_HEADER_SIZE + 4].try_into()?);
            let next_page_ptr = u64::from_le_bytes(
                page.data[PAGE_HEADER_SIZE + 4..PAGE_HEADER_SIZE + 12].try_into()?,
            );
            let used_space = Self::get_payload_end_offset(&page.data, num_collections)?;

            if used_space + entry_size <= PAGE_SIZE {
                let mut write_cursor = used_space;

                page.data[write_cursor..write_cursor + name.as_bytes().len()]
                    .copy_from_slice(name.as_bytes());
                write_cursor += name.as_bytes().len();

                page.data[write_cursor] = 0x00;
                write_cursor += 1;

                page.data[write_cursor..write_cursor + 8]
                    .copy_from_slice(&documents_root_page.to_le_bytes());

                let new_count = num_collections + 1;

                page.data[0..4].copy_from_slice(&new_count.to_le_bytes());

                pager.write_page(&page)?;
                return Ok(());
            }

            if next_page_ptr != 0 {
                current_id = next_page_ptr;
            } else {
                // page is full allocating new page for collections data to store.
                let new_page_id = pager.allocate_page()?;
                page.data[PAGE_HEADER_SIZE + 4..PAGE_HEADER_SIZE + 12]
                    .copy_from_slice(&new_page_id.to_le_bytes());
                pager.write_page(&page)?;

                let mut new_page = Page::new(new_page_id);
                Self::initialize_collection_page(&mut new_page);

                new_page.data[PAGE_HEADER_SIZE..PAGE_HEADER_SIZE + 4]
                    .copy_from_slice(&1u32.to_le_bytes());

                let mut write_cursor = COLLECTION_PAGE_HEADER_SIZE;
                new_page.data[write_cursor..write_cursor + name.as_bytes().len()]
                    .copy_from_slice(name.as_bytes());

                write_cursor += name.as_bytes().len();
                new_page.data[write_cursor] = 0;

                write_cursor += 1;

                new_page.data[write_cursor..write_cursor + 8]
                    .copy_from_slice(&documents_root_page.to_le_bytes());

                pager.write_page(&new_page)?;
                return Ok(());
            }
        }
    }

    pub fn update_collection(
        pager: &mut Pager,
        name: &str,
        new_collection_root_page: u64,
    ) -> Result<()> {
        let mut current_id = pager.header.catalog_page;

        while current_id != 0 {
            let mut page = pager.read_page(current_id)?;
            let num_collections =
                u32::from_le_bytes(page.data[PAGE_HEADER_SIZE..PAGE_HEADER_SIZE + 4].try_into()?);

            let next_page_ptr = u64::from_le_bytes(
                page.data[PAGE_HEADER_SIZE + 4..PAGE_HEADER_SIZE + 12].try_into()?,
            );

            let mut cursor = COLLECTION_PAGE_HEADER_SIZE;

            for _ in 0..num_collections {
                let c_str = CStr::from_bytes_until_nul(&page.data[cursor..])?;
                let name_bytes = c_str.to_bytes();
                let item_name = std::str::from_utf8(name_bytes)?;

                cursor += name_bytes.len() + 1;

                if item_name == name {
                    // Match found! Overwrite the 8-byte pointer value on disk
                    page.data[cursor..cursor + 8]
                        .copy_from_slice(&new_collection_root_page.to_le_bytes());
                    pager.write_page(&page)?;
                    return Ok(());
                }
                cursor += 8;
            }
            current_id = next_page_ptr;
        }

        Err(anyhow!("Collection '{}' not found for update", name))
    }

    pub fn delete_collection(pager: &mut Pager, name: &str) -> Result<()> {
        let mut current_id = pager.header.catalog_page;

        while current_id != 0 {
            let mut page = pager.read_page(current_id)?;
            let num_collections =
                u32::from_le_bytes(page.data[PAGE_HEADER_SIZE..PAGE_HEADER_SIZE + 4].try_into()?);

            let next_page_ptr = u64::from_le_bytes(
                page.data[PAGE_HEADER_SIZE + 4..PAGE_HEADER_SIZE + 12].try_into()?,
            );
            let mut cursor = COLLECTION_PAGE_HEADER_SIZE;

            for _ in 0..num_collections {
                let entry_start = cursor;
                let c_str = CStr::from_bytes_until_nul(&page.data[cursor..])?;
                let name_bytes = c_str.to_bytes();
                let item_name = std::str::from_utf8(name_bytes)?;

                cursor += name_bytes.len() + 1 + 8; // Full size of item segment
                let entry_end = cursor;

                if item_name == name {
                    let used_end = Self::get_payload_end_offset(&page.data, num_collections)?;

                    let bytes_to_shift = used_end - entry_end;

                    if bytes_to_shift > 0 {
                        page.data.copy_within(entry_end..used_end, entry_start);
                    }

                    let cleared_start = used_end - (entry_end - entry_start);

                    page.data[cleared_start..used_end].fill(0);

                    let new_count = num_collections - 1;

                    page.data[PAGE_HEADER_SIZE..PAGE_HEADER_SIZE + 4]
                        .copy_from_slice(&new_count.to_le_bytes());

                    pager.write_page(&page)?;
                    return Ok(());
                }
            }
            current_id = next_page_ptr;
        }

        Err(anyhow!("Collection '{}' not found for deletion", name))
    }

    pub fn get_document_count(pager: &mut Pager, collection_root_page: u64) -> Result<u32> {
        if collection_root_page == 0 {
            return Ok(0);
        }
        let page = pager.read_page(collection_root_page)?;

        let count =
            u32::from_le_bytes(page.data[PAGE_HEADER_SIZE..PAGE_HEADER_SIZE + 4].try_into()?);
        Ok(count)
    }

    fn get_payload_end_offset(data: &[u8; PAGE_SIZE], num_collections: u32) -> Result<usize> {
        let mut cursor = COLLECTION_PAGE_HEADER_SIZE;
        for _ in 0..num_collections {
            if cursor >= PAGE_SIZE {
                return Err(anyhow!("Corrupted tracking entries on size evaluation"));
            }

            let c_str = CStr::from_bytes_until_nul(&data[cursor..])?;
            cursor += c_str.to_bytes().len() + 1 + 8;
        }
        Ok(cursor)
    }
}
