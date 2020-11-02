/* 8259 PIC (Programmable Interrupt Controller) */
//                      ____________                          ____________
// Real Time Clock --> |            |   Timer -------------> |            |
// ACPI -------------> |            |   Keyboard-----------> |            |      _____
// Available --------> | Secondary  |----------------------> | Primary    |     |     |
// Available --------> | Interrupt  |   Serial Port 2 -----> | Interrupt  |---> | CPU |
// Mouse ------------> | Controller |   Serial Port 1 -----> | Controller |     |_____|
// Co-Processor -----> |            |   Parallel Port 2/3 -> |            |
// Primary ATA ------> |            |   Floppy disk -------> |            |
// Secondary ATA ----> |____________|   Parallel Port 1----> |____________|

/*
 * Each controller can be configured through two I/O ports, 
 * one “command” port and one “data” port. For the primary 
 * controller these ports are 0x20 (command) and 0x21 (data). 
 * For the secondary controller they are 0xa0 (command) and 
 * 0xa1 (data). For more information on how the PICs can be 
 * configured see the article on osdev.org.
 */
use pic8259_simple::ChainedPics;
use spin;

/*
 * 0-15 is alredy uesd by CPU exceptions, so wo choose range [32, 47]
 * These are first free numbers after the 32 exception slots.
 */
pub const PRIMARY_PIC_OFFSET: u8 = 32;
pub const SECONDARY_PIC_OFFSET: u8 = PRIMARY_PIC_OFFSET + 8;

pub static PICS: spin::Mutex<ChainedPics> = spin::Mutex::new(unsafe {
    ChainedPics::new(PRIMARY_PIC_OFFSET, SECONDARY_PIC_OFFSET)
});

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptIndex {
    Timer = PRIMARY_PIC_OFFSET,
}

impl InterruptIndex {
    pub fn as_u8(self) -> u8 {
        self as u8
    }
    pub fn as_usize(self) -> usize {
        usize::from(self as u8)
    }
}