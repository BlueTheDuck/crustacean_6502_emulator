use super::cpu::{Cpu, Flags};
use super::error;
use super::opcodes;
use super::OpcodeType;
use super::{addressing_modes::get_size, AddressingMode};

fn invalid_mode<T>(mode_used: AddressingMode) -> T {
    panic!("The addressign mode used ({:?}) is either not valid for this opcode, or expects an argument which was not provided",mode_used)
}

static RESET_VEC_ADDR: usize = 0xFFFC;

macro_rules! fetch {
    ($self:ident PC+$off:expr) => {
        $self.ram[$self.cpu.PC as usize + $off]
    };
    ($self:ident $addr:expr) => {
        $self.ram[$addr as usize]
    };
    ($self:ident D $addr:expr) => {
        ($self.ram[$addr as usize + 1] as u16) << 8 | $self.ram[$addr as usize] as u16
    };
}
macro_rules! operation {
    ($self:ident A+$b:expr) => {
        $self.cpu.A.wrapping_add($b as u8)
    };
    ($self:ident X+$b:expr) => {
        $self.cpu.X.wrapping_add($b as u8)
    };
    (unwrap $arg:ident $addr:ident) => {
        $arg.unwrap_or_else(|| invalid_mode($addr.addr_mode))
    };
}

pub struct Emulator {
    pub cycles: usize,
    pub ram: [u8; 0x10000],
    pub cpu: Cpu,
}
impl Emulator {
    pub fn new() -> Self {
        Self {
            cycles: 0,
            ram: [0x00; 0x10000],
            cpu: Cpu::default(),
        }
    }
    pub fn init(&mut self) -> Result<(), error::EmulatorError> {
        let lo: u16 = self.ram[RESET_VEC_ADDR] as u16;
        let hi: u16 = self.ram[RESET_VEC_ADDR + 1] as u16;
        let addr = hi << 8 | lo;
        self.cpu.PC = addr;
        Ok(())
    }
    pub fn step(&mut self) -> Result<(), error::EmulatorError> {
        if self.cycles == 0 {
            println!("Initializing");
            self.init()?;
        }
        if self.cpu.test(Flags::Break) {
            return Err(error::EmulatorError::Break);
        }
        println!("Step on {:04X}", self.cpu.PC);
        let code = self.ram[self.cpu.PC as usize];
        let code = match opcodes::from_code(code) {
            None => return Err(error::EmulatorError::UnknownOp(code)),
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
                let addr = self.cpu.PC as usize + 1;
                Some(fetch!(self D addr) as u16)
            }
            AddressingMode::ZPG => {
                // Next byte is an address from the range 0x0000-0x00FF
                let addr = self.cpu.PC as usize + 1;
                Some(fetch!(self addr) as u16)
            }
            AddressingMode::INDX => {
                // Take the next byte and add it to X,
                // then use the result as an address and fetch 2 bytes
                let arg = fetch!(self PC+1); // Opcode arg
                let addr: u8 = operation!(self X+arg); // Zero-page addr
                let addr_lo = self.ram[addr as usize] as usize;
                let addr_hi = self.ram[addr.wrapping_add(1) as usize] as usize;
                let res_addr = addr_hi << 8 | addr_lo;
                Some(res_addr as u16)
            }
            AddressingMode::REL => {
                // Add PC with the next byte
                let arg = fetch!(self PC+1) as u8 as i8 as isize;
                let pc = self.cpu.PC as usize as isize;
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
                println!("{:#?}", self.cpu);
                return Err(error::EmulatorError::Break);
            }
            OpcodeType::NOP => {}
            OpcodeType::LDA => match code.addr_mode {
                AddressingMode::IMM => self.cpu.set_a(operation!(unwrap arg code) as u8),
                AddressingMode::ABS => self.cpu.set_a(fetch!(self operation!(unwrap arg code))),
                AddressingMode::ZPG => self.cpu.set_a(fetch!(self operation!(unwrap arg code))),
                _ => panic!("Invalid addressing mode for {:?}", code.name),
            },
            OpcodeType::STA => match code.addr_mode {
                AddressingMode::ABS => self.ram[operation!(unwrap arg code) as usize] = self.cpu.A,
                AddressingMode::ZPG => self.ram[operation!(unwrap arg code) as usize] = self.cpu.A,
                AddressingMode::INDX => self.ram[operation!(unwrap arg code) as usize] = self.cpu.A,
                _ => panic!("Invalid addressing mode for {:?}", code.name),
            },
            OpcodeType::ADC => match code.addr_mode {
                AddressingMode::IMM => self.cpu.add_a(operation!(unwrap arg code) as u8),
                _ => panic!("Invalid addressing mode for {:?}", code.name),
            },
            OpcodeType::JMP => match code.addr_mode {
                AddressingMode::ABS => self.cpu.PC = operation!(unwrap arg code) as u16,
                _ => panic!("Invalid addressing mode for {:?}", code.name),
            },
            OpcodeType::BEQ if code.addr_mode == AddressingMode::REL => {
                if self.cpu.test(Flags::Zero) {
                    self.cpu.PC = operation!(unwrap arg code);
                    branch_taken = true;
                }
            }
            OpcodeType::BNE if code.addr_mode == AddressingMode::REL => {
                if !self.cpu.test(Flags::Zero) {
                    self.cpu.PC = operation!(unwrap arg code);
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
            self.cpu.PC += get_size(code.addr_mode) as u16;
        }
        self.cycles += 1;
        Ok(())
    }
    pub fn restart(&mut self) {
        self.cycles = 0;
        self.ram = [0x00; 0x10000];
        self.cpu = Cpu::default();
    }
}

mod test {
    #[test]
    fn test_flags() {
        use super::{Cpu, Flags};
        let mut cpu: Cpu = Cpu::default();
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
