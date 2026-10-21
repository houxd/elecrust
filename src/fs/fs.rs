use crate::err;
use crate::error::{Error, Result};
use crate::fs::{
    fs_driver::{FileHandle, FsDriver, Whence},
    FileStat, FileType,
};
use alloc::{boxed::Box, format, rc::Rc, string::String, vec::Vec};
use core::cell::Cell;

pub struct Fs {
    fs_table: Rc<Vec<FsEntry>>,
}

pub struct FsEntry {
    mount_point: Rc<String>,
    fs_driver: Rc<dyn FsDriver>,
    mounted: Cell<bool>,
}

impl FsEntry {
    pub fn new(mount_point: impl Into<String>, fs_driver: Rc<dyn FsDriver>) -> Self {
        Self {
            mount_point: Rc::new(mount_point.into()),
            fs_driver: fs_driver.clone(),
            mounted: Cell::new(false),
        }
    }
    pub fn mount_point(&self) -> &str {
        &self.mount_point.as_ref()
    }
    pub fn fs_driver(&self) -> &Rc<dyn FsDriver> {
        &self.fs_driver
    }
    pub fn mounted(&self) -> bool {
        self.mounted.get()
    }
}

impl Fs {
    pub fn new(fs_table: Vec<FsEntry>) -> Self {
        Self {
            fs_table: Rc::new(fs_table),
        }
    }

    pub fn fstab(&self) -> &Vec<FsEntry> {
        &self.fs_table
    }

