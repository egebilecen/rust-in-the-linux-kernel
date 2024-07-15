//! Rust kernel module.
use crate::present80::Present80;
use core::cmp::min;
use kernel::error::code;
use kernel::prelude::*;
use kernel::sync::{smutex::Mutex, Arc, ArcBorrow};
use kernel::{file, miscdev};

mod present80;

// Setup the module and set the properties.
module! {
    type: KernelModule,
    name: "rust_misc_dev",
    author: "Ege Bilecen",
    description: "Miscellaneous device written in Rust.",
    license: "GPL",
}

// Device prefix.
const DEV_PREFIX: &str = "present80";
// Max. buffer size to allocate in stack.
const MAX_BUFFER_SIZE: usize = 10;

// Kernel module setup.
struct KernelModule {
    _key_dev: Pin<Box<miscdev::Registration<DeviceOperations>>>,
    _encrypt_dev: Pin<Box<miscdev::Registration<DeviceOperations>>>,
}

// Device type.
enum DeviceType {
    Key,
    Encryption,
}

// Device data.
struct DeviceInner {
    is_in_use: bool,
    in_buffer: [u8; MAX_BUFFER_SIZE],
    out_buffer: [u8; MAX_BUFFER_SIZE],
}

// Device. Each device has access to the other device.
struct Device {
    r#type: DeviceType,
    key: Arc<Mutex<DeviceInner>>,
    encryption: Arc<Mutex<DeviceInner>>,
}

// Convenience function to get the "DeviceInner" from "Device".
#[inline]
fn get_device_inner<'a>(dev: &'a ArcBorrow<'a, Device>) -> &'a Arc<Mutex<DeviceInner>> {
    match dev.r#type {
        DeviceType::Key => &dev.key,
        DeviceType::Encryption => &dev.encryption,
    }
}

struct DeviceOperations;

#[vtable]
impl file::Operations for DeviceOperations {
    // Type of the data.
    type Data = Arc<Device>;
    // Type of the open data.
    type OpenData = Self::Data;

    // Called when device is opened.
    fn open(data: &Self::OpenData, _file: &file::File) -> Result<Self::Data> {
        let device = data.as_arc_borrow();
        // Get device inner.
        let mut device = get_device_inner(&device).lock();

        // Check if device is in use already. Return "EBUSY" error if in use.
        if device.is_in_use {
            return Err(code::EBUSY);
        }

        // Set the device as in use.
        device.is_in_use = true;

        // Clear the input and output buffer of the device.
        device.in_buffer.fill(0);
        device.out_buffer.fill(0);

        Ok(data.clone())
    }

    // Called when device is read.
    fn read(
        data: ArcBorrow<'_, Device>,
        _file: &file::File,
        writer: &mut impl kernel::io_buffer::IoBufferWriter,
        offset: u64,
    ) -> Result<usize> {
        // Key device doesn't support read operation.
        if let DeviceType::Key = data.r#type {
            pr_err!("Key device doesn't support read operation.");
            pr_info!("");
            return Err(code::EPERM);
        }

        // Partial read is not supported. Return "EINVAL" error.
        if offset != 0 {
            pr_err!("Encryption device doesn't support partial read. Offset is not 0.");
            pr_info!("");
            return Err(code::EINVAL);
        }

        // Get the encryption device.
        let mut device = data.encryption.lock();
        // Encryption device needs the key set in the key device. Get the key device data.
        let key_device = data.key.lock();

        // Create the PRESENT-80 cipher.
        let cipher = Present80::new(&key_device.in_buffer);

        // Convert slice reference to fixed size slice as cipher relies on fixed size slice.
        let buffer = if let Ok(val) =
            <[u8; present80::BLOCK_SIZE]>::try_from(&device.in_buffer[..present80::BLOCK_SIZE])
        {
            val
        } else {
            return Err(code::ENOMEM);
        };

        // Perform the encryption.
        let cipher_text = cipher.encrypt(&buffer)?;

