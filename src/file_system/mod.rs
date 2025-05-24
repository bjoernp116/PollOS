
pub mod ata;
use core::marker::PhantomData;

use alloc::{string::String, vec::Vec};
pub use ata::*;
use fat16::{BootSector, DirEntry};

use crate::utils::DoubleVecIndex;

pub mod fat16;

pub mod io;
type Result<T> = core::result::Result<T, anyhow::Error>;
pub const SECTOR_SIZE: usize = 512;


#[derive(Debug)]
pub struct File {
    pub name: [u8; 11],
    pub start_sector: u32,
    pub size: u32,
}

#[derive(Debug)]
pub struct Directory<'a> {
    contents: DoubleVecIndex<String, DirEntry>,
    files: Vec<File>,
    directories: Vec<&'a Directory<'a>>,
    name: String,
}

impl<'a> Directory<'a> {
    pub fn take_child(&mut self, child: String) -> bool {
        if let Some(child) = self.contents.take(child) {
            if child.is_dir() {
                
            }
        }
        false
    }
}

impl<'a> Default for Directory<'a> {
    fn default() -> Self {
        Directory { 
            contents: DoubleVecIndex::new(Vec::new()), 
            files: Vec::new(), 
            directories: Vec::new(), 
            name: String::new() 
        }
    }
}

pub struct FileSystem<'a, T: StorageFormat<'a>> {
    storage_format: T,
    _phantom: PhantomData<&'a T>
}

impl<'a, T: StorageFormat<'a>> FileSystem<'a, T> {
    pub fn new(ata: &'a ATABus, drive: BusDrive) -> anyhow::Result<Self> {
        let storage_format = T::new(ata, drive)?;
        Ok(FileSystem {
            storage_format,
            _phantom: PhantomData
        })
    }
    pub fn root(&'a self) -> anyhow::Result<Directory<'a>> {
        self.storage_format.get_root()
    }
}

pub trait StorageFormat<'a>: Sized {
    fn new(ata_bus: &'a ATABus, drive: BusDrive) -> anyhow::Result<Self>;
    fn boot_sector(&self) -> BootSector;
    fn get_root(&self) -> anyhow::Result<Directory>;
}






