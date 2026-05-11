use std::collections::HashMap;

#[derive(Debug)]
pub struct Cache {
    data: HashMap<String, Vec<u8>>,
}

impl Cache {
    pub fn new() -> Self {
        Cache {
            data: HashMap::new(),
        }
    }

    pub fn insert(&mut self, key: String, value: Vec<u8>) {
        self.data.insert(key, value);
    }

    pub fn get(&mut self, key: &String) -> Option<Vec<u8>> {
        self.data.get(key).cloned()
    }

    pub fn delete(&mut self, key: &String) {
        self.data.remove(key);
    }
}
