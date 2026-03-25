extern crate alloc;

use axklib::mem::iomap;
use rdif_clk::{ClockId, Interface};
use rdrive::{DriverGeneric, KError, PlatformDevice, module_driver, probe::OnProbeError, register::FdtInfo};
use rk3568_clk::CRU;
use rk3568_clk::cru_clksel_con28_bits::*;

pub struct ClkDriver(CRU);

pub const EMMC_CLK_ID: usize = 0x7c;

impl ClkDriver {
    pub fn new(cru_address: u64) -> Self {
        Self(CRU::new(cru_address as *mut _))
    }
}

impl DriverGeneric for ClkDriver {
    fn name(&self) -> &str {
        "rk3568-clk"
    }
}

impl Interface for ClkDriver {
    fn perper_enable(&mut self) {}

    fn get_rate(&self, id: ClockId) -> Result<u64, KError> {
        let rate = match usize::from(id) {
            EMMC_CLK_ID => {
                let con = self.0.cru_clksel_get_cclk_emmc();
                con >> CRU_CLKSEL_CCLK_EMMC_POS
            }
            _ => return Err(KError::InvalidArg { name: "clock_id" }),
        };
        Ok(rate as u64)
    }

    fn set_rate(&mut self, id: ClockId, rate: u64) -> Result<(), KError> {
        match usize::from(id) {
            EMMC_CLK_ID => {
                let rate = rate as u32;
                let src_clk = match rate {
                    24_000_000 => CRU_CLKSEL_CCLK_EMMC_XIN_SOC0_MUX,
                    50_000_000 | 52_000_000 => CRU_CLKSEL_CCLK_EMMC_CPL_DIV_50M,
                    100_000_000 => CRU_CLKSEL_CCLK_EMMC_CPL_DIV_100M,
                    150_000_000 => CRU_CLKSEL_CCLK_EMMC_GPL_DIV_150M,
                    200_000_000 => CRU_CLKSEL_CCLK_EMMC_GPL_DIV_200M,
                    375_000 | 400_000 => CRU_CLKSEL_CCLK_EMMC_SOC0_375K,
                    _ => return Err(KError::InvalidArg { name: "rate" }),
                };
                self.0.cru_clksel_set_cclk_emmc(src_clk);
                Ok(())
            }
            _ => Err(KError::InvalidArg { name: "clock_id" }),
        }
    }
}

module_driver!(
    name: "Rockchip CRU",
    level: ProbeLevel::PostKernel,
    priority: ProbePriority::CLK,
    probe_kinds: &[
        ProbeKind::Fdt {
            compatibles: &["rockchip,rk3568-cru"],
            on_probe: probe
        }
    ],
);

fn probe(info: FdtInfo<'_>, plat_dev: PlatformDevice) -> Result<(), OnProbeError> {
    let cru_reg = info
        .node
        .regs()
        .into_iter()
        .next()
        .ok_or(OnProbeError::other(alloc::format!(
            "[{}] has no reg",
            info.node.name()
        )))?;

    let cru_reg_base = iomap(
        (cru_reg.address as usize).into(),
        cru_reg.size.unwrap_or(0x1000) as usize,
    )
    .map_err(|e| OnProbeError::other(alloc::format!("failed to iomap CRU: {e:?}")))?;

    let clk = rdif_clk::Clk::new(ClkDriver::new(cru_reg_base.as_ptr() as u64));
    plat_dev.register(clk);
    Ok(())
}
