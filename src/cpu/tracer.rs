use super::{op::OPS, AddressingMode, CPU};
use std::io::Write;

pub trait Loggable {
    fn log(&mut self);
}

impl Loggable for CPU {
    fn log(&mut self) {
        let code = self.read(self.pc);
        let ops = OPS[match OPS.binary_search_by_key(&code, |op| op.hex) {
            Ok(i) => i,
            Err(_) => panic!("Invalid opcode: {:02x}", code),
        }];

        let begin = self.pc;
        let mut hex_dump = vec![];
        hex_dump.push(code);

        let (mem_addr, stored_value) = match ops.addressing_mode {
            AddressingMode::Immediate
            | AddressingMode::Implicit
            | AddressingMode::Accumulator
            | AddressingMode::Relative
            | AddressingMode::Indirect => (0, 0),
            _ => {
                let addr = self
                    .get_absolute_addr(ops.addressing_mode, begin + 1)
                    .unwrap()
                    .0;
                (addr, self.read(addr))
            }
        };

        let tmp = match ops.size {
            1 => match ops.hex {
                0x0a | 0x4a | 0x2a | 0x6a => "A ".to_string(),
                _ => String::from(""),
            },
            2 => {
                let address: u8 = self.read(begin + 1);
                hex_dump.push(address);

                match ops.addressing_mode {
                    AddressingMode::Immediate => format!("#${:02x}", address),
                    AddressingMode::ZeroPage => {
                        format!("${:02x} = {:02x}", mem_addr, self.read(mem_addr))
                    }
                    AddressingMode::ZeroPageX => format!(
                        "${:02x},X @ {:02x} = {:02x}",
                        address, mem_addr, stored_value
                    ),
                    AddressingMode::ZeroPageY => format!(
                        "${:02x},Y @ {:02x} = {:02x}",
                        address, mem_addr, stored_value
                    ),
                    AddressingMode::IndexedIndirect => format!(
                        "(${:02x},X) @ {:02x} = {:04x} = {:02x}",
                        address,
                        (address.wrapping_add(self.x)),
                        mem_addr,
                        stored_value
                    ),
                    AddressingMode::IndirectIndexed => format!(
                        "(${:02x}),Y = {:04x} @ {:04x} = {:02x}",
                        address,
                        (mem_addr.wrapping_sub(self.y as u16)),
                        mem_addr,
                        stored_value
                    ),
                    AddressingMode::Implicit
                    | AddressingMode::Accumulator
                    | AddressingMode::Relative
                    | AddressingMode::Indirect => {
                        // assuming local jumps: BNE, BVS, etc....
                        let address: usize =
                            (begin as usize + 2).wrapping_add((address as i8) as usize);
                        format!("${:04x}", address)
                    }
                    _ => panic!(
                        "Unexpected addressing mode: {:?} for opcode: {:?}",
                        ops.addressing_mode, ops
                    ),
                }
            }
            3 => {
                let address_lo = self.read(begin + 1);
                let address_hi = self.read(begin + 2);
                hex_dump.push(address_lo);
                hex_dump.push(address_hi);

                let address = self.read_16(begin + 1);

                match ops.addressing_mode {
                    AddressingMode::Implicit
                    | AddressingMode::Accumulator
                    | AddressingMode::Relative
                    | AddressingMode::Indirect => {
                        if ops.hex == 0x6c {
                            //jmp indirect
                            let jmp_addr = if address & 0x00FF == 0x00FF {
                                let lo = self.read(address);
                                let hi = self.read(address & 0xFF00);
                                (hi as u16) << 8 | (lo as u16)
                            } else {
                                self.read_16(address)
                            };
                            format!("(${:04x}) = {:04x}", address, jmp_addr)
                        } else {
                            format!("${:04x}", address)
                        }
                    }
                    AddressingMode::Absolute => {
                        if !ops.name.starts_with('J') {
                            format!("${:04x} = {:02x}", mem_addr, stored_value)
                        } else {
                            format!("${:04x}", address)
                        }
                    }
                    AddressingMode::AbsoluteX => {
                        format!(
                            "${:04x},X @ {:04x} = {:02x}",
                            address, mem_addr, stored_value
                        )
                    }
                    AddressingMode::AbsoluteY => {
                        format!(
                            "${:04x},Y @ {:04x} = {:02x}",
                            address, mem_addr, stored_value
                        )
                    }
                    _ => panic!(
                        "unexpected addressing mode {:?} has ops-len 3. code {:02x}",
                        ops.addressing_mode, ops.hex
                    ),
                }
            }
            _ => String::from(""),
        };

        let hex_str = hex_dump
            .iter()
            .map(|z| format!("{:02x}", z))
            .collect::<Vec<String>>()
            .join(" ");
        let asm_str = format!("{:04x}  {:8} {: >4} {}", begin, hex_str, ops.name, tmp)
            .trim()
            .to_string();

        let ppu_cycles = self.cycles * 3;
        let ppu_scanline = ppu_cycles / 341;
        let ppu_cycle = ppu_cycles % 341;

        let msg = format!(
            "{:47} A:{:02x} X:{:02x} Y:{:02x} P:{:02x} SP:{:02x} PPU:{:3},{:3} CYC:{}",
            asm_str,
            self.acc,
            self.x,
            self.y,
            self.status,
            self.sp,
            ppu_scanline,
            ppu_cycle,
            self.cycles
        )
        .to_uppercase();

        writeln!(self.sink, "{}", msg).unwrap();
    }
}
