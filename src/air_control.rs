use hidapi::HidApi;
use hidapi::HidError;
use hidapi::HidDevice;

pub struct AirControl {
    device: HidDevice,
}

const AIR_CONTROL_PADDED_KEY: &[u8] = &[0x00, 0xc4, 0xc6, 0xc0, 0x92, 0x40, 0x23, 0xdc, 0x96];
const AIR_CONTROL_KEY: &[u8] = &[0xc4, 0xc6, 0xc0, 0x92, 0x40, 0x23, 0xdc, 0x96];
const CSTATE: [u8; 8] = [0x48, 0x74, 0x65, 0x6D, 0x70, 0x39, 0x39, 0x65];
const SHUFFLE: [usize; 8] = [2, 4, 0, 7, 1, 6, 5, 3];

const VENDOR_ID: u16 = 0x04d9;
const PRODUCT_ID: u16 = 0xa052;

#[derive(Debug)]
pub enum Response {
    Corrupted,
    Unknown(u8),
    CO2(f32),
    T(f32),
}

impl AirControl {
    pub fn open() -> Result<AirControl, HidError> {
        HidApi::new().and_then(|api| {
            api.open(VENDOR_ID, PRODUCT_ID).and_then(|device| {
                let control = AirControl { device: device };
                control.setup()
            })
        })
    }

    fn setup(self) -> Result<AirControl, HidError> {
        self.device
            .send_feature_report(AIR_CONTROL_PADDED_KEY)
            .map(|_| self)
    }

    fn decrypt(data: &[u8; 8]) -> [u8; 8] {
        let mut phase1: [u8; 8] = [0; 8];
        for i in 0..8 {
            phase1[SHUFFLE[i]] = data[i];
        }

        let mut phase2: [u8; 8] = [0; 8];
        for i in 0..8 {
            phase2[i] = phase1[i] ^ AIR_CONTROL_KEY[i];
        }

        let mut phase3: [u8; 8] = [0; 8];
        for i in 0..8 {
            phase3[i] = ((phase2[i] >> 3) | (phase2[(i + 7) % 8] << 5)) & 0xff;
        }

        let mut ctmp: [u8; 8] = [0; 8];
        for i in 0..8 {
            ctmp[i] = ((CSTATE[i] >> 4) | (CSTATE[i] << 4)) & 0xff;
        }

        let mut out: [u8; 8] = [0; 8];
        for i in 0..8 {
            out[i] = ((0x100 + (phase3[i] as u16) - (ctmp[i] as u16)) as u8) & 0xff;
        }
        out
    }

    fn is_checksum_valid(buf: &[u8; 8]) -> bool {
        let sum = (buf[0] as i16) + (buf[1] as i16) + (buf[2] as i16);
        buf[4] == 0x0d && (sum & 0xff) == buf[3] as i16
    }

    fn decode(&self, buf: &[u8; 8]) -> Response {
        let decrypted = AirControl::decrypt(buf);
        if AirControl::is_checksum_valid(&decrypted) {
            let op = decrypted[0];
            let val = ((decrypted[1] as i16) << 8) | (decrypted[2] as i16);

            if op == 0x50 {
                Response::CO2(val as f32)
            } else if op == 0x42 {
                Response::T((val as f32) / 16.0 - 273.15)
            } else {
                Response::Unknown(op)
            }
        } else {
            Response::Corrupted
        }
    }

    pub fn read(&self) -> Result<Response, HidError> {
        let mut buf: [u8; 8] = [0; 8];
        self.device.read(&mut buf).map(|_| self.decode(&buf))
    }
}