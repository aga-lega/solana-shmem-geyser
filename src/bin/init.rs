use shared_memory::*;
use solana_shmem_bridge::shmem_proto::*;
use std::ptr;

fn main() {
    let size = get_total_shmem_size();
    let os_id = "solana_bridge"; // Remove the slash on macOS if the library is strict

    println!("[shmem-bridge][init] Starting shared memory reset.");

    // 1. Try opening and releasing the old segment if present
    if let Ok(shm) = ShmemConf::new().os_id(os_id).open() {
        println!("[shmem-bridge][init] Existing segment found; releasing.");
        drop(shm); // On some OSes this marks the segment for deletion
    }

    // 2. Create a new segment
    let mut shmem = match ShmemConf::new()
        .size(size)
        .os_id(os_id)
        .create() {
            Ok(m) => {
                println!("[shmem-bridge][init] Created new shared memory segment.");
                m
            },
            Err(e) => {
                println!("[shmem-bridge][init] Create failed; trying open(): {}", e);
                ShmemConf::new().os_id(os_id).open().expect("Failed to open shared memory segment")
            }
        };

    let raw_ptr = shmem.as_ptr();

    // 3. Hard reset (zero the entire region)
    println!("[shmem-bridge][init] Zeroing {} bytes (resetting flags)...", size);
    unsafe {
        ptr::write_bytes(raw_ptr, 0, size);
        
        // Initialize header after zeroing
        let header = raw_ptr as *mut ShmemHeader;
        (*header).version = 1;
        (*header).num_slots = MAX_SLOTS;
        (*header).slot_size = std::mem::size_of::<Slot>();
    }

    println!("[shmem-bridge][init] Shared memory is clean and ready.");
    println!("[shmem-bridge][init] Start the validator when ready.");
    
    // Keep the process alive so the segment is not removed (critical on macOS)
    loop { std::thread::park(); }
}
