// An application for interfacing with a JukeBox over serial.

// #![windows_subsystem = "windows"] // disables console spawning for release build

mod gui;
mod serial;
mod splash;
mod util;
mod reaction;

fn deffered_main() -> Result<(), util::ExitMsg> {
    env_logger::init();

    gui::basic_gui();

    Ok(())
}

fn main() {
    std::process::exit(deffered_main().map_or_else(
        |err| {
            log::error!("{}", err);
            err.code as i32
        },
        |_| 0,
    ))
}
