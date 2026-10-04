pub const UART_BASE: u32 = 0x1000_0000;
pub const TIMER_BASE: u32 = 0x1000_1000;
pub const IRQ_BASE: u32 = 0x1000_2000;
pub const CPU_BASE: u32 = 0x1000_3000;

pub struct Mmio {
    uart_status: u32,
    timer: u64,
    timer_compare: u64,
    timer_control: u32,
    irq_pending: u32,
    irq_enable: u32,
}

impl Mmio {
    pub fn new() -> Self {
        Self {
            uart_status: 1,
            timer: 0,
            timer_compare: 0,
            timer_control: 0,
            irq_pending: 0,
            irq_enable: 0,
        }
    }

    pub fn read(&self, address: u32) -> Option<u32> {
        match address {
            UART_BASE => Some(0),
            x if x == UART_BASE + 4 => Some(self.uart_status),

            TIMER_BASE => Some(self.timer as u32),
            x if x == TIMER_BASE + 4 => Some((self.timer >> 32) as u32),
            x if x == TIMER_BASE + 8 => Some(self.timer_compare as u32),
            x if x == TIMER_BASE + 12 => Some((self.timer_compare >> 32) as u32),
            x if x == TIMER_BASE + 16 => Some(self.timer_control),

            IRQ_BASE => Some(self.irq_pending),
            x if x == IRQ_BASE + 4 => Some(self.irq_enable),
            x if x == IRQ_BASE + 8 => Some(0),

            CPU_BASE => Some(1),
            x if x == CPU_BASE + 4 => Some(0x5354_4350),
            x if x == CPU_BASE + 8 => Some(1),
            x if x == CPU_BASE + 12 => Some(32),

            _ => None,
        }
    }

    pub fn write(&mut self, address: u32, value: u32) -> bool {
        match address {
            UART_BASE => {
                self.uart_status = 1;
                true
            }

            x if x == TIMER_BASE => {
                self.timer = (self.timer & 0xffff_ffff_0000_0000) | (value as u64);
                true
            }

            x if x == TIMER_BASE + 4 => {
                self.timer = (self.timer & 0x0000_0000_ffff_ffff) | ((value as u64) << 32);
                true
            }

            x if x == TIMER_BASE + 8 => {
                self.timer_compare = (self.timer_compare & 0xffff_ffff_0000_0000) | (value as u64);
                true
            }

            x if x == TIMER_BASE + 12 => {
                self.timer_compare =
                    (self.timer_compare & 0x0000_0000_ffff_ffff) | ((value as u64) << 32);
                true
            }

            x if x == TIMER_BASE + 16 => {
                self.timer_control = value;
                true
            }

            x if x == IRQ_BASE + 4 => {
                self.irq_enable = value;
                true
            }

            x if x == IRQ_BASE + 8 => {
                self.irq_pending &= !value;
                true
            }

            _ => false,
        }
    }

    pub fn tick(&mut self, cycles: u64) {
        self.timer = self.timer.wrapping_add(cycles);

        if (self.timer_control & 1) != 0
            && self.timer_compare != 0
            && self.timer >= self.timer_compare
        {
            self.irq_pending |= 1;
            self.timer_compare = 0;
        }
    }

    pub fn irq_pending(&self) -> u32 {
        self.irq_pending & self.irq_enable
    }
}
