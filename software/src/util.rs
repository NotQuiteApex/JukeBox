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

    _CannotInitializeGpu,

    GenericError,

    SerialSendMessageError,
    SerialSendFlushError,
    SerialExpectRecieveError,
    SerialExpectMatchError,

    SerialReadBadData,
    SerialReadTimeout,
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

pub fn is_version_string_newer(new: &str) -> bool {
    let old = env!("CARGO_PKG_VERSION");
    let new = if new.starts_with('v') { &new[1..] } else { new };

    let old: Vec<_> = old.split('.').collect();
    let new: Vec<_> = new.split('.').collect();

    if old.len() != 3 {
        panic!("version not 3 strings!");
    }

    if new.len() != 3 {
        log::warn!("new version `{:?}` not 3 strings", new)
    }

    for (old, new) in std::iter::zip(old, new) {
        let o = old.parse::<usize>();
        let n = new.parse::<usize>();

        if o.is_err() {
            panic!("failed to parse old `{}` as number!", old);
            // return false;
        }

        if n.is_err() {
            log::warn!("failed to parse new `{:?}` as a number", new);
            return false;
        }

        let o = o.unwrap();
        let n = n.unwrap();

        if o > n {
            return false;
        } else if o == n {
            continue;
        } else if o < n {
            return true;
        }
    }

    false
}
