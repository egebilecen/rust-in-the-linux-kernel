//! Rust kernel module.
use core::cmp::min;
use kernel::error::code;
use kernel::prelude::*;
use kernel::sync::{smutex::Mutex, Arc, ArcBorrow};
use kernel::{file, miscdev};

module! {
    type: DeviceDriver,
    name: "rust_misc_dev",
    author: "Ege Bilecen",
    description: "Miscellaneous device written in Rust.",
    license: "GPL",
}

const DEV_PREFIX: &str = "present80";

static mut KEY_DEV: Mutex<DeviceInner> = Mutex::new(DeviceInner {
    is_in_use: false,
    in_buffer: Vec::new(),
    out_buffer: Vec::new(),
});

static mut ENCRYPTION_DEV: Mutex<DeviceInner> = Mutex::new(DeviceInner {
    is_in_use: false,
    in_buffer: Vec::new(),
    out_buffer: Vec::new(),
});

struct DeviceDriver {
    _key_dev: Pin<Box<miscdev::Registration<DeviceOperations>>>,
    _encrypt_dev: Pin<Box<miscdev::Registration<DeviceOperations>>>,
}

enum DeviceType {
    KEY,
    ENCRYPTION,
}

struct DeviceInner {
    is_in_use: bool,
    in_buffer: Vec<u8>,
    out_buffer: Vec<u8>,
}

struct Device {
    r#type: DeviceType,
    key: &'static Mutex<DeviceInner>,
    encryption: &'static Mutex<DeviceInner>,
}

struct DeviceOperations;

#[vtable]
impl file::Operations for DeviceOperations {
    type Data = Arc<Device>;
    type OpenData = Arc<Device>;

    fn open(data: &Self::OpenData, _file: &file::File) -> Result<Self::Data> {
        let mut device = match data.r#type {
            DeviceType::KEY => data.key,
            DeviceType::ENCRYPTION => data.encryption,
        }
        .lock();

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
        let device = match data.r#type {
            DeviceType::KEY => data.key,
            DeviceType::ENCRYPTION => data.encryption,
        }
        .lock();
        let buffer = &device.in_buffer;

        let offset = usize::try_from(offset)?;
        let len = min(writer.len(), buffer.len().saturating_sub(offset));
        writer.write_slice(&buffer[offset..][..len])?;

        Ok(len)
    }

    fn write(
        data: ArcBorrow<'_, Device>,
        _file: &file::File,
        reader: &mut impl kernel::io_buffer::IoBufferReader,
        _offset: u64,
    ) -> Result<usize> {
        let recv_bytes = reader.read_all()?;

        let mut device = match data.r#type {
            DeviceType::KEY => data.key,
            DeviceType::ENCRYPTION => data.encryption,
        }
        .lock();
        let buffer = &mut device.in_buffer;

        buffer.clear();
        buffer.try_extend_from_slice(&recv_bytes[..])?;

        match (*data).r#type {
            DeviceType::KEY => {
                pr_info!("Written into key device.");
            }
            DeviceType::ENCRYPTION => {
                pr_info!("Written into encryption device.");
            }
        }

        Ok(recv_bytes.len())
    }

    fn release(data: Self::Data, _file: &file::File) {
        let mut device = match data.r#type {
            DeviceType::KEY => data.key,
            DeviceType::ENCRYPTION => data.encryption,
        }
        .lock();

        (*device).is_in_use = false;
    }
}

impl kernel::Module for DeviceDriver {
    fn init(_name: &'static CStr, _module: &'static ThisModule) -> Result<Self> {
        pr_info!("Initializing.\n");

        let key_dev = Arc::try_new(Device {
            r#type: DeviceType::KEY,
            key: unsafe { &KEY_DEV },
            encryption: unsafe { &ENCRYPTION_DEV },
        })?;

        let encryption_dev = Arc::try_new(Device {
            r#type: DeviceType::ENCRYPTION,
            key: unsafe { &KEY_DEV },
            encryption: unsafe { &ENCRYPTION_DEV },
        })?;

        Ok(DeviceDriver {
            _key_dev: miscdev::Registration::new_pinned(fmt!("{}_key", DEV_PREFIX), key_dev)?,
            _encrypt_dev: miscdev::Registration::new_pinned(
                fmt!("{}_encrypt", DEV_PREFIX),
                encryption_dev,
            )?,
        })
    }
}

impl Drop for DeviceDriver {
    fn drop(&mut self) {
        pr_info!("Exit.\n");
    }
}
