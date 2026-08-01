//! In-memory transport for development and testing without hardware.
//! See docs/research/ECU_COMMUNICATION_RESEARCH.md section 6 (Mock ECU
//! strategy) and PROJECT_PLAN.md's norm that no phase depends exclusively
//! on the real vehicle.
//!
//! Fase 0 ships the transport shell only, always timing out — there is no
//! `MockEcu` behind it yet. The state machine that answers like a real ECU
//! (sessions, DIDs, DTCs, negative responses, simulated latency) is built in
//! Fase 0B, per its criterio de cierre in PROJECT_PLAN.md.

use super::{Transport, TransportError};
use std::time::Duration;

#[derive(Default)]
pub struct MockTransport {
    connected: bool,
}

impl MockTransport {
    pub fn new() -> Self {
        Self { connected: false }
    }
}

impl Transport for MockTransport {
    fn connect(&mut self) -> Result<(), TransportError> {
        self.connected = true;
        Ok(())
    }

    fn disconnect(&mut self) -> Result<(), TransportError> {
        self.connected = false;
        Ok(())
    }

    fn is_connected(&self) -> bool {
        self.connected
    }

    fn send_receive(&mut self, _data: &[u8], _timeout: Duration) -> Result<Vec<u8>, TransportError> {
        if !self.connected {
            return Err(TransportError::NotConnected);
        }
        // No MockEcu behind this yet (Fase 0B) — nothing to answer with.
        Err(TransportError::Timeout)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_send_before_connect() {
        let mut transport = MockTransport::new();
        let result = transport.send_receive(&[0x10, 0x01], Duration::from_millis(50));
        assert_eq!(result, Err(TransportError::NotConnected));
    }

    #[test]
    fn connect_then_disconnect_tracks_state() {
        let mut transport = MockTransport::new();
        assert!(!transport.is_connected());

        transport.connect().unwrap();
        assert!(transport.is_connected());

        transport.disconnect().unwrap();
        assert!(!transport.is_connected());
    }

    #[test]
    fn connected_with_no_ecu_behind_it_times_out() {
        let mut transport = MockTransport::new();
        transport.connect().unwrap();
        let result = transport.send_receive(&[0x10, 0x01], Duration::from_millis(50));
        assert_eq!(result, Err(TransportError::Timeout));
    }
}
