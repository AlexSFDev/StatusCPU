#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoryError {
    OutOfBounds,
    Misaligned,
}

pub struct Memory {
    data: Vec<u8>,
}

impl Memory {
    pub fn new(size: usize) -> Self {
        Self {
            data: vec![0; size],
        }
    }

    fn range(&self, address: u32, size: usize) -> Result<usize, MemoryError> {
        let start = address as usize;
        let end = start.checked_add(size).ok_or(MemoryError::OutOfBounds)?;
        if end > self.data.len() {
            return Err(MemoryError::OutOfBounds);
        }
        Ok(start)
    }

    pub fn read_u8(&self, address: u32) -> Result<u8, MemoryError> {
        let start = self.range(address, 1)?;
        Ok(self.data[start])
    }

    pub fn read_u16(&self, address: u32) -> Result<u16, MemoryError> {
        if (address & 1) != 0 {
            return Err(MemoryError::Misaligned);
        }
        let start = self.range(address, 2)?;
        Ok(u16::from_le_bytes([self.data[start], self.data[start + 1]]))
    }

    pub fn read_u32(&self, address: u32) -> Result<u32, MemoryError> {
        if (address & 3) != 0 {
            return Err(MemoryError::Misaligned);
        }
        let start = self.range(address, 4)?;
        Ok(u32::from_le_bytes([
            self.data[start],
            self.data[start + 1],
            self.data[start + 2],
            self.data[start + 3],
        ]))
    }

    pub fn write_u8(&mut self, address: u32, value: u8) -> Result<(), MemoryError> {
        let start = self.range(address, 1)?;
        self.data[start] = value;
        Ok(())
    }

    pub fn write_u16(&mut self, address: u32, value: u16) -> Result<(), MemoryError> {
        if (address & 1) != 0 {
            return Err(MemoryError::Misaligned);
        }
        let start = self.range(address, 2)?;
        self.data[start..start + 2].copy_from_slice(&value.to_le_bytes());
        Ok(())
    }

    pub fn write_u32(&mut self, address: u32, value: u32) -> Result<(), MemoryError> {
        if (address & 3) != 0 {
            return Err(MemoryError::Misaligned);
        }
        let start = self.range(address, 4)?;
        self.data[start..start + 4].copy_from_slice(&value.to_le_bytes());
        Ok(())
    }

    pub fn load(&mut self, address: u32, data: &[u8]) -> Result<(), MemoryError> {
        let start = self.range(address, data.len())?;
        self.data[start..start + data.len()].copy_from_slice(data);
        Ok(())
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }
}
