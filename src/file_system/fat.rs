use alloc::vec::Vec;
use x86_64::instructions::port::Port;

use crate::println;

const ATA_PRIMARY_IO: u16 = 0x1F0;
const ATA_PRIMARY_CTRL: u16 = 0x3F6;

const SECTOR_SIZE: usize = 512;

unsafe fn ata_pio_read_sector(lba: u32, buffer: &mut [u8; SECTOR_SIZE]) {

    //let mut io = |port: u16| Port::<u8>::new(port);
    //let mut io16 = |port: u16| Port::<u16>::new(port);

    let lba_bytes = lba.to_le_bytes();


    while Port::<u8>::new(ATA_PRIMARY_IO + 7).read() & 0x80 != 0 {}

    Port::new(ATA_PRIMARY_IO + 2).write(1u16);
    Port::new(ATA_PRIMARY_IO + 3).write(lba_bytes[0]);
    Port::new(ATA_PRIMARY_IO + 4).write(lba_bytes[1]);
    Port::new(ATA_PRIMARY_IO + 5).write(lba_bytes[2]);
    Port::new(ATA_PRIMARY_IO + 6).write(0xE0 | ((lba >> 24) & 0x0F) as u8);


    Port::new(ATA_PRIMARY_IO + 7).write(0x20u16);


    let mut timeout = 100_000;
    while timeout > 0 {
        let status = Port::<u8>::new(ATA_PRIMARY_IO + 7).read();
        println!("ATA status: {:08b}", status);
        if status & 0x80 == 0 && status & 0x08 != 0 {
            break;
        }
        timeout -= 1;
    }

    if timeout == 0 {
        panic!("Timeout waiting for ATA DRQ");
    }

    let mut data_port = Port::<u16>::new(ATA_PRIMARY_IO);

    for i in 0..(SECTOR_SIZE / 2) {
        let word = data_port.read();
        buffer[i * 2] = word as u8;
        buffer[i * 2 + 1] = (word >> 8) as u8;
    }


}

pub unsafe fn read_boot_sector() -> FatBootSector {
    let mut buffer = [0u8; SECTOR_SIZE];
    unsafe {
        ata_pio_read_sector(0, &mut buffer);
    }

    for i in 0..16 {
        println!("{:02X} ", buffer[i]);
    }

    unsafe {
        core::ptr::read(buffer.as_ptr() as *const FatBootSector)
    }
}

#[repr(C, packed)]
#[derive(Debug)]
pub struct FatBootSector {
    jump_boot: [u8; 3],
    oem_name: [u8; 8],
    bytes_per_sector: u16,
    sector_per_cluster: u8,
    reserved_sectors: u16,
    num_fats: u8,
    max_root_dir_entries: u16,
    total_sectors_short: u16,
    media_descriptor: u8,
    sectors_per_fat: u16,
    sectors_per_track: u16,
    num_heads: u16,
    hidden_sectors: u32,
    total_sectors_long: u32,
}

#[repr(C, packed)]
#[derive(Debug)]
pub struct DirEntry {
    name: [u8; 8],
    ext: [u8; 3],
    attr: u8,
    reserved: u8,
    creation_time_tenths: u8,
    creation_time: u16,
    creation_date: u16,
    last_access_date: u16,
    first_cluster_high: u16,
    write_time: u16,
    write_date: u16,
    first_cluster_low: u16,
    file_size: u32,
}

pub fn read_root_directory(boot: &FatBootSector) -> Vec<DirEntry> {
    let root_dir_sectors = ((boot.max_root_dir_entries as usize * 32) + 511) / 512;
    let root_start_sector = 
        boot.reserved_sectors as u32 + 
        boot.num_fats as u32 * 
        boot.sectors_per_fat as u32;

    let mut entries = Vec::new();
    for i in 0..root_dir_sectors {
        let mut buf = [0u8; SECTOR_SIZE];
        unsafe {
            ata_pio_read_sector(root_start_sector + i as u32, &mut buf);
        }

        let entry_count = SECTOR_SIZE / core::mem::size_of::<DirEntry>();
        let entry_ptr = buf.as_ptr() as *const DirEntry;
        for j in 0..entry_count {
            let entry = unsafe { core::ptr::read_unaligned(entry_ptr.add(j)) };
            if entry.name[0] == 0x00 {
                break;
            }
            entries.push(entry);
        }
    }

    entries
}


