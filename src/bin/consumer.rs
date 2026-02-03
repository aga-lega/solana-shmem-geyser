use shared_memory::*;
use solana_shmem_bridge::shmem_proto::*;
use std::sync::atomic::Ordering;

fn main() {
    let size = get_total_shmem_size();
    let shmem = ShmemConf::new()
        .size(size)
        .os_id("solana_bridge")
        .open()
        .expect("Failed to open shared memory");

    let raw_ptr = shmem.as_ptr();
    if (raw_ptr as usize) % 8 != 0 {
        panic!("Memory not 8-byte aligned!");
    }

    let (header, slots) = unsafe {
        let header = raw_ptr as *mut ShmemHeader;
        let header_offset = std::mem::size_of::<ShmemHeader>();
        let slots = raw_ptr.add(header_offset) as *mut Slot;
        (header, slots)
    };

    println!("[shmem-bridge][consumer] Attached; waiting for slots.");

    loop {
        let idx = unsafe { (*header).read_index.fetch_add(1, Ordering::SeqCst) % MAX_SLOTS };
        let slot = unsafe { &*slots.add(idx) };

        if slot
            .status
            .compare_exchange(SLOT_READY, SLOT_READING, Ordering::Acquire, Ordering::Relaxed)
            .is_ok()
        {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos() as u64;

            let diff_ns = now.saturating_sub(slot.timestamp);
            let len = (slot.data_len as usize).min(PAYLOAD_SIZE);

            println!(
                "[shmem-bridge][consumer] Slot {} latency: {} ns | sig: {:?}",
                idx,
                diff_ns,
                &slot.payload[..len.min(8)]
            );

            slot.status.store(SLOT_FREE, Ordering::Release);
        } else {
            std::thread::yield_now();
        }
    }
}
