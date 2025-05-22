
pub mod ata;
pub use ata::*;
use fat16::BootSector;

pub mod fat16;

pub mod io;
type Result<T> = core::result::Result<T, anyhow::Error>;
pub const SECTOR_SIZE: usize = 512;


pub struct File {
    pub name: [u8; 11],
    pub start_sector: u32,
    pub size: u32,
}

pub struct FileSystem<'a> {
    boot_sector: BootSector,
    ata: &'a ATABus,
    drive: BusDrive
}

impl<'a> FileSystem<'a> {
    pub fn new(ata: &'a ATABus, drive: BusDrive, storage_format: impl StorageFormat) -> Self {
        let boot_sector = storage_format.boot_sector();
        Self { ata, drive, boot_sector }
    }

    pub fn read_sector(&self, lba: usize, buffer: &mut [u8; SECTOR_SIZE]) -> Result<()> {
        self.ata.read(buffer, self.drive, lba, 1)?;
        Ok(())
    }
}

pub trait StorageFormat {
    fn boot_sector(&self) -> BootSector;
}








