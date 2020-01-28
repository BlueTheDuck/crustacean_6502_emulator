use super::addressing_modes::Address;

/* #region Ram */
pub struct Ram(pub [u8; 0x10000]);
impl Ram {
    pub fn load(&mut self, code: [u8; 0x10000]) {
        self.0 = code;
    }
}
impl std::ops::Index<Address> for Ram {
    type Output = u8;
    fn index(&self, idx: Address) -> &u8 {
        let idx: usize = idx.into();
        &self.0[idx]
    }
}
impl std::ops::IndexMut<Address> for Ram {
    fn index_mut(&mut self, idx: Address) -> &mut u8 {
        let idx: usize = idx.into();
        &mut self.0[idx]
    }
}
impl std::ops::Deref for Ram {
    type Target = [u8; 0x10000];
    fn deref(&self) -> &[u8; 0x10000] {
        &self.0
    }
}
/* #endregion */

#[repr(u8)]
#[derive(PartialEq, Copy, Clone, Debug)]
pub enum Flags {
    Negative = 128,
    Overflow = 64,
    AlwaysOne = 32,
    Break = 16,
    Decimal = 8,
    Int = 4,
    Zero = 2,
    Carry = 1,
}
#[allow(non_snake_case)]
pub struct Registers {
    pub A: u8,
    pub X: u8,
    pub Y: u8,
    pub PC: Address,
    pub flags: u8,
}
impl Registers {
    pub fn test(&self, flag: Flags) -> bool {
        (self.flags & flag as u8) != 0
    }
    pub fn set_flag(&mut self, flag: Flags, status: bool) {
        if status {
            self.flags |= flag as u8;
        } else {
            self.flags &= 0xFF - flag as u8;
        }
    }
    pub fn add_a(&mut self, n: u8) {
        let n = n as u16;
        let a = self.A as u16;
        let res = n + a;
        if res > 0xFF {
            self.set_flag(Flags::Carry, true);
        } else {
            self.set_flag(Flags::Carry, false);
        }
        self.set_a((res & 0xFF) as u8);
    }
    pub fn set_a(&mut self, value: u8) {
        self.set_flag(Flags::Zero, value == 0x00); // Set Zero if A is zero
        self.set_flag(Flags::Negative, value & 0x80 != 0); // Test sign bit
        self.A = value;
    }
}
impl std::default::Default for Registers {
    fn default() -> Self {
        Self {
            A: 0x00,
            X: 0x00,
            Y: 0x00,
            PC: 0x0000usize.into(),
            flags: 0b_0010_0000,
        }
    }
}
impl std::fmt::Debug for Registers {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(
            f,
            "Registers: \n PC: {:04X}\n A: {:02X} X: {:02X} Y: {:02X}\nNV-BDIZC\n{:08b}",
            *self.PC, self.A, self.X, self.Y, self.flags
        )
    }
}
