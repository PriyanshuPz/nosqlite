use std::ffi::CStr;

use anyhow::{Result, anyhow};

use crate::pager::{
    page::{PAGE_HEADER_SIZE, PAGE_SIZE, Page, PageType},
    pager::Pager,
};

//
// Catalog page layout:
//
// +-------------------------------+
// | Page Header                   |
// +-------------------------------+
// | collection_count (u32)        |
// +-------------------------------+
// | entries...                    |
// +-------------------------------+
//
// Entry layout:
//
// +-------------------------------+
// | collection_name (\0)          |
// +-------------------------------+
// | first_document_page (u64)     |
// +-------------------------------+
// | document_count (u64)          |
// +-------------------------------+
//

pub const CATALOG_HEADER_SIZE: usize = PAGE_HEADER_SIZE + 4;

#[derive(Debug, Clone)]
pub struct CatalogEntry {
    pub name: String,
    pub first_document_page: u64,
    pub document_count: u64,
}

pub struct Catalog;

impl Catalog {
    pub fn initialize(page: &mut Page) {
        page.set_page_type(PageType::CollectionCatalog);

        // collection_count = 0
        page.data[PAGE_HEADER_SIZE..PAGE_HEADER_SIZE + 4].copy_from_slice(&0u32.to_le_bytes());
    }

    pub fn list(pager: &mut Pager) -> Result<Vec<CatalogEntry>> {
        let page = pager.read_page(pager.header.catalog_page)?;

        let count = Self::collection_count(&page)?;

        let mut cursor = CATALOG_HEADER_SIZE;

        let mut entries = Vec::new();

        for _ in 0..count {
            let c_str = CStr::from_bytes_until_nul(&page.data[cursor..])?;

            let name_bytes = c_str.to_bytes();

            let name = std::str::from_utf8(name_bytes)?.to_string();

            cursor += name_bytes.len() + 1;

            let first_document_page = u64::from_le_bytes(page.data[cursor..cursor + 8].try_into()?);
            cursor += 8;

            let document_count = u64::from_le_bytes(page.data[cursor..cursor + 8].try_into()?);
            cursor += 8;

            entries.push(CatalogEntry {
                name,
                first_document_page,
                document_count,
            });
        }

        Ok(entries)
    }

    pub fn find(pager: &mut Pager, collection_name: &str) -> Result<Option<CatalogEntry>> {
        let entries = Self::list(pager)?;

        for entry in entries {
            if entry.name == collection_name {
                return Ok(Some(entry));
            }
        }

        Ok(None)
    }

    pub fn create(pager: &mut Pager, name: &str) -> Result<()> {
        if name.is_empty() || name.contains('\0') {
            return Err(anyhow!("invalid collection name"));
        }

        if Self::find(pager, name)?.is_some() {
            return Err(anyhow!("collection already exists"));
        }

        let mut page = pager.read_page(pager.header.catalog_page)?;

        let count = Self::collection_count(&page)?;

        let used_space = Self::used_space(&page)?;

        //
        // name + null byte
        // first_document_page
        // document_count
        //
        let entry_size = name.as_bytes().len() + 1 + 8 + 8;

        if used_space + entry_size > PAGE_SIZE {
            return Err(anyhow!("catalog page full"));
        }

        let mut cursor = used_space;

        page.data[cursor..cursor + name.len()].copy_from_slice(name.as_bytes());

        cursor += name.len();

        page.data[cursor] = 0;

        cursor += 1;

        page.data[cursor..cursor + 8].copy_from_slice(&0u64.to_le_bytes());

        cursor += 8;

        page.data[cursor..cursor + 8].copy_from_slice(&0u64.to_le_bytes());

        let new_count = count + 1;

        page.data[PAGE_HEADER_SIZE..PAGE_HEADER_SIZE + 4].copy_from_slice(&new_count.to_le_bytes());

        pager.write_page(&page)?;

        Ok(())
    }

    pub fn update(pager: &mut Pager, updated: &CatalogEntry) -> Result<()> {
        let mut page = pager.read_page(pager.header.catalog_page)?;
        let count = Self::collection_count(&page)?;
        let mut cursor = CATALOG_HEADER_SIZE;

        for _ in 0..count {
            let c_str = CStr::from_bytes_until_nul(&page.data[cursor..])?;
            let name_bytes = c_str.to_bytes();
            let name = std::str::from_utf8(name_bytes)?;
            cursor += name_bytes.len() + 1;

            if name == updated.name {
                page.data[cursor..cursor + 8]
                    .copy_from_slice(&updated.first_document_page.to_le_bytes());

                cursor += 8;

                page.data[cursor..cursor + 8]
                    .copy_from_slice(&updated.document_count.to_le_bytes());

                pager.write_page(&page)?;

                return Ok(());
            }

            // Skip:
            // first_document_page
            // document_count
            cursor += 8 + 8;
        }

        Err(anyhow!("collection not found"))
    }

    pub fn delete(pager: &mut Pager, collection_name: &str) -> Result<()> {
        let mut page = pager.read_page(pager.header.catalog_page)?;

        let count = Self::collection_count(&page)?;

        let mut cursor = CATALOG_HEADER_SIZE;

        for _ in 0..count {
            let entry_start = cursor;

            let c_str = CStr::from_bytes_until_nul(&page.data[cursor..])?;

            let name_bytes = c_str.to_bytes();

            let name = std::str::from_utf8(name_bytes)?;

            cursor += name_bytes.len() + 1 + 8 + 8;

            let entry_end = cursor;

            if name == collection_name {
                let used_space = Self::used_space(&page)?;

                let bytes_to_shift = used_space - entry_end;

                if bytes_to_shift > 0 {
                    page.data.copy_within(entry_end..used_space, entry_start);
                }

                let cleared_start = used_space - (entry_end - entry_start);

                page.data[cleared_start..used_space].fill(0);

                let new_count = count - 1;

                page.data[PAGE_HEADER_SIZE..PAGE_HEADER_SIZE + 4]
                    .copy_from_slice(&new_count.to_le_bytes());

                pager.write_page(&page)?;

                return Ok(());
            }
        }

        Err(anyhow!("collection not found"))
    }

    fn collection_count(page: &Page) -> Result<u32> {
        Ok(u32::from_le_bytes(
            page.data[PAGE_HEADER_SIZE..PAGE_HEADER_SIZE + 4].try_into()?,
        ))
    }

    fn used_space(page: &Page) -> Result<usize> {
        let count = Self::collection_count(page)?;

        let mut cursor = CATALOG_HEADER_SIZE;

        for _ in 0..count {
            let c_str = CStr::from_bytes_until_nul(&page.data[cursor..])?;

            cursor += c_str.to_bytes().len() + 1 + 8 + 8;
        }

        Ok(cursor)
    }
}
