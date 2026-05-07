use anyhow::{Result, anyhow};
use qrcode::QrCode;
use qrcode::render::unicode;
use rand::distributions::{Alphanumeric, DistString};

#[derive(Debug, Clone)]
pub struct PairingQr {
    pub instance: String,
    pub secret: String,
    pub payload: String,
}

impl PairingQr {
    pub fn generate() -> Self {
        let instance = format!("studio-{}", safe_random(10));
        let secret = safe_random(16);
        let payload = format!("WIFI:T:ADB;S:{instance};P:{secret};;");

        Self {
            instance,
            secret,
            payload,
        }
    }

    pub fn render_terminal(&self) -> Result<String> {
        let code = QrCode::new(self.payload.as_bytes())
            .map_err(|error| anyhow!("failed to build QR code: {error}"))?;

        Ok(code.render::<unicode::Dense1x2>().quiet_zone(true).build())
    }
}

fn safe_random(length: usize) -> String {
    Alphanumeric.sample_string(&mut rand::thread_rng(), length)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generates_adb_wifi_qr_payload() {
        let qr = PairingQr::generate();

        assert!(qr.instance.starts_with("studio-"));
        assert_eq!(qr.instance.len(), "studio-".len() + 10);
        assert_eq!(qr.secret.len(), 16);
        assert_eq!(
            qr.payload,
            format!("WIFI:T:ADB;S:{};P:{};;", qr.instance, qr.secret)
        );
        assert!(
            qr.payload
                .chars()
                .all(|character| character.is_ascii_alphanumeric()
                    || matches!(character, ':' | ';' | '-' | '_'))
        );
    }
}
