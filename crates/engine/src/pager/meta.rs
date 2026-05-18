use anyhow::{Ok, Result, bail};
use std::{
    fs::File,
    io::{Read, Seek, SeekFrom, Write},
};

use crate::pager::page::{PAGE_SIZE, Page};

pub const FILE_HEADER_SIZE: u64 = 4096;
pub const MAGIC_BYTES: &[u8; 16] = b"NOSQLITE_DB___P8";

pub const MAGIC_STR: &'static [u8; 15] = b"NOSQLITE FORMAT";
pub const ROOT_PAGE: u8 = 1;

#[derive(Debug, Clone)]
pub struct FileHeader {
    pub version: u32,
    pub total_pages: u64,
    pub free_list_head: u64,
    pub collections_root_page: u64,
}

impl FileHeader {
    pub fn new() -> Self {
        Self {
            version: 1,
            total_pages: 0,
            free_list_head: 0,
            collections_root_page: 0,
        }
    }
}

pub fn write_header(file: &mut File, header: &FileHeader) -> Result<()> {
    let mut buffer = [0u8; FILE_HEADER_SIZE as usize];

    let mut cursor = 0;

    buffer[cursor..cursor + 16].copy_from_slice(MAGIC_BYTES);
    cursor += 16;

    buffer[cursor..cursor + 4].copy_from_slice(&header.version.to_le_bytes());
    cursor += 4;

    buffer[cursor..cursor + 8].copy_from_slice(&header.total_pages.to_le_bytes());
    cursor += 8;

    buffer[cursor..cursor + 8].copy_from_slice(&header.free_list_head.to_le_bytes());
    cursor += 8;

    buffer[cursor..cursor + 8].copy_from_slice(&header.collections_root_page.to_le_bytes());

    file.seek(SeekFrom::Start(0))?;
    file.write_all(&buffer)?;

    Ok(())
}

pub fn read_header(file: &mut File) -> Result<FileHeader> {
    let mut buffer = [0u8; FILE_HEADER_SIZE as usize];
    file.seek(SeekFrom::Start(0))?;
    file.read_exact(&mut buffer)?;

    if &buffer[0..16] != MAGIC_BYTES {
        bail!("Invalid database file format");
    }

    let mut cursor = 16;

    let version = u32::from_le_bytes(buffer[cursor..cursor + 4].try_into()?);
    cursor += 4;

    let total_pages = u64::from_le_bytes(buffer[cursor..cursor + 8].try_into()?);
    cursor += 8;

    let free_list_head = u64::from_le_bytes(buffer[cursor..cursor + 8].try_into()?);
    cursor += 8;

    let collections_root_page = u64::from_le_bytes(buffer[cursor..cursor + 8].try_into()?);

    Ok(FileHeader {
        version,
        total_pages,
        free_list_head,
        collections_root_page,
    })
}

// I NEED TO CLEAN THIS OLD PROTOTYPE!!
pub struct Metadata {
    pub version: u8,
    pub root_page: u64,
}

pub fn write_metadata(file: &mut File) -> Result<()> {
    let mut page = Page::new(0);
    let mut cursor = 0;

    page.data[cursor..cursor + MAGIC_STR.len()].copy_from_slice(MAGIC_STR);
    cursor += MAGIC_STR.len();
    cursor += 1;
    page.data[cursor..cursor + 1].copy_from_slice(&[1]);
    cursor += 1;
    page.data[cursor..cursor + 1].copy_from_slice(&ROOT_PAGE.to_le_bytes());
    let offset = page.id * PAGE_SIZE as u64;
    file.seek(SeekFrom::Start(offset))?;
    file.write_all(&page.data)?;
    file.flush()?;

    Ok(())
}

pub fn read_metadata(file: &mut File) -> Result<Metadata> {
    let mut buffer = [0u8; PAGE_SIZE];

    file.seek(SeekFrom::Start(0))?;
    file.read_exact(&mut buffer)?;

    if &buffer[0..MAGIC_STR.len()] != MAGIC_STR {
        bail!("invalid database file");
    }

    let version = u8::from_le_bytes([buffer[16]]);
    let root_page = u8::from_le_bytes([buffer[17]]) as u64;
    Ok(Metadata { version, root_page })
}
