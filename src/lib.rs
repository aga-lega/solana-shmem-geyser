use agave_geyser_plugin_interface::geyser_plugin_interface::*;
use std::sync::atomic::Ordering;
use shared_memory::*;

pub mod shmem_proto;
use shmem_proto::*;

pub struct ShmemBridgePlugin {
    shmem: Option<Shmem>, 
    header: *mut ShmemHeader,
    slots: *mut Slot,
}

impl std::fmt::Debug for ShmemBridgePlugin {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ShmemBridgePlugin")
    }
}

unsafe impl Send for ShmemBridgePlugin {}
unsafe impl Sync for ShmemBridgePlugin {}

impl Default for ShmemBridgePlugin {
    fn default() -> Self {
        Self {
            shmem: None,
            header: std::ptr::null_mut(),
            slots: std::ptr::null_mut(),
        }
    }
}

impl GeyserPlugin for ShmemBridgePlugin {
    fn name(&self) -> &'static str { "ShmemBridge" }

    // These two methods enable data streaming
    fn transaction_notifications_enabled(&self) -> bool {
        true
    }

    fn account_data_notifications_enabled(&self) -> bool {
        true
    }

    fn on_load(&mut self, _config_file: &str, _is_reloading: bool) -> Result<()> {
        let size = get_total_shmem_size();
        // Cross-platform ID
        let os_id = if cfg!(target_os = "linux") { "/solana_bridge" } else { "solana_bridge" };
        
        let shmem = ShmemConf::new()
            .size(size)
            .os_id(os_id)
            .open()
            .map_err(|e| GeyserPluginError::Custom(Box::new(e)))?;

        let raw_ptr = shmem.as_ptr();

        if (raw_ptr as usize) % 8 != 0 {
            return Err(GeyserPluginError::Custom(Box::new(
                std::io::Error::new(std::io::ErrorKind::Other, "Memory not 8-byte aligned!")
            )));
        }

        unsafe {
            self.header = raw_ptr as *mut ShmemHeader;
            let header_offset = std::mem::size_of::<ShmemHeader>();
            self.slots = raw_ptr.add(header_offset) as *mut Slot;
        }

        self.shmem = Some(shmem);
        println!("[shmem-bridge] Plugin loaded; waiting for transactions.");
        Ok(())
    }

    fn notify_transaction(
        &self, 
        tx_info: ReplicaTransactionInfoVersions, 
        _slot_num: u64
    ) -> Result<()> {
        if self.header.is_null() || self.slots.is_null() { return Ok(()); }

        // Visible in the validator logs.
        eprintln!("[shmem-bridge][debug] Transaction received.");

        unsafe {
            let header = &*self.header;
            let idx = header.write_index.fetch_add(1, Ordering::SeqCst) % MAX_SLOTS;
            let slot_ptr = self.slots.add(idx);
            let slot = &*slot_ptr;

            if slot.status.compare_exchange(
                SLOT_FREE, SLOT_WRITING, 
                Ordering::Acquire, Ordering::Relaxed
            ).is_ok() {
                
                let sig_bytes = match tx_info {
                    ReplicaTransactionInfoVersions::V0_0_1(t) => t.signature.as_ref(),
                    ReplicaTransactionInfoVersions::V0_0_2(t) => t.signature.as_ref(),
                    ReplicaTransactionInfoVersions::V0_0_3(t) => t.signature.as_ref(),
                };

                let len = sig_bytes.len().min(PAYLOAD_SIZE);
                
                std::ptr::copy_nonoverlapping(
                    sig_bytes.as_ptr(), 
                    (*slot_ptr).payload.as_ptr() as *mut u8, 
                    len
                );
                
                (*slot_ptr).data_len = len as u32;
                (*slot_ptr).timestamp = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos() as u64;

                slot.status.store(SLOT_READY, Ordering::Release);
            } else {
                header.dropped_count.fetch_add(1, Ordering::Relaxed);
            }
        }
        Ok(())
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn _create_plugin() -> *mut dyn GeyserPlugin {
    let plugin = ShmemBridgePlugin::default();
    let boxed = Box::new(plugin);
    Box::into_raw(boxed)
}
