//! In-memory state machine that answers UDS requests like a real ECU would:
//! session control, configurable DIDs/DTCs, negative responses, simulated
//! latency. Sits behind `MockTransport` so `protocol::uds::Uds` can be built
//! and tested end to end without the vehicle. See
//! docs/research/ECU_COMMUNICATION_RESEARCH.md section 6.
//!
//! Only the V1 service subset is implemented (session control, ReadDataByIdentifier,
//! ReadDTCInformation by status mask, TesterPresent) — see that doc's section 3
//! for what's deliberately out of scope and why.

use std::collections::HashMap;
use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosticSession {
    Default,
    Extended,
}

/// UDS Negative Response Codes actually used by the mock (ISO 14229-1).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Nrc {
    ServiceNotSupported = 0x11,
    SubFunctionNotSupported = 0x12,
    RequestOutOfRange = 0x31,
}

pub struct MockEcu {
    session: DiagnosticSession,
    dids: HashMap<u16, Vec<u8>>,
    dtcs: Vec<(u32, u8)>,
    latency: Duration,
}

impl MockEcu {
    pub fn new() -> Self {
        let mut dids = HashMap::new();
        // 0xF190: VIN, DID estándar de UDS (ISO 14229). Valor de pega, 17
        // caracteres como un VIN real, para poder probar parsing de longitud.
        dids.insert(0xF190u16, b"WBAF12345LC000001".to_vec());
        Self {
            session: DiagnosticSession::Default,
            dids,
            dtcs: Vec::new(),
            latency: Duration::ZERO,
        }
    }

    pub fn with_dtc(mut self, code: u32, status: u8) -> Self {
        self.dtcs.push((code, status));
        self
    }

    pub fn with_did(mut self, did: u16, value: Vec<u8>) -> Self {
        self.dids.insert(did, value);
        self
    }

    pub fn with_latency(mut self, latency: Duration) -> Self {
        self.latency = latency;
        self
    }

    pub fn simulated_latency(&self) -> Duration {
        self.latency
    }

    pub fn session(&self) -> DiagnosticSession {
        self.session
    }

    /// Processes one raw UDS request and returns the raw response — positive
    /// or negative — exactly as bytes would come back over the wire.
    pub fn handle_request(&mut self, request: &[u8]) -> Vec<u8> {
        let Some(&sid) = request.first() else {
            return negative_response(0x00, Nrc::ServiceNotSupported);
        };

        match sid {
            0x10 => self.diagnostic_session_control(request),
            0x22 => self.read_data_by_identifier(request),
            0x19 => self.read_dtc_information(request),
            0x3E => self.tester_present(request),
            _ => negative_response(sid, Nrc::ServiceNotSupported),
        }
    }

    fn diagnostic_session_control(&mut self, request: &[u8]) -> Vec<u8> {
        let Some(&sub_function) = request.get(1) else {
            return negative_response(0x10, Nrc::SubFunctionNotSupported);
        };

        self.session = match sub_function {
            0x01 => DiagnosticSession::Default,
            0x03 => DiagnosticSession::Extended,
            _ => return negative_response(0x10, Nrc::SubFunctionNotSupported),
        };

        // Positiva: 0x50, sub-función, P2Server (2 bytes), P2*Server (2 bytes).
        // Valores conservadores de la literatura ISO 14229 — a recalibrar
        // contra el gateway real en Fase 0B (docs/research sección 7).
        vec![0x50, sub_function, 0x00, 0x32, 0x01, 0xF4]
    }

    fn read_data_by_identifier(&self, request: &[u8]) -> Vec<u8> {
        if request.len() < 3 {
            return negative_response(0x22, Nrc::RequestOutOfRange);
        }
        let did = u16::from_be_bytes([request[1], request[2]]);

        match self.dids.get(&did) {
            Some(value) => {
                let mut response = vec![0x62, request[1], request[2]];
                response.extend_from_slice(value);
                response
            }
            None => negative_response(0x22, Nrc::RequestOutOfRange),
        }
    }

