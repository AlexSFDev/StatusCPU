pub const CSR_STATUS: u16 = 0x000;
pub const CSR_CAUSE: u16 = 0x001;
pub const CSR_EPC: u16 = 0x002;
pub const CSR_TVEC: u16 = 0x003;
pub const CSR_SCRATCH: u16 = 0x004;
pub const CSR_CYCLE: u16 = 0x005;
pub const CSR_INSTRET: u16 = 0x006;
pub const CSR_MODE: u16 = 0x007;
pub const CSR_IE: u16 = 0x008;
pub const CSR_IP: u16 = 0x009;
pub const CSR_CPU_ID: u16 = 0x00a;

pub const STATUS_INTERRUPT_ENABLE: u32 = 1;

pub const MODE_USER: u32 = 0;
pub const MODE_SUPERVISOR: u32 = 1;
pub const MODE_MACHINE: u32 = 2;

#[derive(Clone, Copy)]
pub struct Csr {
    pub status: u32,
    pub cause: u32,
    pub epc: u32,
    pub tvec: u32,
    pub scratch: u32,
    pub cycle: u64,
    pub instret: u64,
    pub mode: u32,
    pub ie: u32,
    pub ip: u32,
}

impl Csr {
    pub fn new() -> Self {
        Self {
            status: 0,
            cause: 0,
            epc: 0,
            tvec: 0xffff_0100,
            scratch: 0,
            cycle: 0,
            instret: 0,
            mode: MODE_MACHINE,
            ie: 0,
            ip: 0,
        }
    }

    pub fn read(&self, csr: u16) -> u32 {
        match csr {
            CSR_STATUS => self.status,
            CSR_CAUSE => self.cause,
            CSR_EPC => self.epc,
            CSR_TVEC => self.tvec,
            CSR_SCRATCH => self.scratch,
            CSR_CYCLE => self.cycle as u32,
            CSR_INSTRET => self.instret as u32,
            CSR_MODE => self.mode,
            CSR_IE => self.ie,
            CSR_IP => self.ip,
            CSR_CPU_ID => 0x5354_4350,
            _ => 0,
        }
    }

    pub fn write(&mut self, csr: u16, value: u32) {
        match csr {
            CSR_STATUS => {
                self.status = value;
            }
            CSR_CAUSE => {
                self.cause = value;
            }
            CSR_EPC => {
                self.epc = value;
            }
            CSR_TVEC => {
                self.tvec = value;
            }
            CSR_SCRATCH => {
                self.scratch = value;
            }
            CSR_CYCLE => {
                self.cycle = value as u64;
            }
            CSR_INSTRET => {
                self.instret = value as u64;
            }
            CSR_MODE => {
                self.mode = value & 3;
            }
            CSR_IE => {
                self.ie = value;
            }
            CSR_IP => {
                self.ip = value;
            }
            _ => {}
        }
    }
}
