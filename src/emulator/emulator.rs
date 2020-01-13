use super::error;
use super::opcodes;
use super::OpcodeType;
use super::{addressing_modes::get_size, AddressingMode};

fn invalid_mode<T>(mode_used: AddressingMode) -> T {
    panic!("The addressign mode used ({:?}) is either not valid for this opcode, or expects an argument which was not provided",mode_used)
}

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
    pub fn step(&mut self) -> Result<(), error::EmulatorError> {
        println!("Step on {:04X}", self.cpu.PC);
        let code = self.ram[self.cpu.PC as usize];
        let code = match opcodes::from_code(code) {
            None => return Err(error::EmulatorError::UnknownOp(code)),
            Some(v) => v,
        };
        println!(" Opcode {:?}", code.name);
        let arg: Option<u16> = match code.addr_mode {
            AddressingMode::IMPL => None,
            AddressingMode::A => None,
            AddressingMode::IMM => {
                let addr = fetch!(self PC+1);
                Some(addr as u16)
            }
            AddressingMode::ABS => {
                let addr = self.cpu.PC as usize + 1;
                Some(fetch!(self D addr) as u16)
            }
            AddressingMode::ZPG => {
                let addr = self.cpu.PC as usize + 1;
                Some(fetch!(self addr) as u16)
            }
            _ => {
                unimplemented!("Unimplemented addressing mode {:?}", code.addr_mode);
            }
        };
        println!(" Argument: {:#04X?}", arg);
        match code.name {
            OpcodeType::BRK => {
                println!("Stepped on break. Ending");
                println!("{:#?}", self.cpu);
                return Err(error::EmulatorError::Break);
            }
            OpcodeType::LDA => match code.addr_mode {
                AddressingMode::IMM => {
                    self.cpu.A = arg.unwrap_or_else(|| invalid_mode(code.addr_mode)) as u8
                }
                AddressingMode::ABS => {
                    self.cpu.A = fetch!(self arg.unwrap_or_else(||invalid_mode(code.addr_mode)))
                }
                AddressingMode::ZPG => {
                    self.cpu.A = fetch!(self arg.unwrap_or_else(||invalid_mode(code.addr_mode)))
                }
                _ => panic!("Invalid addressing mode for {:?}", code.name),
            },
            OpcodeType::STA => match code.addr_mode {
                AddressingMode::ABS => {
                    self.ram[arg.unwrap_or_else(|| invalid_mode(code.addr_mode)) as usize] =
                        self.cpu.A
                }
                _ => panic!("Invalid addressing mode for {:?}", code.name),
            },
            OpcodeType::ADC => match code.addr_mode {
                AddressingMode::IMM => {
                    self.cpu.A += arg.unwrap_or_else(|| invalid_mode(code.addr_mode)) as u8
                }
                _ => panic!("Invalid addressing mode for {:?}", code.name),
            },
            OpcodeType::JMP => match code.addr_mode {
                AddressingMode::ABS => {
                    self.cpu.PC = arg.unwrap_or_else(|| invalid_mode(code.addr_mode)) as u16
                }
                _ => panic!("Invalid addressing mode for {:?}", code.name),
            },
            _ => {
                unimplemented!(
                    "Unimplemented opcode {:?} with {:?}",
                    code.name,
                    code.addr_mode
                );
            }
        }
        if code.name != OpcodeType::JMP {
            self.cpu.PC += get_size(code.addr_mode) as u16;
        }
        Ok(())
    }
    pub fn restart(&mut self) {
        self.cycles = 0;
        self.ram = [0x00; 0x10000];
        self.cpu = Cpu::default();
    }
}

#[allow(non_snake_case)]
pub struct Cpu {
    A: u8,
    X: u8,
    Y: u8,
    PC: u16,
}
impl std::default::Default for Cpu {
    fn default() -> Self {
        Self {
            A: 0x00,
            X: 0x00,
            Y: 0x00,
            PC: 0x0600,
        }
    }
}
impl std::fmt::Debug for Cpu {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(
            f,
            "Registers: \n PC: {:04X}\n A: {:02X} X: {:02X} Y: {:02X}\n",
            self.PC, self.A, self.X, self.Y,
        )
    }
}
