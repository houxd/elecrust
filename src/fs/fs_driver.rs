use crate::{driver::Driver, error::Result, fs::fs::DirEntry};
use alloc::{boxed::Box, vec::Vec};

pub trait FsDriver: Driver {
    fn info(&self) -> Result<FsInfo>;
    fn mount(&self) -> Result<()>;
    fn unmount(&self) -> Result<()>;
    fn format(&self) -> Result<()>;
    fn mkdir(&self, path: &str) -> Result<()>;
    fn unlink(&self, path: &str) -> Result<()>;
    fn rename(&self, old_path: &str, new_path: &str) -> Result<()>;
    fn stat(&self, path: &str) -> Result<FileStat>;
    fn readdir(&self, path: &str) -> Result<Vec<DirEntry>>;
    fn exists(&self, path: &str) -> Result<bool>;
    fn sync(&self) -> Result<()>;
    fn open(&self, path: &str, mode: &str) -> Result<Box<dyn FileHandle>>;
}

pub trait FileHandle {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize>;
    fn write(&mut self, buf: &[u8]) -> Result<usize>;
    fn seek(&mut self, pos: isize, whence: Whence) -> Result<isize>;
    fn flush(&mut self) -> Result<()>;
    fn close(&mut self) -> Result<()>;
}

pub struct FsInfo {
    pub total_bytes: isize,
    pub used_bytes: isize,
}

impl FsInfo {
    pub fn new(total_bytes: isize, used_bytes: isize) -> Self {
        Self {
            total_bytes,
            used_bytes,
        }
    }
}

#[allow(unused)]
#[allow(non_camel_case_types)]
pub enum Whence {
    SEEK_SET = 0,
    SEEK_CUR = 1,
    SEEK_END = 2,
}

pub struct FileStat {
    pub file_type: FileType,
    pub size: usize,
    pub access_time: u32,
    pub modification_time: u32,
    pub creation_time: u32,
}

pub enum FileType {
    File,
    Directory,
    Symlink,
    NotExist,
}
