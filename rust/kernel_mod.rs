//! Rust kernel module.
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

struct Device {
    buffer: Mutex<Vec<u8>>,
}

struct DeviceOperations;

#[vtable]
impl file::Operations for DeviceOperations {
    type Data = Arc<Device>;
    type OpenData = Arc<Device>;

    fn open(context: &Self::OpenData, _file: &file::File) -> Result<Self::Data> {
        Ok(context.clone())
    }

    fn read(
        data: ArcBorrow<'_, Device>,
        _file: &file::File,
        writer: &mut impl kernel::io_buffer::IoBufferWriter,
        offset: u64,
    ) -> Result<usize> {
        if writer.is_empty() || offset != 0 {
            return Ok(0);
        }

        let bytes = "Hello World!".as_bytes();
        writer.write_slice(bytes)?;

        Ok(bytes.len())
    }
}

struct DeviceDriver {
    _dev: Pin<Box<miscdev::Registration<DeviceOperations>>>,
}

impl kernel::Module for DeviceDriver {
    fn init(_name: &'static CStr, _module: &'static ThisModule) -> Result<Self> {
        pr_info!("Initializing.\n");

        let device = Arc::try_new(Device {
            buffer: Mutex::new(Vec::new()),
        })?;

        Ok(DeviceDriver {
            _dev: miscdev::Registration::new_pinned(fmt!("ee580"), device)?,
        })
    }
}

impl Drop for DeviceDriver {
    fn drop(&mut self) {
        pr_info!("Exit.\n");
    }
}
