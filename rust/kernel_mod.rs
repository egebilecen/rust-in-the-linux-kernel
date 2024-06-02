//! Rust kernel module.
use crate::present80::Present80;
use core::cmp::min;
use kernel::error::code;
use kernel::prelude::*;
use kernel::sync::{smutex::Mutex, Arc, ArcBorrow};
use kernel::{file, miscdev};

mod present80;

module! {
    type: KernelModule,
    name: "rust_misc_dev",
    author: "Ege Bilecen",
    description: "Miscellaneous device written in Rust.",
    license: "GPL",
}

const DEV_PREFIX: &str = "present80";
const MAX_BUFFER_SIZE: usize = 10;

struct KernelModule {
    _key_dev: Pin<Box<miscdev::Registration<DeviceOperations>>>,
    _encrypt_dev: Pin<Box<miscdev::Registration<DeviceOperations>>>,
}

enum DeviceType {
    Key,
    Encryption,
}

#[allow(unused)]
impl DeviceType {
    fn as_str(&self) -> &str {
        match self {
            DeviceType::Key => "Key",
            DeviceType::Encryption => "Encryption",
        }
    }
}

struct DeviceInner {
    is_in_use: bool,
    in_buffer: [u8; MAX_BUFFER_SIZE],
    out_buffer: [u8; MAX_BUFFER_SIZE],
}

struct Device {
    r#type: DeviceType,
    key: Arc<Mutex<DeviceInner>>,
    encryption: Arc<Mutex<DeviceInner>>,
}

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
    type Data = Arc<Device>;
    type OpenData = Self::Data;

    fn open(data: &Self::OpenData, _file: &file::File) -> Result<Self::Data> {
        let device = data.as_arc_borrow();
        let mut device = (get_device_inner(&device)).lock();

        if device.is_in_use {
            return Err(code::EBUSY);
        }

        device.is_in_use = true;
        device.in_buffer.fill(0);
        device.out_buffer.fill(0);

        Ok(data.clone())
    }

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

        if offset != 0 {
            pr_err!("Encryption device doesn't support partial read. Offset is not 0.");
            pr_info!("");
            return Err(code::EINVAL);
        }

        let mut device = data.encryption.lock();
        let key_device = data.key.lock();

        let cipher = Present80::new(&key_device.in_buffer);
        let buffer = if let Ok(val) =
            <[u8; present80::BLOCK_SIZE]>::try_from(&device.in_buffer[..present80::BLOCK_SIZE])
        {
            val
        } else {
            return Err(code::ENOMEM);
        };
        let cipher_text = cipher.encrypt(&buffer)?;

        for (i, &byte) in cipher_text.iter().enumerate() {
            device.out_buffer[i] = byte;
        }

        let offset = usize::try_from(offset)?;
        let len = min(writer.len(), device.out_buffer.len().saturating_sub(offset));
        writer.write_slice(&device.out_buffer[offset..][..len])?;

        Ok(len)
    }

    fn write(
        data: ArcBorrow<'_, Device>,
        _file: &file::File,
        reader: &mut impl kernel::io_buffer::IoBufferReader,
        offset: u64,
    ) -> Result<usize> {
        if offset != 0 {
            pr_err!("PRESENT80 devices doesn't support partial write. Offset is not 0.");
            pr_info!("");
            return Err(code::EINVAL);
        }

        let recv_bytes = reader.read_all()?;

        match data.r#type {
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
            }
        }

        let mut device = (get_device_inner(&data)).lock();

        for (i, &byte) in recv_bytes.iter().enumerate() {
            device.in_buffer[i] = byte;
        }

        Ok(recv_bytes.len())
    }

    fn release(data: Self::Data, _file: &file::File) {
        let device = data.as_arc_borrow();
        let mut device = (get_device_inner(&device)).lock();

        device.is_in_use = false;
    }
}

impl kernel::Module for KernelModule {
    fn init(_name: &'static CStr, _module: &'static ThisModule) -> Result<Self> {
        pr_info!("Initializing...");
        pr_info!("");

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
    fn drop(&mut self) {
        pr_info!("Exiting...");
        pr_info!("");
    }
}
