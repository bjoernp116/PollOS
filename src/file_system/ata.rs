use super::io::*;

type Result<T> = core::result::Result<T, anyhow::Error>;
use anyhow::anyhow;

use bitflags::bitflags;

use crate::{println, warn};
use crate::serial_println;

use super::SECTOR_SIZE;

const PORT_MASK: u16 = 0xFFFC;
const TIMEOUT: usize = 1000;

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
    Slave = 1 << 4,
}

#[repr(u8)]
#[allow(unused)]
enum ATACommand {
    Read = 0x20,
    Write = 0x30,
    CacheFlush = 0xE7,
    Identify = 0xEC,
}


#[allow(unused)]
#[derive(Debug)]
pub struct ATABus {
    data: PortRW<u16>,              // BAR0 + 0
    error: PortReader<u8>,      // BAR0 + 1
    sector_count: PortRW<u8>,       // BAR0 + 2
    lba_low: PortRW<u8>,            // BAR0 + 3
    lba_mid: PortRW<u8>,            // BAR0 + 4
    lba_high: PortRW<u8>,           // BAR0 + 5
    drive_select: PortRW<u8>,       // BAR0 + 6
    command: PortWriter<u8>,   // BAR0 + 7
    status: PortReader<u8>,     // BAR0 + 7
    alt_status: PortReader<u8>, // BAR1 + 2
    control: PortWriter<u8>,   // BAR1 + 2
}

impl ATABus {
    pub fn new(data_bar: u16, ctrl_bar: u16) -> Self {
        let data_bar = data_bar & PORT_MASK;
        let ctrl_bar = ctrl_bar & PORT_MASK;

        ATABus {
            data: PortRW::new(data_bar),
            error: PortReader::new(data_bar + 1),
            sector_count: PortRW::new(data_bar + 2),
            lba_low: PortRW::new(data_bar + 3),
            lba_mid: PortRW::new(data_bar + 4),
            lba_high: PortRW::new(data_bar + 5),
            drive_select: PortRW::new(data_bar + 6),
            command: PortWriter::new(data_bar + 7),
            status: PortReader::new(data_bar + 7),
            control: PortWriter::new(ctrl_bar + 2),
            alt_status: PortReader::new(ctrl_bar + 2),
        }
    }

    pub fn read(
        &self,
        buffer: &mut [u8],
        which: BusDrive,
        lba_start: usize,
        sector_count: usize,
    ) -> Result<usize> {
        if sector_count == 0 {
            return Ok(0);
        }

        let using_lba_28 = true;

        self.wait_for_done()?;

        if using_lba_28 {
            self.drive_select.write(
                0xE0 | (which as u8) | ((lba_start >> 24) as u8 & 0x0F));
            self.sector_count.write(sector_count as u8);
            self.lba_high.write((lba_start >> 16) as u8);
            self.lba_mid.write((lba_start >> 8) as u8);
            self.lba_low.write(lba_start as u8);
            self.command.write(ATACommand::Read as u8);
        }

        let mut buffer_offset = 0;
        for _lba in lba_start .. (lba_start + sector_count) {
            self.wait_for_ready()?;

            let range = buffer_offset .. (buffer_offset + SECTOR_SIZE);
            for chunk in buffer[range].chunks_exact_mut(2) {
                let word: u16 = self.data.read();
                chunk[0] = word as u8;
                chunk[1] = (word >> 8) as u8;
            }
            buffer_offset += SECTOR_SIZE;
        }
        self.wait_for_done()?;
        Ok(sector_count)
    }