    fn read_dtc_information(&self, request: &[u8]) -> Vec<u8> {
        let Some(&sub_function) = request.get(1) else {
            return negative_response(0x19, Nrc::SubFunctionNotSupported);
        };

        // Solo reportDTCByStatusMask (0x02): es lo único que necesita el
        // módulo Diagnostics en V1 (docs/research sección 3).
        if sub_function != 0x02 {
            return negative_response(0x19, Nrc::SubFunctionNotSupported);
        }

        let mut response = vec![0x59, sub_function, 0xFF]; // statusAvailabilityMask: todo disponible
        for (code, status) in &self.dtcs {
            let code_bytes = code.to_be_bytes();
            response.extend_from_slice(&code_bytes[1..]); // 3 bytes de DTC
            response.push(*status);
        }
        response
    }

    fn tester_present(&self, request: &[u8]) -> Vec<u8> {
        match request.get(1) {
            Some(&0x00) => vec![0x7E, 0x00],
            _ => negative_response(0x3E, Nrc::SubFunctionNotSupported),
        }
    }
}

impl Default for MockEcu {
    fn default() -> Self {
        Self::new()
    }
}

fn negative_response(sid: u8, nrc: Nrc) -> Vec<u8> {
    vec![0x7F, sid, nrc as u8]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn session_control_switches_session_and_acks() {
        let mut ecu = MockEcu::new();
        let response = ecu.handle_request(&[0x10, 0x03]);
        assert_eq!(response[0], 0x50);
        assert_eq!(response[1], 0x03);
        assert_eq!(ecu.session(), DiagnosticSession::Extended);
    }

    #[test]
    fn session_control_rejects_unknown_subfunction() {
        let mut ecu = MockEcu::new();
        let response = ecu.handle_request(&[0x10, 0xEE]);
        assert_eq!(response, vec![0x7F, 0x10, Nrc::SubFunctionNotSupported as u8]);
    }

    #[test]
    fn read_data_by_identifier_returns_configured_vin() {
        let mut ecu = MockEcu::new();
        let response = ecu.handle_request(&[0x22, 0xF1, 0x90]);
        assert_eq!(&response[..3], &[0x62, 0xF1, 0x90]);
        assert_eq!(&response[3..], b"WBAF12345LC000001");
    }

    #[test]
    fn read_data_by_identifier_unknown_did_is_negative() {
        let mut ecu = MockEcu::new();
        let response = ecu.handle_request(&[0x22, 0x00, 0x00]);
        assert_eq!(response, vec![0x7F, 0x22, Nrc::RequestOutOfRange as u8]);
    }

    #[test]
    fn read_dtc_information_lists_configured_dtcs() {
        let mut ecu = MockEcu::new().with_dtc(0x001234, 0x08);
        let response = ecu.handle_request(&[0x19, 0x02]);
        assert_eq!(response[0], 0x59);
        assert_eq!(&response[3..6], &[0x00, 0x12, 0x34]);
        assert_eq!(response[6], 0x08);
    }

    #[test]
    fn read_dtc_information_rejects_unsupported_subfunction() {
        let mut ecu = MockEcu::new();
        let response = ecu.handle_request(&[0x19, 0x04]);
        assert_eq!(response, vec![0x7F, 0x19, Nrc::SubFunctionNotSupported as u8]);
    }

    #[test]
    fn tester_present_acks() {
        let mut ecu = MockEcu::new();
        assert_eq!(ecu.handle_request(&[0x3E, 0x00]), vec![0x7E, 0x00]);
    }

    #[test]
    fn unknown_service_is_negative() {
        let mut ecu = MockEcu::new();
        let response = ecu.handle_request(&[0x99]);
        assert_eq!(response, vec![0x7F, 0x99, Nrc::ServiceNotSupported as u8]);
    }
}
