use std::{
    fs::{File, OpenOptions},
    io::{Read, Seek, SeekFrom, Write},
    path::Path,
};

use anyhow::Result;

use crate::pager::{
    meta::{Metadata, read_metadata, write_metadata},
    page::{PAGE_SIZE, Page},
};

pub struct Pager {
    file: File,
}

impl Pager {
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let mut file = OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .open(path)?;
        let _ = write_metadata(&mut file);
        Ok(Self { file })
    }

    pub fn read_page(&mut self, page_id: u64) -> Result<Page> {
        let mut page = Page::new(page_id);

        let offset = page_id * PAGE_SIZE as u64;

        self.file.seek(SeekFrom::Start(offset))?;
        self.file.read(&mut page.data)?;

        Ok(page)
    }

    pub fn check_metadata(&mut self) -> Result<Metadata> {
        read_metadata(&mut self.file)
    }

    pub fn write_page(&mut self, page: &Page) -> Result<()> {
        let offset = page.id * PAGE_SIZE as u64;

        self.file.seek(SeekFrom::Start(offset))?;
        self.file.write_all(&page.data)?;
        self.file.flush()?;

        Ok(())
    }
}
