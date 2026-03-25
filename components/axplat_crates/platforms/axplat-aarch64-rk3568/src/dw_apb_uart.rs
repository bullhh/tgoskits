use axplat::console::ConsoleIf;
use axplat::mem::{PhysAddr, pa};
use dw_apb_uart::DW8250;
use kspin::SpinNoIrq;

use crate::mem::phys_to_virt;

const UART_BASE: PhysAddr = pa!(crate::config::devices::UART_PADDR);

static UART: SpinNoIrq<DW8250> = SpinNoIrq::new(DW8250::new(phys_to_virt(UART_BASE).as_usize()));

pub fn init_early() {
    UART.lock().init();
}

#[cfg(feature = "irq")]
pub fn init_irq() {
    UART.lock().set_ier(true);
}

fn getchar() -> Option<u8> {
    UART.lock().getchar()
}

struct ConsoleIfImpl;

#[impl_plat_interface]
impl ConsoleIf for ConsoleIfImpl {
    fn write_bytes(bytes: &[u8]) {
        let mut uart = UART.lock();
        for &c in bytes {
            match c {
                b'\r' | b'\n' => {
                    uart.putchar(b'\r');
                    uart.putchar(b'\n');
                }
                _ => uart.putchar(c),
            }
        }
    }

    fn read_bytes(bytes: &mut [u8]) -> usize {
        let mut read_len = 0;
        while read_len < bytes.len() {
            let Some(c) = getchar() else {
                break;
            };
            bytes[read_len] = c;
            read_len += 1;
        }
        read_len
    }

    #[cfg(feature = "irq")]
    fn irq_num() -> Option<usize> {
        Some(crate::config::devices::UART_IRQ)
    }
}
