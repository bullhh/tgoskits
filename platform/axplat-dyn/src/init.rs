use axplat::init::InitIf;

use crate::console;

// 公开以确保链接
pub struct InitIfImpl;

/// 读取 TPIDR_EL1 寄存器 (AArch64 percpu 基址，percpu crate 使用 EL1)
#[cfg(target_arch = "aarch64")]
fn read_tpidr_el1() -> usize {
    let val: usize;
    unsafe {
        core::arch::asm!("mrs {}, tpidr_el1", out(reg) val);
    }
    val
}

#[cfg(not(target_arch = "aarch64"))]
fn read_tpidr_el1() -> usize {
    0
}

/// 写入 TPIDR_EL1 寄存器
#[cfg(target_arch = "aarch64")]
unsafe fn write_tpidr_el1(val: usize) {
    unsafe {
        core::arch::asm!("msr tpidr_el1, {}", in(reg) val);
    }
}

#[impl_plat_interface]
impl InitIf for InitIfImpl {
    /// Initializes the platform at the early stage for the primary core.
    ///
    /// This function should be called immediately after the kernel has booted,
    /// and performed earliest platform configuration and initialization (e.g.,
    /// early console, clocking).
    fn init_early(_cpu_id: usize, _dtb: usize) {
        console::setup_early();
        axcpu::init::init_trap();
        #[cfg(all(target_arch = "aarch64", feature = "fp-simd"))]
        {
            axcpu::asm::enable_fp();
            debug!("axplat-dyn: fp/simd enabled");
        }
        somehal::timer::enable();
        debug!("axplat-dyn: init_early complete");
    }

    /// Initializes the platform at the early stage for secondary cores.
    #[cfg(feature = "smp")]
    fn init_early_secondary(_cpu_id: usize) {
        axcpu::init::init_trap();
        #[cfg(all(target_arch = "aarch64", feature = "fp-simd"))]
        {
            axcpu::asm::enable_fp();
            debug!("axplat-dyn: secondary fp/simd enabled");
        }
        somehal::timer::enable();
        debug!("axplat-dyn: init_early_secondary complete");
    }

    /// Initializes the platform at the later stage for the primary core.
    ///
    /// This function should be called after the kernel has done part of its
    /// initialization (e.g, logging, memory management), and finalized the rest of
    /// platform configuration and initialization.
    fn init_later(_cpu_id: usize, _dtb: usize) {
        somehal::post_paging();
        somehal::timer::irq_enable();
        console::init();
        debug!("axplat-dyn: init_later complete");
    }

    /// Initializes the platform at the later stage for secondary cores.
    #[cfg(feature = "smp")]
    fn init_later_secondary(_cpu_id: usize) {
        somehal::timer::irq_enable();
        debug!("axplat-dyn: init_later_secondary complete");
    }
}
