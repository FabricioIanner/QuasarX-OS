use alloc::{string::String, vec::Vec};

#[derive(Debug, Clone)]
pub enum VNode {
    File(File),
    Directory(Directory),
}

/// Representa um arquivo aberto (nome + posição + conteúdo em memória)
pub struct File {
    pub name: String,
    pub pos: usize,
    pub data: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct Directory {
    pub name: String,
    pub entries: Vec<DirEntry>,
}

impl File {
    pub fn read_byte(&mut self) -> Option<u8> {
        if self.pos >= self.data.len() {
            None
        } else {
            let byte = self.data[self.pos];
            self.pos += 1;
            Some(byte)
        }
    }

    pub fn rewind(&mut self) {
        self.pos = 0;
    }
}

pub trait Filesystem {
    fn open(&self, name: &str) -> Option<File>;
    fn list_dir(&self, path: &str) -> Option<Directory>;
}

#[derive(Debug, Clone)]
pub struct DirEntry {
    pub name: String,
    pub is_dir: bool,
    pub cluster: u16,
    pub size: u32,
}

/// Registro do VFS
pub struct VFS {
    fs: Option<&'static dyn Filesystem>,
}

impl VFS {
    pub const fn new() -> Self {
        Self { fs: None }
    }

    pub fn mount(&mut self, fs: &'static dyn Filesystem) {
        self.fs = Some(fs);
    }

    pub fn open(&self, name: &str) -> Option<File> {
        self.fs?.open(name)
    }

     pub fn list_dir(&self, path: &str) -> Option<Directory> {
        self.fs?.list_dir(path)
    }
}

use spin::Mutex;
use lazy_static::lazy_static;

lazy_static! {
    pub static ref VFS_INSTANCE: Mutex<VFS> = Mutex::new(VFS::new());
}
