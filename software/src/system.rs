// Representation of the PC System

use once_cell::sync::OnceCell;

use nvml_wrapper::{
    enum_wrappers::device::{Clock, TemperatureSensor},
    error::NvmlError,
    Device, Nvml,
};
use sysinfo::{Components, CpuRefreshKind, MemoryRefreshKind, System};

use crate::util::ExitMsg;

static NVML: OnceCell<Nvml> = OnceCell::new();

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
    device: Device<'static>,
    name: String,
    temperature: u32,
    core_clock: u32,
    core_load: u32,
    vram_clock: u32,
    vram_load: u32,
}
impl NvidiaGpu {
    pub fn new() -> Result<Self, NvmlError> {
        let nvml = NVML.get_or_try_init(|| Nvml::init())?;
        if nvml.device_count()? == 0 {
            return Err(NvmlError::NotFound);
        }
        // TODO: handle GPU's being hot pluggable
        let device = nvml.device_by_index(0).unwrap();

        Ok(NvidiaGpu {
            device: device,
            name: String::new(),
            temperature: 0,
            core_clock: 0,
            core_load: 0,
            vram_clock: 0,
            vram_load: 0,
        })
    }
}
impl SysGpu for NvidiaGpu {
    fn update(&mut self) {
        self.name = self.device.name().unwrap();
        self.temperature = self.device.temperature(TemperatureSensor::Gpu).unwrap();
        let utils = self.device.utilization_rates().unwrap();
        self.core_clock = self.device.clock_info(Clock::Graphics).unwrap();
        self.core_load = utils.gpu;
        self.vram_clock = self.device.clock_info(Clock::Memory).unwrap();
        self.vram_load = utils.memory;
    }

    fn name(&self) -> String {
        self.name.clone()
    }

    fn temperature(&self) -> String {
        self.temperature.to_string()
    }

    fn core_clock(&self) -> String {
        self.core_clock.to_string()
    }

    fn core_load(&self) -> String {
        self.core_load.to_string()
    }

    fn vram_clock(&self) -> String {
        self.vram_clock.to_string()
    }

    fn vram_load(&self) -> String {
        self.vram_load.to_string()
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
        // self.sys.refresh_all();

        // self.sys.refresh_cpu(); // <- This doesn't refresh everything on all platforms.
        self.sys.refresh_cpu_specifics(CpuRefreshKind::everything());
        self.sys
            .refresh_memory_specifics(MemoryRefreshKind::new().without_swap());
        // self.sys.refresh_components();
    }

    pub fn cpu_name(&self) -> String {
        self.sys.cpus().get(0).unwrap().brand().trim().to_owned()
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
        // See https://github.com/GuillaumeGomez/sysinfo/issues/543 on why this doesn't use global_cpu_info
        let cpus = self.sys.cpus();
        let mut freq = 0u64;
        for cpu in cpus {
            freq += cpu.frequency();
        }

        format!("{:.2}", ((freq / cpus.len() as u64) as f64) / (1000 as f64))
    }

    pub fn cpu_temp(&self) -> String {
        "N/A".to_owned()
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

    pub fn sensors(&self) -> Components {
        Components::new_with_refreshed_list()
    }

    pub fn probe_report(&self) {
        log::info!("PROBE START...");
        // log::info!("");

        log::info!("CPU Name: ---- '{}'", self.cpu_name());
        log::info!("GPU Name: ---- '{}'", self.gpu_name());
        log::info!("Total Memory: - {} GiB", self.memory_total());
        // log::info!("CPU Vendor: -- '{}'", self.sys.global_cpu_info().vendor_id());
        // log::info!("");
        log::info!("CPU Freq: ------- {} GHz", self.cpu_freq());
        log::info!("CPU Temp: ------- {} * C", self.cpu_temp());
        log::info!("CPU Load: ------- {} %", self.cpu_load());
        log::info!("Memory Used: ---- {} GiB", self.memory_used());
        log::info!("GPU Temp: ------- {} * C", self.gpu_temp());
        log::info!("GPU Core Clock: - {} MHz", self.gpu_core_clock());
        log::info!("GPU Core Load: -- {} %", self.gpu_core_load());
        log::info!("GPU VRAM Clock: - {} MHz", self.gpu_memory_clock());
        log::info!("GPU VRAM Load: -- {} %", self.gpu_memory_load());
        // log::info!("");

        log::info!("Sensors:");
        for (i, c) in self.sensors().iter().enumerate() {
            log::info!("\t{}. {:?}", i + 1, c)
        }

        // log::info!("");
        log::info!("PROBE ENDED!!!");
    }
}