    pub fn identify(&self, which: BusDrive) -> Result<DriveIdentity> {
        self.wait_for_done()?;

        self.drive_select.write(0xA0 | which as u8);
        //self.wait_for_done();
        self.sector_count.write(0);
        self.lba_high.write(0);
        self.lba_mid.write(0);
        self.lba_low.write(0);

        self.command.write(0xEC); //ATACommand::Identify as u8);
        println!("pointer");

        if self.status().is_empty() {
            return Err(anyhow!("drive did not exist!"));
        }

        while self.status().intersects(ATAStatus::BUSY) {
            if self.lba_mid.read() != 0 || self.lba_high.read() != 0 {
                return Err(anyhow!("drive was not ATA!"));
            }
        }

        let mut buffer: [u8; SECTOR_SIZE] = [0; SECTOR_SIZE];
        self.wait_for_ready()?;
        for chunk in buffer.chunks_exact_mut(2) {
            let word: u16 = self.data.read();
            chunk[0] = word as u8;
            chunk[1] = (word >> 8) as u8;
        }
        self.wait_for_done()?;
        Ok(DriveIdentity::new(buffer))
    }


    fn wait_for_done(&self) -> Result<()> {
        let mut timeout = 0;
        loop {
            let status = self.status();
            timeout += 1;
            if status.intersects(ATAStatus::ERROR | ATAStatus::DRIVE_WRITE_FAULT) {
                return Err(anyhow!("Status intersects ERROR | DRIVE WRITE FAULT"))
            }
            if status.intersects(ATAStatus::BUSY) && timeout == TIMEOUT {
                serial_println!("ATA Driver has been looping for 1 million iterations!");
            }
            if !status.intersects(ATAStatus::DATA_REQUEST_READY) {
                return Ok(());
            }
        }
    }

    fn wait_for_ready(&self) -> Result<()> {
		let mut _loop_counter = 0;
		loop {
			let status = self.status();
			_loop_counter += 1;
			if status.intersects(ATAStatus::ERROR | ATAStatus::DRIVE_WRITE_FAULT) {
				return Err(anyhow!("Status intersects ERROR | DRIVE WRITE FAULT"));
			}
			if status.intersects(ATAStatus::BUSY) { 
				if _loop_counter % TIMEOUT == 0 {
					warn!("AtaBus::wait_for_data_ready() has been busy waiting for a long time... is there a device/driver problem? (status: {:?})", status);
				}
				continue;
			}
			if status.intersects(ATAStatus::DATA_REQUEST_READY) {
				return Ok(()); // ready to go!
			}
		}
	}

    fn status(&self) -> ATAStatus {
        self.alt_status.read();
        self.alt_status.read();
        self.alt_status.read();
        self.alt_status.read();
        ATAStatus::from_bits_truncate(self.status.read())
    }
}




#[allow(unused)]
#[derive(Debug)]
pub struct DriveIdentity {
    general_config: u16,
    model_number: [u8; 20],
    support_lba48: bool,
    user_adressable_sectors: u32,
}

impl DriveIdentity {
    pub fn new(buffer: [u8; SECTOR_SIZE]) -> Self {
        let general_config = u16::from_le_bytes([buffer[0], buffer[1]]);
        let model_number = [0; 20];
        let support_lba48 = get_bit_u16(u16::from_le_bytes([buffer[83], buffer[84]]), 10);
        let user_adressable_sectors = u32::from_le_bytes([
            buffer[120], buffer[121], buffer[122], buffer[123]
        ]);
        Self { general_config, model_number, user_adressable_sectors, support_lba48 }
    }
}



fn get_bit_u16(input: u16, bit: u8) -> bool {
    (input >> bit) & 0x1 == 1
}




pub fn test_ata_read(ata: &ATABus) {
    let mut buf = [0u8; SECTOR_SIZE];
    let result = ata.read(&mut buf, BusDrive::Master, 0, 1);

    match result {
        Ok(1) => {
            let signature = u16::from_le_bytes([buf[510], buf[511]]);
            if signature == 0xAA55 {
                println!("ATA PIO read success: Valid MBR signature found (0xAA55)");
            } else {
                panic!("ATA read failed: Invalid signature {:#X}", signature);
            }
        }
        Ok(n) => println!("ATA read returned unexpected sector count: {}", n),
        Err(_) => panic!("ATA read failed: read_pio() returned Err"),
    }
}

