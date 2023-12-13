// Representation of the PC System

use nvml_wrapper::{Nvml, error::NvmlError, enum_wrappers::device::{TemperatureSensor, Clock}};
use sysinfo::System;

use crate::util::{ExitMsg, ExitCode};

fn get_nvml() -> Result<Nvml, NvmlError> {
	let nvml = Nvml::init()?;

	if nvml.device_count()? == 0 {
		return Err(NvmlError::NotFound);
	}

	Ok(nvml)
}

#[derive(Debug)]
struct NvidiaGpu {
	nvml: Nvml,
	pub name: String,
	pub temp: String,
	pub gfx_clock: String,
	pub gfx_load: String,
	pub mem_clock: String,
	pub mem_load: String,
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
			temp: String::new(),
			gfx_clock: String::new(),
			gfx_load: String::new(),
			mem_clock: String::new(),
			mem_load: String::new(),
		})
	}

	pub fn update(&mut self) {
		let device = self.nvml.device_by_index(0).unwrap();
		self.name = device.name().unwrap();
		self.temp = device.temperature(TemperatureSensor::Gpu).unwrap().to_string();
		let utils = device.utilization_rates().unwrap();
		self.gfx_clock = device.clock_info(Clock::Graphics).unwrap().to_string();
		self.gfx_load = utils.gpu.to_string();
		self.mem_clock = device.clock_info(Clock::Memory).unwrap().to_string();
		self.mem_load = utils.memory.to_string();
	}
}

#[derive(Debug)]
pub struct PCSystem {
	sys: System,
	gpu: NvidiaGpu,
}
impl PCSystem {
	pub fn new() -> Result<Self, ExitMsg> {
		let gpu = NvidiaGpu::new().map_err(|why|
			ExitMsg::new(
				ExitCode::CannotInitializeGpu,
				format!("Failed to initialize NVIDIA GPU telemetry, reason: '{}'.", why)
			)
		)?;

		let mut pcs = PCSystem {
			sys: System::new_all(),
			gpu: gpu
		};
		pcs.update();

		Ok(pcs)
	}

	pub fn update(&mut self) {
		self.gpu.update();
        self.sys.refresh_cpu();
        self.sys.refresh_memory();
	}

	pub fn cpu_name(&self) -> String {
		self.sys.global_cpu_info().brand().trim().to_owned()
	}

	pub fn gpu_name(&self) -> String {
		self.gpu.name.clone()
	}
	
	pub fn memory_total(&self) -> String {
		// FIXME: the postfix should be removed later
		format!("{:.1}GiB", (self.sys.total_memory() as f64) / ((1 << 30) as f64))
	}

	pub fn cpu_freq(&self) -> String {
		format!("{:.2}", (self.sys.global_cpu_info().frequency() as f64) / (1000 as f64))
	}

	pub fn cpu_temp(&self) -> String {
		"N/A".to_owned()
	}

	pub fn cpu_load(&self) -> String {
		format!("{:.1}", self.sys.global_cpu_info().cpu_usage())
	}

	pub fn memory_used(&self) -> String {
		format!("{:.1}", (self.sys.used_memory() as f64) / ((1 << 30) as f64))
	}

	pub fn gpu_temp(&self) -> String {
		self.gpu.temp.clone()
	}

	pub fn gpu_core_clock(&self) -> String {
		self.gpu.gfx_clock.clone()
	}

	pub fn gpu_core_load(&self) -> String {
		self.gpu.gfx_load.clone()
	}

	pub fn gpu_memory_clock(&self) -> String {
		self.gpu.mem_clock.clone()
	}

	pub fn gpu_memory_load(&self) -> String {
		self.gpu.mem_load.clone()
	}
}