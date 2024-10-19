use fdt_rs::prelude::{FallibleIterator, PropReader};
use flattened_device_tree::error::FdtError;
use flattened_device_tree::FlattenedDeviceTree;

fn read_unaligned_u64(ptr: *const u8) -> u64 {
    // Step 1: Create a temporary array to hold the bytes
    let mut buf = [0u8; 8]; // For u64, we need 8 bytes

    // Step 2: Read the bytes from the unaligned pointer
    unsafe {
        // Copy bytes from the pointer into the buffer
        for i in 0..8 {
            buf[i] = *ptr.add(i); // Use add(i) to get each byte
        }
    }

    // Step 3: Convert the byte array to u64
    u64::from_be_bytes(buf) // Convert to u64 (you can use from_be_bytes if needed)
}

fn write_unaligned_u64(ptr: *mut u8, value: u64) {
    // Step 1: Convert the u64 value to a byte array
    let bytes = value.to_be_bytes(); // You can use to_be_bytes() or to_ne_bytes() depending on endianness

    // Step 2: Write the bytes to the unaligned pointer
    unsafe {
        for i in 0..8 {
            *ptr.add(i) = bytes[i]; // Use add(i) to get each byte's address
        }
    }
}

pub fn divide_memory_region_size(device_tree_blob_addr: usize) -> Result<(), FdtError> {
    let fdt: FlattenedDeviceTree;
    unsafe { fdt = FlattenedDeviceTree::from_raw_pointer(device_tree_blob_addr as *const u8)? }

    let mem_prop = fdt
        .inner
        .props()
        .find(|p| Ok(p.name()? == "device_type" && p.str()? == "memory"))?
        .ok_or_else(|| FdtError::NoMemoryNode())?;

    let reg_prop = mem_prop
        .node()
        .props()
        .find(|p| Ok(p.name().unwrap_or("empty") == "reg"))?
        .ok_or_else(|| FdtError::NoMemoryNode())?;

    unsafe {
        let ptr: *const u8 = reg_prop.propbuf().as_ptr().add(8);

        let memory_size = read_unaligned_u64(ptr);
        write_unaligned_u64(ptr as *mut u8, memory_size / 2);
    }

    Ok(())
}
