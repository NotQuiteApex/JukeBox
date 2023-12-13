// Representation of the PC System

use nvml_wrapper::{
    enum_wrappers::device::{Clock, TemperatureSensor},
    error::NvmlError,
    Nvml,
};
use sysinfo::{Component, CpuExt, System, SystemExt};

use crate::util::{ExitCode, ExitMsg};

trait SysGpu {
    fn update(&mut self);
    fn name(&self) -> String;
    fn temperature(&self) -> String;
    fn core_clock(&self) -> String;
    fn core_load(&self) -> String;
    fn vram_clock(&self) -> String;
    fn vram_load(&self) -> String;
}

struct NoneGpu {}
impl SysGpu for NoneGpu {
    fn update(&mut self) {}

    fn name(&self) -> String {
        "N/A".to_owned()
    }

    fn temperature(&self) -> String {
        "N/A".to_owned()
    }

    fn core_clock(&self) -> String {
        "N/A".to_owned()
    }

    fn core_load(&self) -> String {
        "N/A".to_owned()
    }

    fn vram_clock(&self) -> String {
        "N/A".to_owned()
    }

    fn vram_load(&self) -> String {
        "N/A".to_owned()
    }
}

struct NvidiaGpu {
    nvml: Nvml,
    name: String,
    temperature: String,
    core_clock: String,
    core_load: String,
    vram_clock: String,
    vram_load: String,
}
impl NvidiaGpu {
    pub fn new() -> Result<Self, NvmlError> {
        Ok(NvidiaGpu {
            nvml: {
                let nvml = Nvml::init()?;
                if nvml.device_count()? == 0 {
                    return Err(NvmlError::NotFound);
                }
                nvml
            },
            name: String::new(),
            temperature: String::new(),
            core_clock: String::new(),
            core_load: String::new(),
            vram_clock: String::new(),
            vram_load: String::new(),
        })
    }
}
impl SysGpu for NvidiaGpu {
    fn update(&mut self) {
        let device = self.nvml.device_by_index(0).unwrap();
        self.name = device.name().unwrap();
        self.temperature = device
            .temperature(TemperatureSensor::Gpu)
            .unwrap()
            .to_string();
        let utils = device.utilization_rates().unwrap();
        self.core_clock = device.clock_info(Clock::Graphics).unwrap().to_string();
        self.core_load = utils.gpu.to_string();
        self.vram_clock = device.clock_info(Clock::Memory).unwrap().to_string();
        self.vram_load = utils.memory.to_string();
    }

    fn name(&self) -> String {
        self.name.clone()
    }

    fn temperature(&self) -> String {
        self.temperature.clone()
    }

    fn core_clock(&self) -> String {
        self.core_clock.clone()
    }

    fn core_load(&self) -> String {
        self.core_load.clone()
    }

    fn vram_clock(&self) -> String {
        self.vram_clock.clone()
    }

    fn vram_load(&self) -> String {
        self.vram_load.clone()
    }
}

pub struct PCSystem {
    sys: System,
    gpu: Box<dyn SysGpu>,
}
impl PCSystem {
    pub fn new() -> Result<Self, ExitMsg> {
        let gpu: Box<dyn SysGpu> = match NvidiaGpu::new() {
            Ok(g) => Box::new(g),
            Err(_) => {
                log::warn!("Could not initialize NVIDIA GPU.");
                Box::new(NoneGpu {})
            }
        };

        let mut pcs = PCSystem {
            sys: System::new_all(),
            gpu: gpu,
        };
        pcs.update();

        Ok(pcs)
    }

    pub fn update(&mut self) {
        self.gpu.update();
        self.sys.refresh_cpu();
        self.sys.refresh_memory();
        self.sys.refresh_components();
    }

    pub fn cpu_name(&self) -> String {
        self.sys.global_cpu_info().brand().trim().to_owned()
    }

    pub fn gpu_name(&self) -> String {
        self.gpu.name()
    }

    pub fn memory_total(&self) -> String {
        // FIXME: the postfix should be removed later
        format!(
            "{:.1}GiB",
            (self.sys.total_memory() as f64) / ((1 << 30) as f64)
        )
    }

    pub fn cpu_freq(&self) -> String {
        println!("TEST {:?}", self.sys.global_cpu_info());
        format!(
            "{:.2}",
            (self.sys.global_cpu_info().frequency() as f64) / (1000 as f64)
        )
    }

    pub fn cpu_temp(&self) -> String {
        "(N/A)".to_owned()
    }

    pub fn cpu_load(&self) -> String {
        format!("{:.1}", self.sys.global_cpu_info().cpu_usage())
    }

    pub fn memory_used(&self) -> String {
        format!(
            "{:.1}",
            (self.sys.used_memory() as f64) / ((1 << 30) as f64)
        )
    }

    pub fn gpu_temp(&self) -> String {
        self.gpu.temperature()
    }

    pub fn gpu_core_clock(&self) -> String {
        self.gpu.core_clock()
    }

    pub fn gpu_core_load(&self) -> String {
        self.gpu.core_load()
    }

    pub fn gpu_memory_clock(&self) -> String {
        self.gpu.vram_clock()
    }

    pub fn gpu_memory_load(&self) -> String {
        self.gpu.vram_load()
    }

    pub fn sensors(&self) -> &[Component] {
        self.sys.components()
    }
}
