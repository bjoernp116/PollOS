use anyhow::anyhow;
use core::{fmt::Display, marker::PhantomData};

use alloc::{boxed::Box, string::String, vec::Vec};
use fat16::BootSector;

use crate::{
    print, println, serial_print, serial_println, utils::DoubleVecIndex,
};

pub use ata::*;
pub mod ata;
pub mod fat16;
pub mod io;

pub const SECTOR_SIZE: usize = 512;

#[derive(Debug)]
pub struct File {
    pub name: String,
    pub ext: String,
    pub start_sector: usize,
    pub start_cluster: u16,
    pub size: u32,
    pub time_stamp: TimeStamp,
}

impl File {
    pub fn name(&self) -> String {
        alloc::format!("{}.{}", self.name, self.ext)
    }
}

#[derive(Debug)]
pub struct Directory<T: StorageEntry> {
    pub files: Vec<File>,
    pub directories: Vec<Box<Directory<T>>>,
    contents: DoubleVecIndex<String, T>,
    name: String,
}

impl<T: StorageEntry> Directory<T> {
    pub fn take(&mut self, key: String) -> Option<T> {
        self.contents.take(key)
    }
}

pub struct FileSystem<'a, T: StorageFormat<'a>> {
    pub storage_format: T,
    _phantom: PhantomData<&'a T>,
}

impl<'a, T: StorageFormat<'a>> FileSystem<'a, T> {
    pub fn new(ata: &'a ATABus, drive: BusDrive) -> anyhow::Result<Self> {
        let storage_format = T::new(ata, drive)?;
        Ok(FileSystem {
            storage_format,
            _phantom: PhantomData,
        })
    }
    pub fn root(&'a self) -> anyhow::Result<Directory<T::Entry>> {
        self.storage_format.get_root()
    }
    pub fn load_directory(
        &self,
        child: String,
        directory: &mut Directory<T::Entry>,
    ) -> anyhow::Result<()> {
        match self.storage_format.load_child(child.clone(), directory) {
            LoadChildResult::Directory(_) => Ok(()),
            LoadChildResult::File(_) => {
                Err(anyhow!("Expected Directory: {}, found file!", child))
            }
            LoadChildResult::NotFound => {
                Err(anyhow!("Directory {} not found!", child))
            }
        }
    }
    pub fn get_content(&self, file: &File) -> Vec<u8> {
        self.storage_format.get_content(file).unwrap()
    }
    pub fn load_file(
        &self,
        child: String,
        directory: &mut Directory<T::Entry>,
    ) -> anyhow::Result<()> {
        match self.storage_format.load_child(child.clone(), directory) {
            LoadChildResult::File(_) => Ok(()),
            LoadChildResult::Directory(_) => {
                Err(anyhow!("Expected File: {}, found directory!", child))
            }
            LoadChildResult::NotFound => {
                Err(anyhow!("Directory {} not found!", child))
            }
        }
    }
}

pub trait StorageFormat<'a>: Sized {
    type Entry: StorageEntry;
    fn new(ata_bus: &'a ATABus, drive: BusDrive) -> anyhow::Result<Self>;
    fn boot_sector(&self) -> BootSector;
    fn get_root(&self) -> anyhow::Result<Directory<Self::Entry>>;
    fn load_child(
        &self,
        child: String,
        directory: &mut Directory<Self::Entry>,
    ) -> LoadChildResult;
    fn is_directory(entry: &Self::Entry) -> bool;
    fn get_content(&self, file: &File) -> anyhow::Result<Vec<u8>>;
    fn read_bytes(
        &self,
        file: &File,
        buffer: &mut [u8],
        offset: usize,
    ) -> anyhow::Result<()>;
}

pub enum LoadChildResult {
    Directory(usize),
    File(usize),
    NotFound,
}

pub trait StorageEntry: Display + Into<String> + Clone {}

#[derive(Default, Debug)]
pub struct TimeStamp {
    second: u8,
    minute: u8,
    hour: u8,
    day: u8,
    month: u8,
    year: u16,
}

impl Display for TimeStamp {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "{}:{}:{} {}/{}/{}",
            self.hour,
            self.minute,
            self.second,
            self.day,
            self.month,
            self.year
        )
    }
}

impl<T: StorageEntry> Display for Directory<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let file_names: Vec<String> =
            self.files.iter().map(|f| f.name()).collect();
        let dir_names: Vec<String> =
            self.directories.iter().map(|d| d.name.clone()).collect();
        writeln!(f, "Directory: {}", self.name)?;
        writeln!(f, "\tUnloaded: {:?}", self.contents.keys())?;
        writeln!(f, "\tFiles: {:?}", file_names)?;
        writeln!(f, "\tSubdirectories: {:?}", dir_names)?;
        Ok(())
    }
}

impl Display for File {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        writeln!(f, "File: {}", self.name)?;
        writeln!(f, "\tTimestamp: {}", self.time_stamp)?;
        writeln!(f, "\tSize: {}", self.size)?;
        Ok(())
    }
}

#[allow(unused)]
pub fn print_buffer(buffer: &[u8; SECTOR_SIZE]) {
    for (i, byte) in buffer.iter().enumerate() {
        if i % 16 == 0 {
            print!("\n{:04x}: ", i);
        }
        print!("{:02x} ", byte);
    }
    println!();
    for (i, byte) in buffer.iter().enumerate() {
        if i % 16 == 0 {
            serial_print!("\n{:04x}: ", i);
        }
        serial_print!("{:02x} ", byte);
    }
    serial_println!();
}
