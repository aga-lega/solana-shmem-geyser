// Inside the plugin struct in lib.rs
impl GeyserPlugin for SharedMemoryBridgePlugin {
    fn on_load(&mut self, _config_file: &str) -> GeyserResult<()> {
        // 1. Open the segment created by init-bridge
        let shmem = ShmemConf::new()
            .os_path("/solana_bridge")
            .open()
            .map_err(|e| GeyserPluginError::Custom(Box::new(e)))?;

        let raw_ptr = shmem.as_ptr();

        // 2. Wire header and slot pointers
        unsafe {
            self.header = raw_ptr as *mut ShmemHeader;
            self.slots = raw_ptr.add(std::mem::size_of::<ShmemHeader>()) as *mut Slot;
            
            // Validate protocol version
            if (*self.header).version != 1 {
                return Err(GeyserPluginError::Custom(Box::new(
                    std::io::Error::new(std::io::ErrorKind::Other, "Protocol version mismatch")
                )));
            }
        }

        println!("[shmem-bridge] Plugin attached to validator shared memory.");
        Ok(())
    }
    
    // notify_transaction follows...
}