    fn path_to_driver_path<'a>(&self, path: &'a str) -> Result<(Rc<dyn FsDriver>, &'a str)> {
        if path.is_empty() {
            return Err(Error::new("path is empty"));
        }

        if path == "/" {
            return Err(Error::new("path is root"));
        }

        for entry in self.fs_table.iter() {
            if path.starts_with(&entry.mount_point()) {
                let mut sub_path = &path[entry.mount_point().len()..];

                // if sub_path is empty, it means the path is the mount point
                if sub_path.is_empty() {
                    sub_path = "/";
                }

                return Ok((entry.fs_driver().clone(), sub_path));
            }
        }
        Err(Error::new("path not found"))
    }

    /// 归一化路径。Vfs 只接受绝对路径（相对路径由上层 shell 结合 cwd 解析后传入），
    /// 这里负责消除 `.`、`..` 以及多余的 `/`，并保证结果以 `/` 开头。
    pub fn normalize_path(&self, path: impl Into<String>) -> String {
        let path = path.into();
        let mut normalized_parts: Vec<&str> = Vec::new();
        for part in path.split('/') {
            if part == "." || part.is_empty() {
                continue;
            } else if part == ".." {
                normalized_parts.pop();
            } else {
                normalized_parts.push(part);
            }
        }
        format!("/{}", normalized_parts.join("/"))
    }

    pub fn open(&self, path: impl Into<String>, mode: &str) -> Result<File> {
        let path = self.normalize_path(path);
        let (fs, path) = self.path_to_driver_path(&path)?;
        let node = fs.open(path, mode)?;
        Ok(File {
            node,
            _fs: fs.clone(),
        })
    }

    pub fn mount(&self, mount_point: impl Into<String>) -> Result<()> {
        let mount_point = self.normalize_path(mount_point);
        for entry in self.fs_table.iter() {
            if entry.mount_point() == mount_point {
                if entry.mounted.get() {
                    return Err(err!("{mount_point} already mounted"));
                }
                entry.fs_driver().mount()?;
                entry.mounted.set(true);
                return Ok(());
            }
        }
        Err(err!("{mount_point} not found"))
    }

    pub fn unmount(&self, mount_point: impl Into<String>) -> Result<()> {
        let mount_point = self.normalize_path(mount_point);
        for entry in self.fs_table.iter() {
            if entry.mount_point() == mount_point {
                if !entry.mounted.get() {
                    return Err(err!("{mount_point} not mounted"));
                }
                entry.fs_driver().unmount()?;
                entry.mounted.set(false);
                return Ok(());
            }
        }
        Err(err!("{mount_point} not found"))
    }

    /// 格式化。文件系统驱动通常无法在挂载状态下安全地重建自身，
    /// 因此这里先卸载再格式化，格式化前是挂载状态的话最后再挂回去。
    pub fn format(&self, mount_point: impl Into<String>) -> Result<()> {
        let mount_point = self.normalize_path(mount_point);
        for entry in self.fs_table.iter() {
            if entry.mount_point() == mount_point {
                let was_mounted = entry.mounted.get();
                if was_mounted {
                    entry.fs_driver().unmount()?;
                    entry.mounted.set(false);
                }
                entry.fs_driver().format()?;
                if was_mounted {
                    entry.fs_driver().mount()?;
                    entry.mounted.set(true);
                }
                return Ok(());
            }
        }
        Err(err!("{mount_point} not found"))
    }
    pub fn mkdir(&self, path: impl Into<String>) -> Result<()> {
        let path = self.normalize_path(path);
        if path == "/" {
            return Err(Error::new("cannot create root directory"));
        }
        let (fs, path) = self.path_to_driver_path(&path)?;
        fs.mkdir(path)
    }
    pub fn unlink(&self, path: impl Into<String>) -> Result<()> {
        let path = self.normalize_path(path);
        if path == "/" {
            return Err(Error::new("cannot delete root directory"));
        }
        if path == ".." {
            return Err(Error::new("cannot delete parent directory(..)"));
        }
        let (fs, path) = self.path_to_driver_path(&path)?;
        fs.unlink(path)
    }
    pub fn rename(&self, old_path: impl Into<String>, new_path: impl Into<String>) -> Result<()> {
        let old_path = self.normalize_path(old_path);
        let new_path = self.normalize_path(new_path);
        if old_path == "/" || new_path == "/" {
            return Err(Error::new("cannot rename root directory"));
        }
        let (old_fs, old_path) = self.path_to_driver_path(&old_path)?;
        let (new_fs, new_path) = self.path_to_driver_path(&new_path)?;
        if core::ptr::addr_eq(
            old_fs.as_ref() as *const dyn FsDriver,
            new_fs.as_ref() as *const dyn FsDriver,
        ) {
            return old_fs.rename(old_path, new_path);
        }

        let mut src_file = self.open(old_path, "r")?;
        let mut dst_file = self.open(new_path, "w")?;
        let mut buf = [0u8; 128];
        while let Ok(n) = src_file.read(&mut buf) {
            if n == 0 {
                break;
            }
            dst_file.write(&buf[..n])?;
        }
        src_file.close()?;
        dst_file.close()?;
        Ok(())
    }
    pub fn stat(&self, path: impl Into<String>) -> Result<FileStat> {
        let path = self.normalize_path(path);
        if path == "/" {
            return Ok(FileStat {
                file_type: FileType::Directory,
                size: 0,
                access_time: 0,
                modification_time: 0,
                creation_time: 0,
            });
        }
        let (fs, path) = self.path_to_driver_path(&path)?;
        fs.stat(path)
    }
    pub fn exists(&self, path: impl Into<String>) -> Result<bool> {
        let stat = self.stat(path)?;
        match stat.file_type {
            FileType::NotExist => {
                return Ok(false);
            }
            _ => {
                return Ok(true);
            }
        }
    }
    pub fn readdir(&self, path: impl Into<String>) -> Result<Vec<DirEntry>> {
        let path = self.normalize_path(path);
        if path == "/" {
            let mut entries: Vec<DirEntry> = Vec::new();
            for entry in self.fs_table.iter() {
                entries.push(DirEntry::new(
                    entry.mount_point().trim_start_matches('/'),
                    FileType::Directory,
                    0,
                ));
            }
            return Ok(entries);
        }
        let (fs, path) = self.path_to_driver_path(&path)?;
        fs.readdir(path)
    }
    pub fn sync(&self) -> Result<()> {
        for entry in self.fs_table.iter() {
            entry.fs_driver().sync()?;
        }
        Ok(())
    }
}

pub struct File {
    node: Box<dyn FileHandle>,
    // 持有文件系统的 Rc，保证在文件打开期间底层文件系统不被释放，
    // 避免句柄内部指向文件系统状态的指针悬垂。
    _fs: Rc<dyn FsDriver>,
}

impl File {
    pub fn close(&mut self) -> Result<()> {
        self.node.close()
    }
    pub fn flush(&mut self) -> Result<()> {
        self.node.flush()
    }
    pub fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        self.node.read(buf)
    }
    pub fn write(&mut self, buf: &[u8]) -> Result<usize> {
        self.node.write(buf)
    }
    pub fn seek(&mut self, pos: isize, whence: Whence) -> Result<isize> {
        self.node.seek(pos, whence)
    }
}

pub struct DirEntry {
    pub name: String,
    pub file_type: FileType,
    pub size: usize,
}

impl DirEntry {
    pub fn new(name: impl Into<String>, file_type: FileType, size: usize) -> Self {
        Self {
            name: name.into(),
            file_type,
            size,
        }
    }
    pub fn is_file(&self) -> bool {
        matches!(self.file_type, FileType::File)
    }
    pub fn is_dir(&self) -> bool {
        matches!(self.file_type, FileType::Directory)
    }
}
