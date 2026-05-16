use anyhow::{Result, bail};
use std::{
    fs::File,
    io::{Read, Seek, SeekFrom, Write},
};

use crate::pager::page::{PAGE_SIZE, Page};

pub const MAGIC_STR: &'static [u8; 15] = b"NOSQLITE FORMAT";
pub const ROOT_PAGE: u32 = 1;

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
    cursor += 2;
    page.data[cursor..cursor + 4].copy_from_slice(&ROOT_PAGE.to_le_bytes());
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
    let root_page = u32::from_le_bytes([buffer[19], buffer[20], buffer[21], buffer[22]]) as u64;

    Ok(Metadata { version, root_page })
}
