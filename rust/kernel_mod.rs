//! Rust kernel module.
use core::cmp::min;
use kernel::error::code;
use kernel::prelude::*;
use kernel::sync::{smutex::Mutex, Arc, ArcBorrow};
use kernel::{file, miscdev};
use present80::key::Key;

use crate::present80::Present80;

mod present80;

module! {
    type: KernelModule,
    name: "rust_misc_dev",
    author: "Ege Bilecen",
    description: "Miscellaneous device written in Rust.",
    license: "GPL",
}

const DEV_PREFIX: &str = "present80";

struct KernelModule {
    _key_dev: Pin<Box<miscdev::Registration<DeviceOperations>>>,
    _encrypt_dev: Pin<Box<miscdev::Registration<DeviceOperations>>>,
}

enum DeviceType {
    Key,
    Encryption,
}

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
    in_buffer: Vec<u8>,
    out_buffer: Vec<u8>,
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
        pr_info!("{} device is opened.", &data.r#type.as_str());

        let device = data.as_arc_borrow();
        let mut device = (get_device_inner(&device)).lock();

        if device.is_in_use {
            return Err(code::EBUSY);
        }

        device.is_in_use = true;
        device.in_buffer.clear();
        device.out_buffer.clear();

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
            pr_warn!("Key device doesn't support read operation.");
            return Err(code::EPERM);
        }

        let mut device = data.encryption.lock();
        let key_device = data.key.lock();

        if device.out_buffer.is_empty() {
            let key = Key::try_from(key_device.in_buffer.as_slice())?;
            let cipher = Present80::new(key);
            let cipher_text = cipher.encrypt(device.in_buffer.as_slice())?;

            let out_buffer = &mut device.out_buffer;
            out_buffer.try_extend_from_slice(&cipher_text)?;
        }

        let out_buffer = &device.out_buffer;

        let offset = usize::try_from(offset)?;
        let len = min(writer.len(), out_buffer.len().saturating_sub(offset));
        writer.write_slice(&out_buffer[offset..][..len])?;

        Ok(len)
    }

    fn write(
        data: ArcBorrow<'_, Device>,
        _file: &file::File,
        reader: &mut impl kernel::io_buffer::IoBufferReader,
        offset: u64,
    ) -> Result<usize> {
        let recv_bytes = reader.read_all()?;
        let mut device = (get_device_inner(&data)).lock();

        let in_buffer = &mut device.in_buffer;

        if offset == 0 {
            in_buffer.clear();
        }

        in_buffer.try_extend_from_slice(&recv_bytes[..])?;

        if offset == 0 {
            if let DeviceType::Encryption = data.r#type {
                let out_buffer = &mut device.out_buffer;
                out_buffer.clear();
            }
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
        pr_info!("Initializing...\n");

        let key_dev_inner = Arc::try_new(Mutex::new(DeviceInner {
            is_in_use: false,
            in_buffer: Vec::new(),
            out_buffer: Vec::new(),
        }))?;

        let encryption_dev_inner = Arc::try_new(Mutex::new(DeviceInner {
            is_in_use: false,
            in_buffer: Vec::new(),
            out_buffer: Vec::new(),
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
        pr_info!("Exiting...\n");
    }
}
