//! Transport: how raw bytes reach the vehicle. `Protocol` implementations
//! (UDS, KWP2000, OBD2) build frames and hand them to a `Transport` — this
//! trait knows nothing about diagnostic protocols, only byte transfer.
//! See PROJECT_PLAN.md 3.2 and docs/research/ECU_COMMUNICATION_RESEARCH.md.

pub mod mock;
pub mod mock_ecu;

use std::time::Duration;

#[derive(Debug, PartialEq, Eq)]
pub enum TransportError {
    ConnectionFailed(String),
    Timeout,
    NotConnected,
    Io(String),
}

impl std::fmt::Display for TransportError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TransportError::ConnectionFailed(reason) => write!(f, "connection failed: {reason}"),
            TransportError::Timeout => write!(f, "timeout waiting for response"),
            TransportError::NotConnected => write!(f, "not connected"),
            TransportError::Io(reason) => write!(f, "I/O error: {reason}"),
        }
    }
}

impl std::error::Error for TransportError {}

/// Raw byte-level transport to the vehicle. Implementations: `EnetTransport`
/// (Fase 1, BMW F-Series over TCP/DoIP-like), `MockTransport` (this module,
/// for development/testing without hardware). `KDcanTransport`/`Elm327Transport`
/// are post-Fase-0 additions — this trait is the extension point for them.
pub trait Transport: Send {
    fn connect(&mut self) -> Result<(), TransportError>;
    fn disconnect(&mut self) -> Result<(), TransportError>;
    fn is_connected(&self) -> bool;

    /// Sends raw bytes and waits up to `timeout` for a raw response.
    /// Network timeout vs. protocol-level "response pending" are distinct
    /// concerns — this only reports the transport-level outcome; a `Protocol`
    /// decides what a `0x78` NRC means (see docs/research section 7).
    fn send_receive(&mut self, data: &[u8], timeout: Duration) -> Result<Vec<u8>, TransportError>;
}
