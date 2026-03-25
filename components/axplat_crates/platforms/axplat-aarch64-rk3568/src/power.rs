use axplat::power::PowerIf;

struct PowerImpl;

#[impl_plat_interface]
impl PowerIf for PowerImpl {
    #[cfg(feature = "smp")]
    fn cpu_boot(cpu_id: usize, stack_top_paddr: usize) {
        use axplat::mem::{va, virt_to_phys};

        let entry = virt_to_phys(va!(crate::boot::_start_secondary as *const () as usize));
        axplat_aarch64_peripherals::psci::cpu_on(
            crate::config::plat::CPU_ID_LIST[cpu_id],
            entry.as_usize(),
            stack_top_paddr,
        );
    }

    fn system_off() -> ! {
        info!("Shutting down...");
        axplat_aarch64_peripherals::psci::system_off()
    }

    fn cpu_num() -> usize {
        crate::config::plat::MAX_CPU_NUM
    }
}