        // Copy the encryption result to the output buffer.
        for (i, &byte) in cipher_text.iter().enumerate() {
            device.out_buffer[i] = byte;
        }

        // Write the result in output buffer to the user space.
        let offset = usize::try_from(offset)?;
        let len = min(writer.len(), device.out_buffer.len().saturating_sub(offset));
        writer.write_slice(&device.out_buffer[offset..][..len])?;

        Ok(len)
    }

    // Called when some bytes written into device.
    fn write(
        data: ArcBorrow<'_, Device>,
        _file: &file::File,
        reader: &mut impl kernel::io_buffer::IoBufferReader,
        offset: u64,
    ) -> Result<usize> {
        // Partial write is not supported. Return "EINVAL" error.
        if offset != 0 {
            pr_err!("PRESENT80 devices doesn't support partial write. Offset is not 0.");
            pr_info!("");
            return Err(code::EINVAL);
        }

        // Read the user space data.
        let recv_bytes = reader.read_all()?;

        // Validate the size of the incoming data before proceeding further.
        let len = match data.r#type {
            DeviceType::Key => {
                if recv_bytes.len() != present80::KEY_SIZE {
                    pr_err!(
                        "Key device requires {} bytes to be written. Found {} bytes.",
                        present80::KEY_SIZE,
                        recv_bytes.len()
                    );
                    pr_info!("");
                    return Err(code::EINVAL);
                }

                present80::KEY_SIZE
            }
            DeviceType::Encryption => {
                if recv_bytes.len() != present80::BLOCK_SIZE {
                    pr_err!(
                        "Encryption device requires {} bytes to be written. Found {} bytes.",
                        present80::BLOCK_SIZE,
                        recv_bytes.len()
                    );
                    pr_info!("");
                    return Err(code::EINVAL);
                }

                present80::BLOCK_SIZE
            }
        };

        // Get the device.
        let mut device = get_device_inner(&data).lock();
        // Copy incoming data to the input buffer.
        device.in_buffer[..len].copy_from_slice(&recv_bytes);

        Ok(recv_bytes.len())
    }

    // Called when device is closed.
    fn release(data: Self::Data, _file: &file::File) {
        // Get the related device.
        let device = data.as_arc_borrow();
        let mut device = get_device_inner(&device).lock();

        // Set the device as not in use.
        device.is_in_use = false;
    }
}

impl kernel::Module for KernelModule {
    // Called when the module is loaded.
    fn init(_name: &'static CStr, _module: &'static ThisModule) -> Result<Self> {
        // Initialize the device inners.
        let key_dev_inner = Arc::try_new(Mutex::new(DeviceInner {
            is_in_use: false,
            in_buffer: [0; MAX_BUFFER_SIZE],
            out_buffer: [0; MAX_BUFFER_SIZE],
        }))?;

        let encryption_dev_inner = Arc::try_new(Mutex::new(DeviceInner {
            is_in_use: false,
            in_buffer: [0; MAX_BUFFER_SIZE],
            out_buffer: [0; MAX_BUFFER_SIZE],
        }))?;

        // Create the device and assign the inners.
        let key_dev = Arc::try_new(Device {
            r#type: DeviceType::Key,
            key: key_dev_inner.clone(),
            encryption: encryption_dev_inner.clone(),
        })?;

        let encryption_dev = Arc::try_new(Device {
            r#type: DeviceType::Encryption,
            key: key_dev_inner.clone(),
            encryption: encryption_dev_inner.clone(),
        })?;

        // Register the misc. devices.
        Ok(KernelModule {
            _key_dev: miscdev::Registration::new_pinned(fmt!("{}_key", DEV_PREFIX), key_dev)?,
            _encrypt_dev: miscdev::Registration::new_pinned(
                fmt!("{}_encrypt", DEV_PREFIX),
                encryption_dev,
            )?,
        })
    }
}

impl Drop for KernelModule {
    fn drop(&mut self) {}
}
