//! Protocol: what a diagnostic message means, independent of how it travels.
//! UDS/KWP2000/OBD2 implementations build/parse messages here and delegate
//! byte transfer to a `Transport` — this trait never touches a socket or
//! serial port directly.
//!
//! UDS service subset (`0x10`, `0x19`, `0x22`, `0x3E`) implemented in
//! `uds` per `docs/research/ECU_COMMUNICATION_RESEARCH.md` section 3.
//! `0x27` (Security Access) not implemented — added only if a concrete
//! V1 DID turns out to require it.

pub mod uds;

use crate::transport::TransportError;

#[derive(Debug, PartialEq, Eq)]
pub enum ProtocolError {
    Transport(TransportError),
    UnexpectedResponse(String),
}

impl std::fmt::Display for ProtocolError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProtocolError::Transport(err) => write!(f, "transport error: {err}"),
            ProtocolError::UnexpectedResponse(reason) => write!(f, "unexpected response: {reason}"),
        }
    }
}

impl std::error::Error for ProtocolError {}

impl From<TransportError> for ProtocolError {
    fn from(err: TransportError) -> Self {
        ProtocolError::Transport(err)
    }
}

/// Diagnostic protocol built on top of a `Transport`. Implementations: `Uds`
/// (Fase 0B/2, BMW F-Series), `Kwp2000`/`Obd2` (post-Fase-0, see roadmap).
pub trait Protocol {
    /// Human-readable name for logs/telemetry (e.g. "UDS", "OBD-II").
    fn name(&self) -> &'static str;
}
