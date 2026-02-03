use std::sync::atomic::{AtomicU8, AtomicUsize, AtomicU64};

pub const SLOT_FREE: u8 = 0;
pub const SLOT_WRITING: u8 = 1;
pub const SLOT_READY: u8 = 2;
pub const SLOT_READING: u8 = 3;

pub const MAX_SLOTS: usize = 1024; // Keep default for tests
pub const PAYLOAD_SIZE: usize = 1024;

#[repr(C, align(8))] // Explicit alignment for 64-bit systems
pub struct Slot {
    pub status: AtomicU8,      // 1 byte
    pub _pad: [u8; 3],         // 3 bytes padding (align u32 below)
    pub data_len: u32,         // 4 bytes
    pub timestamp: u64,        // 8 bytes
    pub payload: [u8; PAYLOAD_SIZE],
}

#[repr(C, align(8))]
pub struct ShmemHeader {
    pub version: u32,          // 4
    pub _pad: u32,             // 4 (align to 8)
    pub num_slots: usize,      // 8
    pub slot_size: usize,      // 8
    pub write_index: AtomicUsize, // 8
    pub read_index: AtomicUsize,  // 8
    pub dropped_count: AtomicU64, // 8
}

pub fn get_total_shmem_size() -> usize {
    std::mem::size_of::<ShmemHeader>() + (std::mem::size_of::<Slot>() * MAX_SLOTS)
}
