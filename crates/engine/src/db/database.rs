use std::collections::HashMap;
use std::path::Path;

use anyhow::{Result, anyhow};

use crate::db::collections::Collection;
use crate::document::document::Document;
use crate::pager::page::Page;
use crate::pager::pager::Pager;

pub struct Database {
    pager: Pager,
    storage: HashMap<String, Document>,
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
        let page = pager.read_page(meta.root_page)?;
        let storage = Self::deserialize(&page.data);

        let collection_pages = Collection::load_collection_pages(&mut pager, meta.root_page)?;
        let views = Collection::fetch_all_collections(&collection_pages)?;

        for view in views {
            collections.insert(view.name.to_string(), view.root_page);
        }

        Ok(Self {
            pager,
            storage,
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

    /// Redirects an existing collection lookup key to point to a new root address block
    pub fn update_collection(&mut self, name: &str, new_collection_root_page: u64) -> Result<()> {
        if !self.collections.contains_key(name) {
            return Err(anyhow!("Collection '{}' does not exist", name));
        }

        // Sync directly down onto targeted disk offsets
        Collection::update_collection(
            &mut self.pager,
            self.collection_root_page_id,
            name,
            new_collection_root_page,
        )?;

        // Sync local memory data trace
        self.collections
            .insert(name.to_string(), new_collection_root_page);
        Ok(())
    }

    /// Completely removes a collection registration layout out of active systems
    pub fn delete_collection(&mut self, name: &str) -> Result<()> {
        if !self.collections.contains_key(name) {
            return Err(anyhow!("Collection '{}' does not exist", name));
        }

        // Clean out page segments and shift layout values safely
        Collection::delete_collection(&mut self.pager, self.collection_root_page_id, name)?;

        // Drop from runtime tracking
        self.collections.remove(name);
        Ok(())
    }

    /// Quick read API to check where a collection's storage tree starts
    // pub fn get_collection_root(&self, name: &str) -> Option<u64> {
    //     self.collections.get(name).copied()
    // }

    pub fn set(&mut self, key: impl Into<String>, doc: Document) -> Result<()> {
        self.storage.insert(key.into(), doc);

        self.persist()
    }

    pub fn get(&self, key: &str) -> Option<&Document> {
        self.storage.get(key)
    }

    pub fn delete(&mut self, key: &str) -> Result<()> {
        self.storage.remove(key);

        self.persist()
    }

    fn persist(&mut self) -> Result<()> {
        let mut page = Page::new(5);

        let bytes = Self::serialize(&self.storage);

        let len = bytes.len().min(page.data.len());

        page.data[..len].copy_from_slice(&bytes[..len]);

        self.pager.write_page(&page)?;

        Ok(())
    }

    fn serialize(storage: &HashMap<String, Document>) -> Vec<u8> {
        let mut bytes = Vec::new();

        for (key, value) in storage {
            let key_bytes = key.as_bytes();

            let Ok(value_bytes) = value.to_vec() else {
                continue;
            };

            bytes.extend_from_slice(&(key_bytes.len() as u32).to_le_bytes());
            bytes.extend_from_slice(&(value_bytes.len() as u32).to_le_bytes());

            bytes.extend_from_slice(key_bytes);
            bytes.extend_from_slice(&value_bytes);
        }

        bytes
    }

    fn deserialize(data: &[u8]) -> HashMap<String, Document> {
        let mut map = HashMap::new();
        let mut cursor = 0;
        while cursor + 8 <= data.len() {
            let key_len = u32::from_le_bytes([
                data[cursor],
                data[cursor + 1],
                data[cursor + 2],
                data[cursor + 3],
            ]) as usize;
            cursor += 4;
            let doc_len = u32::from_le_bytes([
                data[cursor],
                data[cursor + 1],
                data[cursor + 2],
                data[cursor + 3],
            ]) as usize;
            cursor += 4;
            if key_len == 0 && doc_len == 0 {
                break;
            }

            if cursor + key_len + doc_len > data.len() {
                break;
            }

            let key = String::from_utf8_lossy(&data[cursor..cursor + key_len]).to_string();
            cursor += key_len;

            let doc_bytes = &data[cursor..cursor + doc_len];
            cursor += doc_len;

            let Ok(document) = Document::from_reader(doc_bytes) else {
                break;
            };

            map.insert(key, document);
        }

        map
    }
}
