//! In-memory transport for development and testing without hardware.
//! See docs/research/ECU_COMMUNICATION_RESEARCH.md section 6 (Mock ECU
//! strategy) and PROJECT_PLAN.md's norm that no phase depends exclusively
//! on the real vehicle.
//!
//! Without a `MockEcu` attached, every request times out — that's still a
//! valid, useful configuration (tests the "nothing answers" path). With one
//! attached (`MockTransport::with_ecu`), requests get routed to it and
//! answered like a real ECU would.

use super::mock_ecu::MockEcu;
use super::{Transport, TransportError};
use std::time::Duration;

pub struct MockTransport {
    connected: bool,
    ecu: Option<MockEcu>,
}

impl MockTransport {
    pub fn new() -> Self {
        Self { connected: false, ecu: None }
    }

    pub fn with_ecu(ecu: MockEcu) -> Self {
        Self { connected: false, ecu: Some(ecu) }
    }
}

impl Default for MockTransport {
    fn default() -> Self {
        Self::new()
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

    fn send_receive(&mut self, data: &[u8], timeout: Duration) -> Result<Vec<u8>, TransportError> {
        if !self.connected {
            return Err(TransportError::NotConnected);
        }

        match &mut self.ecu {
            Some(ecu) => {
                if ecu.simulated_latency() > timeout {
                    return Err(TransportError::Timeout);
                }
                Ok(ecu.handle_request(data))
            }
            // Sin MockEcu configurado: comportamiento de Fase 0, no hay
            // nada detrás que responda.
            None => Err(TransportError::Timeout),
        }
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

    #[test]
    fn connected_with_ecu_routes_request_and_answers() {
        let mut transport = MockTransport::with_ecu(MockEcu::new());
        transport.connect().unwrap();
        let response = transport
            .send_receive(&[0x22, 0xF1, 0x90], Duration::from_millis(50))
            .unwrap();
        assert_eq!(&response[..3], &[0x62, 0xF1, 0x90]);
    }

    #[test]
    fn ecu_latency_above_timeout_reports_transport_timeout() {
        let ecu = MockEcu::new().with_latency(Duration::from_millis(200));
        let mut transport = MockTransport::with_ecu(ecu);
        transport.connect().unwrap();
        let result = transport.send_receive(&[0x22, 0xF1, 0x90], Duration::from_millis(50));
        assert_eq!(result, Err(TransportError::Timeout));
    }
}
