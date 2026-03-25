use axplat::init::InitIf;
use axplat::mem::{pa, phys_to_virt};

use crate::config::devices::{GICC_PADDR, GICD_PADDR, TIMER_IRQ};
use crate::config::plat::PSCI_METHOD;

struct InitIfImpl;

#[impl_plat_interface]
impl InitIf for InitIfImpl {
    fn init_early(_cpu_id: usize, _dtb: usize) {
        axcpu::init::init_trap();
        crate::dw_apb_uart::init_early();
        axplat_aarch64_peripherals::psci::init(PSCI_METHOD);
        axplat_aarch64_peripherals::generic_timer::init_early();
    }

    #[cfg(feature = "smp")]
    fn init_early_secondary(_cpu_id: usize) {
        axcpu::init::init_trap();
    }

    fn init_later(_cpu_id: usize, _dtb: usize) {
        #[cfg(feature = "irq")]
        {
            axplat_aarch64_peripherals::gic::init_gic(
                phys_to_virt(pa!(GICD_PADDR)),
                phys_to_virt(pa!(GICC_PADDR)),
            );
            axplat_aarch64_peripherals::gic::init_gicc();
            axplat_aarch64_peripherals::generic_timer::enable_irqs(TIMER_IRQ);
            crate::dw_apb_uart::init_irq();
        }
    }

    #[cfg(feature = "smp")]
    fn init_later_secondary(_cpu_id: usize) {
        #[cfg(feature = "irq")]
        {
            axplat_aarch64_peripherals::gic::init_gicc();
            axplat_aarch64_peripherals::generic_timer::enable_irqs(TIMER_IRQ);
        }
    }
}
