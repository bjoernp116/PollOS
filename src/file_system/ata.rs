use alloc::vec::Vec;
use x86_64::{instructions::port::{Port, PortGeneric, PortReadOnly, PortWriteOnly, ReadOnlyAccess, WriteOnlyAccess}, structures::port::PortRead};

use bitflags::bitflags;

use crate::println;
use crate::serial_println;

const PORT_MASK: u16 = 0xFFFC;

bitflags! {
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub struct ATAStatus: u8 {
        const BUSY                 = 0x80;
        const DRIVE_READY          = 0x40;
        const DRIVE_WRITE_FAULT    = 0x20;
        const DRIVE_SEEK_COMPLETE  = 0x10;
        const DATA_REQUEST_READY   = 0x08;
        const CORRECTED_DATA       = 0x04;
        const INDEX                = 0x02;
        const ERROR                = 0x01;
    }
}

#[derive(Copy, Clone, Debug)]
#[repr(u8)]
pub enum BusDrive {
    Master = 0 << 4,
    Slave = 1 << 4
}

#[repr(u8)]
enum ATACommand {
    Read = 0x20,
    Write = 0x30,
    CacheFlush = 0xE7,
    Identify = 0xEC
}

pub struct ATABus {
    data: Port<u16>,            // BAR0 + 0
    error: PortReadOnly<u8>,    // BAR0 + 1
    sector_count: Port<u8>,     // BAR0 + 2
    lba_low: Port<u8>,          // BAR0 + 3
    lba_mid: Port<u8>,          // BAR0 + 4
    lba_high: Port<u8>,         // BAR0 + 5
    drive_select: Port<u8>,     // BAR0 + 6
    command: PortWriteOnly<u8>, // BAR0 + 7
    status: PortReadOnly<u8>,   // BAR0 + 7
    alt_status: PortReadOnly<u8>,// BAR1 + 2
    control: PortWriteOnly<u8>  // BAR1 + 2
}

impl ATABus {
    pub fn new(data_bar: u16, ctrl_bar: u16) -> Self {
        let data_bar = data_bar & PORT_MASK;
        let ctrl_bar = ctrl_bar & PORT_MASK;

        ATABus {
            data: Port::new(data_bar),
            error: PortReadOnly::new(data_bar + 1),
            sector_count: Port::new(data_bar + 2),
            lba_low: Port::new(data_bar + 3),
            lba_mid: Port::new(data_bar + 4),
            lba_high: Port::new(data_bar + 5),
            drive_select: Port::new(data_bar + 6),
            command: PortWriteOnly::new(data_bar + 7),
            status: PortReadOnly::new(data_bar + 7),
            control: PortWriteOnly::new(ctrl_bar + 2),
            alt_status: PortReadOnly::new(ctrl_bar + 2),
        }
    }

    unsafe fn read_pio(&mut self, buffer: &mut [u8], lba_start: usize, sector_count: usize) -> usize {
        if sector_count == 0 { return 0; }
        
        todo!()
    }

    pub fn identify_drive(&mut self, which: BusDrive) {
        self.wait_for_done();
        println!("Done!");

        unsafe {
            self.drive_select.write(0xA0 | which as u8);
            self.sector_count.write(0);
            self.lba_high.write(0);
            self.lba_mid.write(0);
            self.lba_low.write(0);


            self.command.write(ATACommand::Identify as u8); // Causing Double Fault!

            if self.status().is_empty() {
                panic!("drive did not exist!");
            }

            while self.status().intersects(ATAStatus::BUSY) {
                if self.lba_mid.read() != 0 || self.lba_high.read() != 0 {
                    panic!("drive was not ATA!");
                }
            }

            println!("You gooooood!");
        }

    }

    fn wait_for_done(&mut self) {
        let mut timeout = 0;
        loop {
            let status = unsafe { self.status() };
            timeout += 1;
            if status.intersects(ATAStatus::ERROR | ATAStatus::DRIVE_WRITE_FAULT) {
                panic!("Error!");
            }
            if status.intersects(ATAStatus::BUSY) && timeout == 1_000_000 {
                serial_println!("ATA Driver has been looping for 1 million iterations!");
            }
            if !status.intersects(ATAStatus::DATA_REQUEST_READY) {
                return;
            }
        }
    }

    unsafe fn status(&mut self) -> ATAStatus {
        self.alt_status.read();
        self.alt_status.read();
        self.alt_status.read();
        self.alt_status.read();
        ATAStatus::from_bits_truncate(self.error.read())
    }
}

const SECTOR_SIZE: usize = 512;

#[derive(Default, Clone, Copy)]
struct IDEChannelRegisters {
    base: u8,
    ctrl: u8,
    bmide: u8,
}


pub unsafe fn read_boot_sector() -> FatBootSector {
    let mut buffer = [0u8; SECTOR_SIZE];
    unsafe {
        //ata_pio_read_sector(0, &mut buffer);
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
            //ata_pio_read_sector(root_start_sector + i as u32, &mut buf); }
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
