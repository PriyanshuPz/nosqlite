use std::collections::HashMap;
use std::path::Path;
use std::str::from_utf8_unchecked;

use anyhow::Result;
use bson::Document;

use crate::pager::page::Page;
use crate::pager::pager::Pager;

pub struct Database {
    pager: Pager,
    storage: HashMap<String, Document>,
    collections: HashMap<String, u64>,
    collection_root_page_id: u64,
}

impl Database {
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let mut pager = Pager::open(path)?;
        let meta = pager.check_metadata()?;
        let mut collections = HashMap::new();
        let page = pager.read_page(meta.root_page)?;
        let storage = Self::deserialize(&page.data);

        if meta.root_page == 1 {
            let mut root_page = Page::new(1);
            root_page.data[0..4].copy_from_slice(&0u32.to_be_bytes());
            root_page.data[4..12].copy_from_slice(&0u64.to_be_bytes());
            pager.write_page(&root_page)?;

            return Ok(Self {
                pager,
                storage: storage,
                collections,
                collection_root_page_id: 1,
            });
        }

        Ok(Self { pager, storage })
    }

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

    fn find_collections(&mut self, root: u64) -> Result<()> {
        let mut cursor = 0;

        // let mut map = HashMap::new();
        let page = self.pager.read_page(root)?;
        let num_collections = page.data[0];
        cursor += 1;
        // TODO! implement this extension
        // let next_page = page.data[1];
        if num_collections == 0 {
            return Ok(());
        }

        let mut names = Vec::new();
        let mut name = Vec::new();
        while cursor <= page.data.len() {
            let bit = page.data[cursor];
            if bit == 0 {
                names.push(String::from_utf8(name.clone())?);
                name.clear();
            }

            name.extend_from_slice(&(page.data[cursor]).to_le_bytes());
            cursor += 1;
        }

        Ok(())
    }

    fn persist(&mut self) -> Result<()> {
        let mut page = Page::new(1);

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
