
use core::{fmt::Display, hash::SipHasher, marker::PhantomData};

use alloc::{boxed::Box, vec::Vec};
use fat16::{BootSector, Format83};

use crate::utils::DoubleVecIndex;

pub use ata::*;
pub mod ata;
pub mod fat16;
pub mod io;

pub const SECTOR_SIZE: usize = 512;


#[derive(Debug)]
pub struct File {
    pub name: Format83,
    pub start_sector: u32,
    pub size: u32,
    pub time_stamp: TimeStamp
}

#[derive(Debug)]
pub struct Directory<T: StorageEntry> {
    pub files: Vec<File>,
    pub directories: Vec<Box<Directory<T>>>,
    contents: DoubleVecIndex<Format83, T>,
    name: Format83,
    time_stamp: TimeStamp,
}

impl<T: StorageEntry> Directory<T> {
    pub fn take(&mut self, key: Format83) -> Option<T> {
        self.contents.take(key, &mut SipHasher::new())
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
    pub fn root(&'a self) -> anyhow::Result<Directory<T::Entry>> {
        self.storage_format.get_root()
    }
    pub fn load(&self, child: Format83, directory: &mut Directory<T::Entry>) -> anyhow::Result<()> {
        self.storage_format.load_child(child, directory)
    }
}

pub trait StorageFormat<'a>: Sized {
    type Entry: StorageEntry;
    fn new(ata_bus: &'a ATABus, drive: BusDrive) -> anyhow::Result<Self>;
    fn boot_sector(&self) -> BootSector;
    fn get_root(&self) -> anyhow::Result<Directory<Self::Entry>>;
    fn load_child(&self, child: Format83, directory: &mut Directory<Self::Entry>) -> anyhow::Result<()>;
    fn is_directory(entry: &Self::Entry) -> bool;
}

pub trait StorageEntry: Display + Into<Format83> + Clone {}

#[derive(Default, Debug)]
pub struct TimeStamp {
    second: u8,
    minute: u8,
    hour: u8,
    day: u8,
    month: u8,
    year: u8,
}

impl Display for TimeStamp {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}:{}:{} {}/{}/{}",
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
        let file_names: Vec<&Format83> = self.files.iter().map(|f| &f.name).collect();
        let dir_names: Vec<&Format83> = self.directories.iter().map(|d| &d.name).collect();
        writeln!(f, "Directory: {}", self.name)?;
        writeln!(f, "\tUnloaded: {:?}", self.contents.keys())?;
        writeln!(f, "\tFiles: {:?}", file_names)?;
        writeln!(f, "\tSubdirectories: {:?}", dir_names)?;
        writeln!(f, "\tTimestamp: {}", self.time_stamp)?;
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

pub struct Name {
    
}



