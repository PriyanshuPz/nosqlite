use anyhow::{Result, anyhow};
use std::path::Path;

use bson::{Bson, Document, oid::ObjectId};

use crate::{
    catalog::catalog::{Catalog, CatalogEntry},
    document::page::DocumentPage,
    pager::{page::Page, pager::Pager},
};

pub struct Database {
    pager: Pager,
}

impl Database {
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let pager = Pager::open(path)?;
        Ok(Self { pager })
    }

    // COLLECTION APIs
    pub fn create_collection(&mut self, name: &str) -> Result<()> {
        Catalog::create(&mut self.pager, name)
    }

    pub fn delete_collection(&mut self, name: &str) -> Result<()> {
        Catalog::delete(&mut self.pager, name)
    }

    pub fn list_collections(&mut self) -> Result<Vec<CatalogEntry>> {
        Catalog::list(&mut self.pager)
    }

    // DOCUMENT APIs
    pub fn insert_one(&mut self, collection: &str, mut document: Document) -> Result<()> {
        if !document.contains_key("_id") {
            document.insert("_id", Bson::ObjectId(ObjectId::new()));
        }

        let mut entry = Catalog::find(&mut self.pager, collection)?
            .ok_or_else(|| anyhow!("collection not found"))?;

        let bson_bytes = bson::serialize_to_vec(&document)?;

        if entry.first_document_page == 0 {
            let page_id = self.pager.allocate_page()?;

            let mut page = Page::new(page_id);

            DocumentPage::initialize(&mut page);
            DocumentPage::append_document(&mut page, &bson_bytes)?;

            self.pager.write_page(&page)?;

            entry.first_document_page = page_id;

            entry.document_count = 1;

            Catalog::update(&mut self.pager, &entry)?;

            return Ok(());
        }

        let mut current_page_id = entry.first_document_page;

        loop {
            let mut page = self.pager.read_page(current_page_id)?;

            // Need record header space too
            let required_space = bson_bytes.len() + 5;

            if DocumentPage::remaining_space(&page)? >= required_space {
                DocumentPage::append_document(&mut page, &bson_bytes)?;

                self.pager.write_page(&page)?;

                entry.document_count += 1;

                Catalog::update(&mut self.pager, &entry)?;

                return Ok(());
            }

            let next_page = DocumentPage::next_page(&page)?;

            // Allocate overflow page
            if next_page == 0 {
                let new_page_id = self.pager.allocate_page()?;

                let mut new_page = Page::new(new_page_id);

                DocumentPage::initialize(&mut new_page);
                DocumentPage::append_document(&mut new_page, &bson_bytes)?;
                DocumentPage::set_next_page(&mut page, new_page_id);

                self.pager.write_page(&page)?;

                self.pager.write_page(&new_page)?;

                entry.document_count += 1;

                Catalog::update(&mut self.pager, &entry)?;

                return Ok(());
            }

            current_page_id = next_page;
        }
    }

    pub fn find_all(&mut self, collection: &str) -> Result<Vec<Document>> {
        let entry = Catalog::find(&mut self.pager, collection)?
            .ok_or_else(|| anyhow!("collection not found"))?;

        let mut documents = Vec::new();

        let mut current_page_id = entry.first_document_page;

        while current_page_id != 0 {
            let page = self.pager.read_page(current_page_id)?;

            let stored_documents = DocumentPage::documents(&page)?;

            for stored in stored_documents {
                // Skip tombstones
                if stored.deleted {
                    continue;
                }
                let document: Document = bson::deserialize_from_slice(stored.bson)?;
                documents.push(document);
            }

            current_page_id = DocumentPage::next_page(&page)?;
        }

        Ok(documents)
    }

    pub fn find_by_id(&mut self, collection: &str, id: ObjectId) -> Result<Option<Document>> {
        let entry = Catalog::find(&mut self.pager, collection)?
            .ok_or_else(|| anyhow!("collection not found"))?;

        let mut current_page_id = entry.first_document_page;

        while current_page_id != 0 {
            let page = self.pager.read_page(current_page_id)?;

            let stored_documents = DocumentPage::documents(&page)?;

            for stored in stored_documents {
                if stored.deleted {
                    continue;
                }

                let document: Document = bson::deserialize_from_slice(stored.bson)?;

                match document.get("_id") {
                    Some(Bson::ObjectId(object_id)) if *object_id == id => {
                        return Ok(Some(document));
                    }
                    _ => {}
                }
            }

            current_page_id = DocumentPage::next_page(&page)?;
        }

        Ok(None)
    }

    pub fn delete_by_id(&mut self, collection: &str, id: ObjectId) -> Result<bool> {
        let mut entry = Catalog::find(&mut self.pager, collection)?
            .ok_or_else(|| anyhow!("collection not found"))?;

        let mut current_page_id = entry.first_document_page;

        while current_page_id != 0 {
            let mut page = self.pager.read_page(current_page_id)?;

            let delete_offset = {
                let stored_documents = DocumentPage::documents(&page)?;

                let mut found = None;

                for stored in stored_documents {
                    if stored.deleted {
                        continue;
                    }

                    let document: Document = bson::deserialize_from_slice(stored.bson)?;

                    match document.get("_id") {
                        Some(Bson::ObjectId(object_id)) if *object_id == id => {
                            found = Some(stored.offset);

                            break;
                        }

                        _ => {}
                    }
                }

                found
            };

            if let Some(offset) = delete_offset {
                DocumentPage::mark_deleted(&mut page, offset);
                self.pager.write_page(&page)?;
                if entry.document_count > 0 {
                    entry.document_count -= 1;
                }

                Catalog::update(&mut self.pager, &entry)?;

                return Ok(true);
            }

            current_page_id = DocumentPage::next_page(&page)?;
        }

        Ok(false)
    }
}
