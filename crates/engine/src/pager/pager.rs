use std::{
    fs::{File, OpenOptions},
    io::{Read, Seek, SeekFrom, Write},
    path::Path,
};

use anyhow::{Result, anyhow};

use crate::pager::{
    meta::{FILE_HEADER_SIZE, FileHeader, read_header, write_header},
    page::{PAGE_SIZE, Page, PageType},
};

pub struct Pager {
    file: File,
    pub header: FileHeader,
}

impl Pager {
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let mut file = OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .open(path)?;

        let header = if file.metadata()?.len() == 0 {
            Self::initialize_database(&mut file)?
        } else {
            read_header(&mut file)?
        };
        Ok(Self { file, header })
    }

    fn initialize_database(file: &mut File) -> Result<FileHeader> {
        let mut header = FileHeader::new();
        header.total_pages = 1;
        header.collections_root_page = 1;

        write_header(file, &header)?;

        let mut root_page = Page::new(1);
        root_page.set_page_type(PageType::CollectionCatalog);

        root_page.data[4..12].copy_from_slice(&0u64.to_le_bytes());

        let offset = Self::page_offset(1);
        file.seek(SeekFrom::Start(offset))?;
        file.write_all(&root_page.data)?;

        Ok(header)
    }

    fn page_offset(page_id: u64) -> u64 {
        FILE_HEADER_SIZE + ((page_id - 1) * PAGE_SIZE as u64)
    }

    pub fn flush_header(&mut self) -> Result<()> {
        write_header(&mut self.file, &self.header)
    }

    pub fn allocate_page(&mut self) -> Result<u64> {
        self.header.total_pages += 1;
        let page_id = self.header.total_pages;

        self.flush_header()?;

        let page = Page::new(page_id);
        self.write_page(&page)?;

        Ok(page_id)
    }

    pub fn read_page(&mut self, page_id: u64) -> Result<Page> {
        if page_id == 0 {
            return Err(anyhow!("page id 0 is invalid"));
        }
        if page_id > self.header.total_pages {
            return Err(anyhow!("page does not exist"));
        }
        let mut page = Page::new(page_id);
        let offset = Self::page_offset(page_id);
        self.file.seek(SeekFrom::Start(offset))?;
        self.file.read_exact(&mut page.data)?;
        Ok(page)
    }
    pub fn write_page(&mut self, page: &Page) -> Result<()> {
        if page.id == 0 {
            return Err(anyhow!("page id 0 is invalid"));
        }
        let offset = Self::page_offset(page.id);
        self.file.seek(SeekFrom::Start(offset))?;
        self.file.write_all(&page.data)?;
        Ok(())
    }
}
