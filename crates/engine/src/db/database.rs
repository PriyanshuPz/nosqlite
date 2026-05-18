use std::collections::HashMap;
use std::path::Path;

use anyhow::{Result, anyhow};

use crate::db::collections::Collection;
use crate::pager::page::{PAGE_HEADER_SIZE, Page, PageType};
use crate::pager::pager::Pager;

pub const PAGE_TYPE_OFFSET: usize = 0;

pub const COLLECTION_COUNT_OFFSET: usize = PAGE_HEADER_SIZE;

pub const COLLECTION_NEXT_PAGE_OFFSET: usize = PAGE_HEADER_SIZE + 4;

pub struct Database {
    pager: Pager,
    collections: HashMap<String, u64>,
}

#[derive(Debug, Clone)]
pub struct CollectionInfo {
    pub name: String,
    pub root_page: u64,
    pub document_count: u32,
}

impl Database {
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let mut pager = Pager::open(path)?;
        let mut collections = HashMap::new();

        let collection_pages = Collection::load_collection_pages(&mut pager)?;
        let views = Collection::fetch_all_collections(&collection_pages)?;

        for view in views {
            collections.insert(view.name.to_string(), view.root_page);
        }

        Ok(Self { pager, collections })
    }

    pub fn list_collections(&mut self) -> Result<Vec<CollectionInfo>> {
        let mut list = vec![];
        for (name, root) in self.collections.iter() {
            let doc_count = Collection::get_document_count(&mut self.pager, root.to_owned())?;
            let view = CollectionInfo {
                document_count: doc_count,
                name: name.to_string(),
                root_page: root.to_owned(),
            };
            list.push(view);
        }

        list.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(list)
    }

    pub fn create_collection(&mut self, name: &str) -> Result<()> {
        if self.collections.contains_key(name) {
            return Err(anyhow!("Collection '{}' already exists", name));
        }

        let collection_root_page = 0;
        let mut page = Page::new(collection_root_page);
        page.set_page_type(PageType::CollectionData);

        page.data[COLLECTION_COUNT_OFFSET..COLLECTION_NEXT_PAGE_OFFSET]
            .copy_from_slice(&0u32.to_le_bytes());
        self.pager.write_page(&page)?;

        Collection::create_collection(&mut self.pager, name, collection_root_page)?;

        self.collections
            .insert(name.to_string(), collection_root_page);
        Ok(())
    }

    pub fn update_collection(&mut self, name: &str, new_collection_root_page: u64) -> Result<()> {
        if !self.collections.contains_key(name) {
            return Err(anyhow!("Collection '{}' does not exist", name));
        }

        Collection::update_collection(&mut self.pager, name, new_collection_root_page)?;

        self.collections
            .insert(name.to_string(), new_collection_root_page);
        Ok(())
    }

    pub fn delete_collection(&mut self, name: &str) -> Result<()> {
        if !self.collections.contains_key(name) {
            return Err(anyhow!("Collection '{}' does not exist", name));
        }

        Collection::delete_collection(&mut self.pager, name)?;

        self.collections.remove(name);
        Ok(())
    }
}
