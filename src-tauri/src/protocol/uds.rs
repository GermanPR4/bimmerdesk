//! UDS (ISO 14229) — el único protocolo de diagnóstico que necesita V1
//! (BMW F-Series). Construye/interpreta el subconjunto de servicios definido
//! en docs/research/ECU_COMMUNICATION_RESEARCH.md sección 3: `0x10`
//! (DiagnosticSessionControl), `0x22` (ReadDataByIdentifier), `0x19`
//! (ReadDTCInformation), `0x3E` (TesterPresent). `0x27` (Security Access) no
//! está implementado todavía — se añade solo si algún DID concreto de V1 lo
//! exige, ver esa misma sección.
//!
//! `Uds<T>` es una struct concreta con su propia API, no fuerza todo a pasar
//! por el trait `Protocol` (que se deja mínimo a propósito): KWP2000/OBD2
//! no existen aún en este código, así que no hay nada real que informe una
//! interfaz compartida — añadirla ahora sería adivinar una abstracción
//! (PROJECT_PRINCIPLES.md, Principio 1).

use crate::protocol::{Protocol, ProtocolError};
use crate::transport::Transport;
use std::time::Duration;

/// Timeout de una petición/respuesta UDS. Valor provisional de la
/// literatura ISO 14229 (P2Server ~50ms) con margen — a recalibrar contra
/// el gateway BMW real en Fase 0B (docs/research sección 7).
const DEFAULT_TIMEOUT: Duration = Duration::from_millis(500);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosticSession {
    Default,
    Extended,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Dtc {
    pub code: u32,
    pub status: u8,
}

pub struct Uds<T: Transport> {
    transport: T,
}

impl<T: Transport> Uds<T> {
    pub fn new(transport: T) -> Self {
        Self { transport }
    }

    pub fn diagnostic_session_control(&mut self, session: DiagnosticSession) -> Result<(), ProtocolError> {
        let sub_function = match session {
            DiagnosticSession::Default => 0x01,
            DiagnosticSession::Extended => 0x03,
        };
        let response = self.request(&[0x10, sub_function])?;
        expect_positive(&response, 0x50)?;
        Ok(())
    }

    pub fn read_data_by_identifier(&mut self, did: u16) -> Result<Vec<u8>, ProtocolError> {
        let [hi, lo] = did.to_be_bytes();
        let response = self.request(&[0x22, hi, lo])?;
        expect_positive(&response, 0x62)?;
        Ok(response[3..].to_vec())
    }

    pub fn read_dtc_information(&mut self) -> Result<Vec<Dtc>, ProtocolError> {
        let response = self.request(&[0x19, 0x02])?;
        expect_positive(&response, 0x59)?;

        let mut dtcs = Vec::new();
        for chunk in response[3..].chunks_exact(4) {
            let code = u32::from_be_bytes([0, chunk[0], chunk[1], chunk[2]]);
            dtcs.push(Dtc { code, status: chunk[3] });
        }
        Ok(dtcs)
    }

    pub fn tester_present(&mut self) -> Result<(), ProtocolError> {
        let response = self.request(&[0x3E, 0x00])?;
        expect_positive(&response, 0x7E)?;
        Ok(())
    }

    fn request(&mut self, data: &[u8]) -> Result<Vec<u8>, ProtocolError> {
        Ok(self.transport.send_receive(data, DEFAULT_TIMEOUT)?)
    }
}

impl<T: Transport> Protocol for Uds<T> {
    fn name(&self) -> &'static str {
        "UDS"
    }
}

fn expect_positive(response: &[u8], expected_first_byte: u8) -> Result<(), ProtocolError> {
    match response.first() {
        Some(&0x7F) => {
            let sid = response.get(1).copied().unwrap_or(0);
            let nrc = response.get(2).copied().unwrap_or(0);
            Err(ProtocolError::UnexpectedResponse(format!(
                "negative response for SID 0x{sid:02X}: NRC 0x{nrc:02X}"
            )))
        }
        Some(&byte) if byte == expected_first_byte => Ok(()),
        Some(&byte) => Err(ProtocolError::UnexpectedResponse(format!(
            "expected response 0x{expected_first_byte:02X}, got 0x{byte:02X}"
        ))),
        None => Err(ProtocolError::UnexpectedResponse("empty response".to_string())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transport::mock::MockTransport;
    use crate::transport::mock_ecu::MockEcu;

    fn connected_uds_with_ecu(ecu: MockEcu) -> Uds<MockTransport> {
        let mut transport = MockTransport::with_ecu(ecu);
        transport.connect().unwrap();
        Uds::new(transport)
    }

    #[test]
    fn full_session_reads_vin_and_dtcs() {
        let ecu = MockEcu::new().with_dtc(0x001234, 0x08);
        let mut uds = connected_uds_with_ecu(ecu);

        uds.diagnostic_session_control(DiagnosticSession::Extended).unwrap();

        let vin_bytes = uds.read_data_by_identifier(0xF190).unwrap();
        assert_eq!(String::from_utf8(vin_bytes).unwrap(), "WBAF12345LC000001");

        let dtcs = uds.read_dtc_information().unwrap();
        assert_eq!(dtcs, vec![Dtc { code: 0x001234, status: 0x08 }]);

        uds.tester_present().unwrap();
    }

    #[test]
    fn reading_unknown_did_surfaces_negative_response_as_error() {
        let mut uds = connected_uds_with_ecu(MockEcu::new());
        let result = uds.read_data_by_identifier(0x0000);
        assert!(matches!(result, Err(ProtocolError::UnexpectedResponse(_))));
    }

    #[test]
    fn no_ecu_behind_transport_surfaces_transport_timeout() {
        let mut transport = MockTransport::new();
        transport.connect().unwrap();
        let mut uds = Uds::new(transport);

        let result = uds.tester_present();
        assert!(matches!(result, Err(ProtocolError::Transport(_))));
    }
}
