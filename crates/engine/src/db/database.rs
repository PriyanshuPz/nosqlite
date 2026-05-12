use std::collections::HashMap;
use std::path::Path;

use anyhow::Result;

use crate::pager::page::Page;
use crate::pager::pager::Pager;

pub struct Database {
    pager: Pager,
    storage: HashMap<String, String>,
}

impl Database {
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let mut pager = Pager::open(path)?;
        let meta = pager.check_metadata()?;
        let page = pager.read_page(meta.root_page)?;

        let storage = Self::deserialize(&page.data);
        Ok(Self { pager, storage })
    }

    pub fn set(&mut self, key: impl Into<String>, value: impl Into<String>) -> Result<()> {
        self.storage.insert(key.into(), value.into());

        self.persist()
    }

    pub fn get(&self, key: &str) -> Option<&String> {
        self.storage.get(key)
    }

    pub fn delete(&mut self, key: &str) -> Result<()> {
        self.storage.remove(key);

        self.persist()
    }

    fn persist(&mut self) -> Result<()> {
        let mut page = Page::new(1);

        let bytes = Self::serialize(&self.storage);

        let len = bytes.len().min(page.data.len());

        page.data[..len].copy_from_slice(&bytes[..len]);

        self.pager.write_page(&page)?;

        Ok(())
    }

    fn serialize(storage: &HashMap<String, String>) -> Vec<u8> {
        let mut bytes = Vec::new();

        for (key, value) in storage {
            let key_bytes = key.as_bytes();
            let value_bytes = value.as_bytes();

            bytes.push(key_bytes.len() as u8);
            bytes.push(value_bytes.len() as u8);

            bytes.extend_from_slice(key_bytes);
            bytes.extend_from_slice(value_bytes);
        }

        bytes
    }

    fn deserialize(data: &[u8]) -> HashMap<String, String> {
        let mut map = HashMap::new();

        let mut cursor = 0;

        while cursor + 2 <= data.len() {
            let key_len = data[cursor] as usize;
            cursor += 1;

            let val_len = data[cursor] as usize;
            cursor += 1;

            if key_len == 0 && val_len == 0 {
                break;
            }

            if cursor + key_len + val_len > data.len() {
                break;
            }

            let key = String::from_utf8_lossy(&data[cursor..cursor + key_len]).to_string();

            cursor += key_len;

            let value = String::from_utf8_lossy(&data[cursor..cursor + val_len]).to_string();

            cursor += val_len;

            map.insert(key, value);
        }

        map
    }
}
