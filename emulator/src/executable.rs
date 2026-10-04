use std::fs;

pub const MAGIC: &[u8; 4] = b"STAT";
pub const VERSION: u16 = 1;
pub const HEADER_SIZE: usize = 32;

pub struct Executable {
    pub entry: u32,
    pub code: Vec<u8>,
    pub data: Vec<u8>,
}

impl Executable {
    pub fn load(path: &str) -> Result<Self, String> {
        let bytes = fs::read(path).map_err(|e| e.to_string())?;

        if bytes.len() < HEADER_SIZE {
            return Err("executable is truncated".into());
        }

        if &bytes[0..4] != MAGIC {
            return Err("invalid executable magic".into());
        }

        let version = u16::from_le_bytes([bytes[4], bytes[5]]);

        if version != VERSION {
            return Err(format!("unsupported executable version {}", version));
        }

        let entry = u32::from_le_bytes(bytes[8..12].try_into().unwrap());
        let code_offset = u32::from_le_bytes(bytes[12..16].try_into().unwrap()) as usize;
        let code_size = u32::from_le_bytes(bytes[16..20].try_into().unwrap()) as usize;
        let data_offset = u32::from_le_bytes(bytes[20..24].try_into().unwrap()) as usize;
        let data_size = u32::from_le_bytes(bytes[24..28].try_into().unwrap()) as usize;

        let code_end = code_offset
            .checked_add(code_size)
            .ok_or("code size overflow")?;

        let data_end = data_offset
            .checked_add(data_size)
            .ok_or("data size overflow")?;

        if code_end > bytes.len() || data_end > bytes.len() {
            return Err("executable contents are truncated".into());
        }

        Ok(Self {
            entry,
            code: bytes[code_offset..code_end].to_vec(),
            data: bytes[data_offset..data_end].to_vec(),
        })
    }
}
