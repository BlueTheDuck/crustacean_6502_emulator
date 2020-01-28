use std::convert::{From, Into};

#[derive(Copy, Clone, PartialEq, PartialOrd, Debug)]
pub struct Address(pub usize);
impl From<usize> for Address {
    fn from(v: usize) -> Self {
        if v > 0xFFFF {
            panic!("Address is bigger than 0x10000 (Got 0x{:04X})", v);
        }
        Self(v)
    }
}
impl From<i32> for Address {
    fn from(v: i32) -> Self {
        if v < 0 || v > 0xFFFF {
            panic!("Can't use {}i32 as address", v);
        }
        Address((v & 0xFFFF) as usize)
    }
}
impl From<u16> for Address {
    fn from(v: u16) -> Self {
        Self(v as usize)
    }
}
impl From<u8> for Address {
    fn from(v: u8) -> Self {
        Address((v as usize) & 0xFF)
    }
}
impl Into<usize> for Address {
    fn into(self) -> usize {
        *self & 0xFFFF
    }
}
impl Into<u16> for Address {
    fn into(self) -> u16 {
        (*self & 0xFFFF) as u16
    }
}
impl std::ops::Deref for Address {
    type Target = usize;
    fn deref(&self) -> &usize {
        &self.0
    }
}
impl std::ops::DerefMut for Address {
    fn deref_mut(&mut self) -> &mut usize {
        &mut self.0
    }
}
impl std::ops::Add for Address {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        let result = *self + *rhs;
        if result > 0xFFFF {
            panic!(
                "Resulting op between Address and 0x{:04X} is bigger than 0xFFFF",
                *rhs
            );
        }
        Address(result)
    }
}
impl Address {
    pub fn same_page_add<I: Into<usize>>(self, rhs: I) -> Self {
        let rhs = rhs.into() & 0x00FF;
        let hi = *self & 0xFF00;
        let lo = (*self + rhs) & 0xFF;
        Address(hi | lo)
    }
    pub fn next(&self) -> Self {
        Address(self.0 + 1)
    }
}

pub fn get_size(addr_mode: AddressingMode) -> usize {
    OP_SIZES[addr_mode as usize]
}

//A,abs,absX,absY,imm,impl,ind,indX,indY,rel,zpg,zpgX,zpgY
//1,  3,   3,   3,   2,  1,  3,   2,   2,  2,  2,   2,   2
pub static OP_SIZES: [usize; 13] = [1, 3, 3, 3, 2, 1, 3, 2, 2, 2, 2, 2, 2];

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum AddressingMode {
    A = 0, // LSR A
    ABS,   // LDA $1234
    ABSX,  // STA $3000,X
    ABSY,  // AND $4000,Y
    IMM,   // LDA #$10
    IMPL,  // CLC
    IND,   // JMP ($FFFC)
    INDX,  // LDA ($40,X)
    INDY,  // LDA ($40),Y
    REL,   // LABEL // +4
    ZPG,   // LDA $10
    ZPGX,  // LDA $10,X
    ZPGY,  // LDA $10,Y
}
