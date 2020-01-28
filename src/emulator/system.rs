use super::addressing_modes::{get_size, Address, AddressingMode};
use super::components::{Flags, Ram, Registers};
use super::error;
use super::opcodes;
use super::OpcodeType;

fn invalid_mode<T>(mode_used: AddressingMode) -> T {
    panic!("The addressign mode used ({:?}) is either not valid for this opcode, or expects an argument which was not provided",mode_used)
}

static RESET_VEC_ADDR: Address = Address(0xFFFC);

macro_rules! fetch {
    ($self:ident PC+$off:expr) => {
        $self.ram[$self.registers.PC + Address($off)]
    };
    ($self:ident $addr:expr) => {
        $self.ram[$addr]
    };
    ($self:ident D $addr:expr) => {
        ($self.ram[$addr.next()] as u16) << 8 | $self.ram[$addr] as u16
    };
}
macro_rules! operation {
    ($self:ident A+$b:expr) => {
        $self.registers.A.wrapping_add($b as u8)
    };
    ($self:ident X+$b:expr) => {
        $self.registers.X.wrapping_add($b as u8)
    };
    (unwrap $arg:ident $addr:ident) => {
        $arg.unwrap_or_else(|| invalid_mode($addr.addr_mode))
    };
}

pub struct System {
    pub cycles: usize,
    pub ram: Ram,
    pub registers: Registers,
}
impl System {
    pub fn new() -> Self {
        Self {
            cycles: 0,
            ram: Ram([0x00; 0x10000]),
            registers: Registers::default(),
        }
    }
    pub fn init(&mut self) -> Result<(), error::CpuError> {
        let lo: u16 = self.ram[RESET_VEC_ADDR] as u16;
        let hi: u16 = self.ram[RESET_VEC_ADDR.same_page_add(1usize)] as u16;
        let addr = hi << 8 | lo;
        self.registers.PC = addr.into();
        Ok(())
    }
    pub fn step(&mut self) -> Result<(), error::CpuError> {
        if self.cycles == 0 {
            println!("Initializing");
            self.init()?;
        }
        /* if self.registers.test(Flags::Break) {
            return Err(error::EmulatorError::Break);
        } */
        println!("Step on {:04X}", *self.registers.PC);
        let code = self.ram[self.registers.PC];
        let code = match opcodes::from_code(code) {
            None => return Err(error::CpuError::UnknownOp(code)),
            Some(v) => v,
        };
        println!(" Opcode {:?}", code.name);
        let arg: Option<u16> = match code.addr_mode {
            AddressingMode::IMPL => None, // No argument
            AddressingMode::A => None,    // No argument
            AddressingMode::IMM => {
                // Next byte is the argument
                let addr = fetch!(self PC+1);
                Some(addr as u16)
            }
            AddressingMode::ABS => {
                // Next 2 bytes are an address from where to fetch the real argument
                let addr = self.registers.PC.next();
                Some(fetch!(self D addr) as u16)
            }
            AddressingMode::ZPG => {
                // Next byte is an address from the range 0x0000-0x00FF
                let addr = self.registers.PC.same_page_add(1u16);
                Some(fetch!(self addr) as u16)
            }
            AddressingMode::INDX => {
                // Take the next byte and add it to X,
                // then use the result as an address and fetch 2 bytes
                let arg = fetch!(self PC+1); // Opcode arg
                let addr: Address = operation!(self X+arg).into(); // Zero-page addr
                let addr_lo = self.ram[addr] as usize;
                let addr_hi = self.ram[addr.next()] as usize;
                let res_addr = addr_hi << 8 | addr_lo;
                Some(res_addr as u16)
            }
            AddressingMode::REL => {
                // Add PC with the next byte
                let arg = fetch!(self PC+1) as u8 as i8 as isize;
                let pc = (*self.registers.PC) as isize;
                let new_pc = (arg + pc) & 0xFFFF;
                Some(new_pc as u16)
            }
            _ => {
                unimplemented!("Unimplemented addressing mode {:?}", code.addr_mode);
            }
        };
        let mut branch_taken = false; // Don't update PC if we take a branch
        println!(" Argument: {:#04X?}", arg);
        match code.name {
            OpcodeType::BRK => {
                println!("Stepped on break. Ending");
                println!("{:#?}", self.registers);
                /* return Err(error::EmulatorError::Break); */
            }
            OpcodeType::NOP => {}
            OpcodeType::LDA => match code.addr_mode {
                AddressingMode::IMM => self.registers.set_a(operation!(unwrap arg code) as u8),
                AddressingMode::ABS => self
                    .registers
                    .set_a(fetch!(self operation!(unwrap arg code).into())),
                AddressingMode::ZPG => self
                    .registers
                    .set_a(fetch!(self operation!(unwrap arg code).into())),
                _ => panic!("Invalid addressing mode for {:?}", code.name),
            },
            OpcodeType::STA => match code.addr_mode {
                AddressingMode::ABS => {
                    self.ram[operation!(unwrap arg code).into()] = self.registers.A
                }
                AddressingMode::ZPG => {
                    self.ram[operation!(unwrap arg code).into()] = self.registers.A
                }
                AddressingMode::INDX => {
                    self.ram[operation!(unwrap arg code).into()] = self.registers.A
                }
                _ => panic!("Invalid addressing mode for {:?}", code.name),
            },
            OpcodeType::ADC => match code.addr_mode {
                AddressingMode::IMM => self.registers.add_a(operation!(unwrap arg code) as u8),
                _ => panic!("Invalid addressing mode for {:?}", code.name),
            },
            OpcodeType::JMP => match code.addr_mode {
                AddressingMode::ABS => self.registers.PC = operation!(unwrap arg code).into(),
                _ => panic!("Invalid addressing mode for {:?}", code.name),
            },
            OpcodeType::BEQ if code.addr_mode == AddressingMode::REL => {
                if self.registers.test(Flags::Zero) {
                    self.registers.PC = operation!(unwrap arg code).into();
                    branch_taken = true;
                }
            }
            OpcodeType::BNE if code.addr_mode == AddressingMode::REL => {
                if !self.registers.test(Flags::Zero) {
                    self.registers.PC = operation!(unwrap arg code).into();
                    branch_taken = true;
                }
            }
            _ => {
                unimplemented!(
                    "Unimplemented opcode {:?} with {:?}",
                    code.name,
                    code.addr_mode
                );
            }
        }
        if branch_taken || code.name == OpcodeType::JMP {
            println!("Don't increment PC");
        } else {
            self.registers.PC = self.registers.PC + get_size(code.addr_mode).into();
        }
        self.cycles += 1;
        Ok(())
    }
    pub fn restart(&mut self) {
        self.cycles = 0;
        self.ram.load([0x00; 0x10000]);
        self.registers = Registers::default();
    }
}

mod test {
    #[test]
    fn test_flags() {
        use super::{Flags, Registers};
        let mut cpu: Registers = Registers::default();
        cpu.set_flag(Flags::Zero, true);
        assert_eq!(cpu.test(Flags::Zero), true);
        cpu.set_flag(Flags::Zero, false);
        assert_eq!(cpu.test(Flags::Zero), false);
        cpu.set_flag(Flags::Zero, true);
        cpu.set_flag(Flags::Negative, true);
        cpu.set_flag(Flags::Int, true);
        assert_eq!(cpu.test(Flags::Zero), true);
        assert_eq!(cpu.test(Flags::Negative), true);
        assert_eq!(cpu.test(Flags::Int), true);
        cpu.set_flag(Flags::Negative, false);
        assert_eq!(cpu.test(Flags::Zero), true);
        assert_eq!(cpu.test(Flags::Negative), false);
        assert_eq!(cpu.test(Flags::Int), true);
    }
}
