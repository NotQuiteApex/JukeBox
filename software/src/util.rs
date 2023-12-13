// Utility functions and types

// Self-describing exit codes.
// Each exit point of the program should be using a very clear exit code, along
// with a message sent to stderr for more details. Certain codes may be reserved
// or not used, as indicated by the leading underscore in its name.
#[derive(Debug, Clone)]
pub enum ExitCode {
    // Special codes
    _CleanExit,
    Interrupted,
    _ReservedByClap,
    StderrLogger,
    CannotRegisterSignalHandler,

    CannotInitializeGpu,

    GenericError,

    SerialReadBadData,
    SerialReadTimeout,

    SerialStageGreetHost,
    SerialStageGreetDevice,
    SerialStageLinkConfirmHost,
    SerialStageLinkConfirmDevice,

    SerialTransmitComputerPartInitSend,
    SerialTransmitComputerPartInitAck,
    SerialTransmitHeartbeatSend,
    SerialTransmitHeartbeatAck,
    SerialTransmitComputerPartStatSend,
    SerialTransmitComputerPartStatAck,
}

#[derive(Debug, Clone)]
pub struct ExitMsg {
    pub code: ExitCode,
    pub msg: String,
}
impl ExitMsg {
    pub fn new(code: ExitCode, msg: String) -> Self {
        log::warn!(
            "ExitMsg - {:?} ({}) - {}",
            code,
            code.clone() as i32,
            msg.as_str()
        );
        ExitMsg {
            code: code,
            msg: msg,
        }
    }
}
impl std::fmt::Display for ExitMsg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} Exit code: {:?} ({})",
            self.msg.as_str(),
            self.code,
            self.code.clone() as i32
        )
    }
}
impl std::error::Error for ExitMsg {}
