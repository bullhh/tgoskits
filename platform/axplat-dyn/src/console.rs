use core::sync::atomic::{AtomicBool, Ordering};

use axplat::console::ConsoleIf;

static EARLY_READY: AtomicBool = AtomicBool::new(false);
static LATE_READY: AtomicBool = AtomicBool::new(false);

pub(crate) fn setup_early() {
    if EARLY_READY.swap(true, Ordering::AcqRel) {
        return;
    }

    match somehal::console::set_earlycon_by_cmdline() {
        Ok(()) => info!("axplat-dyn: early console initialized from bootargs"),
        Err(err) => debug!("axplat-dyn: no explicit earlycon setup from bootargs: {err}"),
    }
}

pub(crate) fn init() {
    if LATE_READY.swap(true, Ordering::AcqRel) {
        return;
    }

    info!("axplat-dyn: console late init complete");
}

struct ConsoleIfImpl;

#[impl_plat_interface]
impl ConsoleIf for ConsoleIfImpl {
    /// Writes given bytes to the console.
    fn write_bytes(bytes: &[u8]) {
        let s = core::str::from_utf8(bytes).unwrap_or_default();
        let mut remaining = s;
        while let Some(pos) = remaining.find('\n') {
            somehal::console::_write_str(&remaining[..pos]);
            somehal::console::_write_str("\r\n");
            remaining = &remaining[pos + 1..];
        }
        if !remaining.is_empty() {
            somehal::console::_write_str(remaining);
        }
    }

    /// Reads bytes from the console into the given mutable slice.
    ///
    /// Returns the number of bytes read.
    fn read_bytes(bytes: &mut [u8]) -> usize {
        let mut read_len = 0;
        while read_len < bytes.len() {
            if let Some(c) = somehal::console::read_byte() {
                bytes[read_len] = c;
            } else {
                break;
            }
            read_len += 1;
        }
        read_len
    }

    /// Returns the IRQ number for the console input interrupt.
    ///
    /// Returns `None` if input interrupt is not supported.
    #[cfg(feature = "irq")]
    fn irq_num() -> Option<usize> {
        if !LATE_READY.load(Ordering::Acquire) {
            return None;
        }
        None
    }
}
