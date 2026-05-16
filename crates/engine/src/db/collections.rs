use std::ffi::CStr;

use anyhow::{Result, anyhow};

use crate::pager::{
    page::{PAGE_SIZE, Page},
    pager::Pager,
};

const HEADER_SIZE: usize = 12;

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
    pub fn fetch_all_collections<'a>(page_buf: &'a [Page]) -> Result<Vec<CollectionView<'a>>> {
        let mut collections = Vec::new();

        for page in page_buf {
            let data = &page.data;
            let num_collections = u32::from_le_bytes(data[0..4].try_into()?);

            let mut cursor = HEADER_SIZE;

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

    pub fn load_collection_pages(pager: &mut Pager, root_page_id: u64) -> Result<Vec<Page>> {
        let mut pages = Vec::new();
        let mut current_id = root_page_id;

        while current_id != 0 {
            let page = pager.read_page(current_id)?;
            let next_page_id = u64::from_le_bytes(page.data[4..12].try_into()?);
            pages.push(page);
            current_id = next_page_id;
        }

        Ok(pages)
    }

    pub fn create_collection(
        pager: &mut Pager,
        root_page_id: u64,
        name: &str,
        collection_root_page: u64,
        next_available_page_id: &mut u64,
    ) -> Result<()> {
        if name.contains('\0') || name.is_empty() {
            return Err(anyhow!(
                "Invalid collection name, i found null term or empty"
            ));
        }

        let mut current_id = root_page_id;
        let entry_size = name.as_bytes().len() + 1 + 8;

        loop {
            let mut page = pager.read_page(current_id)?;
            let num_collections = u32::from_le_bytes(page.data[0..4].try_into()?);
        }

        Ok(())
    }
}
