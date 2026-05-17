use std::collections::HashMap;
use std::path::Path;

use anyhow::{Result, anyhow};

use crate::db::collections::Collection;
use crate::pager::pager::Pager;

pub struct Database {
    pager: Pager,
    collections: HashMap<String, u64>,
    collection_root_page_id: u64,
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
        let meta = pager.check_metadata()?;
        let mut collections = HashMap::new();

        let collection_pages = Collection::load_collection_pages(&mut pager, meta.root_page)?;
        let views = Collection::fetch_all_collections(&collection_pages)?;

        for view in views {
            collections.insert(view.name.to_string(), view.root_page);
        }

        Ok(Self {
            pager,
            collections,
            collection_root_page_id: meta.root_page.into(),
        })
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
        // TODO: Implement a way to automatically get ptr to empty page.
        let doc_page_ptr = 4;

        // Sync to physical disk blocks via collection layer
        Collection::create_collection(
            &mut self.pager,
            self.collection_root_page_id,
            name,
            doc_page_ptr,
            &mut 9,
        )?;

        // Update working memory cache if disk transaction succeeds
        self.collections.insert(name.to_string(), doc_page_ptr);
        Ok(())
    }

    pub fn update_collection(&mut self, name: &str, new_collection_root_page: u64) -> Result<()> {
        if !self.collections.contains_key(name) {
            return Err(anyhow!("Collection '{}' does not exist", name));
        }

        Collection::update_collection(
            &mut self.pager,
            self.collection_root_page_id,
            name,
            new_collection_root_page,
        )?;

        self.collections
            .insert(name.to_string(), new_collection_root_page);
        Ok(())
    }

    pub fn delete_collection(&mut self, name: &str) -> Result<()> {
        if !self.collections.contains_key(name) {
            return Err(anyhow!("Collection '{}' does not exist", name));
        }

        Collection::delete_collection(&mut self.pager, self.collection_root_page_id, name)?;

        self.collections.remove(name);
        Ok(())
    }
}
