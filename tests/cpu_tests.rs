#[cfg(test)]
mod test {
    use nerust_jg::CPU;
    use nerust_jg::Memory;
    use nerust_jg::bus::Bus;
    use nerust_jg::cartridge::Rom;
    use nerust_jg::cartridge::test::test_rom;
    use nerust_jg::opcodes;
    use std::collections::HashMap;

    // ============================================================================
    // Helper Functions
    // ============================================================================

    /// Creates a test ROM with a custom program loaded at 0x8000
    fn create_test_rom_with_program(program: Vec<u8>) -> Rom {
        let mut test_rom = vec![
            0x4E, 0x45, 0x53, 0x1A, // NES magic
            0x01, // 1 PRG ROM page (16KB)
            0x00, // 0 CHR ROM pages
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ];

        let mut prg_rom = vec![0; 16384];
        for (i, &byte) in program.iter().enumerate() {
            if i < prg_rom.len() {
                prg_rom[i] = byte;
            }
        }
        // Set reset vector to point to 0x8000 (start of ROM)
        prg_rom[0x3FFC] = 0x00;
        prg_rom[0x3FFD] = 0x80;

        test_rom.extend(prg_rom);
        Rom::new(&test_rom).unwrap()
    }

    /// Generates a trace string for CPU instruction debugging
    /// Format: "ADDR  BYTES  MNEMONIC OPERANDS    A:XX X:XX Y:XX P:XX SP:XX"
    pub fn trace(cpu: &mut CPU) -> String {
        let ref opcodes: HashMap<u8, &'static opcodes::OpCode> = *opcodes::OPCODES_MAP;

        let code = cpu.mem_read(cpu.program_counter);
        let ops = opcodes.get(&code).unwrap();

        let begin = cpu.program_counter;
        let mut hex_dump = vec![];
        hex_dump.push(code);

        let (mem_addr, stored_value) = match ops.mode {
            nerust_jg::cpu::AddressingMode::Immediate
            | nerust_jg::cpu::AddressingMode::NoneAddressing => (0, 0),
            _ => {
                // Temporarily adjust program counter to point at operand for get_operand_address
                let original_pc = cpu.program_counter;
                cpu.program_counter = begin + 1;
                let addr = cpu.get_operand_address(&ops.mode);
                cpu.program_counter = original_pc;
                (addr, cpu.mem_read(addr))
            }
        };

        let tmp = match ops.len {
            1 => match ops.code {
                0x0a | 0x4a | 0x2a | 0x6a => format!("A "),
                _ => String::from(""),
            },
            2 => {
                let address: u8 = cpu.mem_read(begin + 1);
                hex_dump.push(address);

                match ops.mode {
                    nerust_jg::cpu::AddressingMode::Immediate => format!("#${:02x}", address),
                    nerust_jg::cpu::AddressingMode::ZeroPage => {
                        format!("${:02x} = {:02x}", mem_addr, stored_value)
                    }
                    nerust_jg::cpu::AddressingMode::ZeroPage_X => {
                        format!(
                            "${:02x},X @ {:02x} = {:02x}",
                            address, mem_addr, stored_value
                        )
                    }
                    nerust_jg::cpu::AddressingMode::ZeroPage_Y => {
                        format!(
                            "${:02x},Y @ {:02x} = {:02x}",
                            address, mem_addr, stored_value
                        )
                    }
                    nerust_jg::cpu::AddressingMode::Indirect_X => {
                        format!(
                            "(${:02x},X) @ {:02x} = {:04x} = {:02x}",
                            address,
                            (address.wrapping_add(cpu.register_x)),
                            mem_addr,
                            stored_value
                        )
                    }
                    nerust_jg::cpu::AddressingMode::Indirect_Y => {
                        format!(
                            "(${:02x}),Y = {:04x} @ {:04x} = {:02x}",
                            address,
                            (mem_addr.wrapping_sub(cpu.register_y as u16)),
                            mem_addr,
                            stored_value
                        )
                    }
                    nerust_jg::cpu::AddressingMode::NoneAddressing => {
                        let address: usize =
                            (begin as usize + 2).wrapping_add((address as i8) as usize);
                        format!("${:04x}", address)
                    }
                    _ => panic!(
                        "unexpected addressing mode {:?} has ops-len 2. code {:02x}",
                        ops.mode, ops.code
                    ),
                }
            }
            3 => {
                let address_lo = cpu.mem_read(begin + 1);
                let address_hi = cpu.mem_read(begin + 2);
                hex_dump.push(address_lo);
                hex_dump.push(address_hi);

                let address = cpu.mem_read_u16(begin + 1);

                match ops.mode {
                    nerust_jg::cpu::AddressingMode::NoneAddressing => {
                        if ops.code == 0x6c {
                            let jmp_addr = if address & 0x00FF == 0x00FF {
                                let lo = cpu.mem_read(address);
                                let hi = cpu.mem_read(address & 0xFF00);
                                (hi as u16) << 8 | (lo as u16)
                            } else {
                                cpu.mem_read_u16(address)
                            };
                            format!("(${:04x}) = {:04x}", address, jmp_addr)
                        } else {
                            format!("${:04x}", address)
                        }
                    }
                    nerust_jg::cpu::AddressingMode::Absolute => {
                        format!("${:04x} = {:02x}", mem_addr, stored_value)
                    }
                    nerust_jg::cpu::AddressingMode::Absolute_X => {
                        format!(
                            "${:04x},X @ {:04x} = {:02x}",
                            address, mem_addr, stored_value
                        )
                    }
                    nerust_jg::cpu::AddressingMode::Absolute_Y => {
                        format!(
                            "${:04x},Y @ {:04x} = {:02x}",
                            address, mem_addr, stored_value
                        )
                    }
                    _ => panic!(
                        "unexpected addressing mode {:?} has ops-len 3. code {:02x}",
                        ops.mode, ops.code
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

        format!(
            "{:47} A:{:02x} X:{:02x} Y:{:02x} P:{:02x} SP:{:02x}",
            asm_str,
            cpu.register_a,
            cpu.register_x,
            cpu.register_y,
            cpu.status.bits(),
            cpu.stack_pointer
        )
        .to_ascii_uppercase()
    }

    // ============================================================================
    // Basic Instruction Tests
    // ============================================================================

    #[test]
    fn test_0xa9_lda_immediate_load_data() {
        let program = vec![0xa9, 0x05, 0x00]; // LDA #$05, BRK
        let rom = create_test_rom_with_program(program);
        let mut cpu = CPU::new(Bus::new(rom));
        cpu.reset();
        cpu.run();

        assert_eq!(cpu.register_a, 5);
        assert!(cpu.status.bits() & 0b0000_0010 == 0b00); // Zero flag not set
        assert!(cpu.status.bits() & 0b1000_0000 == 0); // Negative flag not set
    }

    #[test]
    fn test_0xaa_tax_move_a_to_x() {
        let program = vec![0xa9, 0x0a, 0xaa, 0x00]; // LDA #$0A, TAX, BRK
        let rom = create_test_rom_with_program(program);
        let mut cpu = CPU::new(Bus::new(rom));
        cpu.reset();
        cpu.run();

        assert_eq!(cpu.register_x, 10);
    }

    #[test]
    fn test_inx_overflow() {
        let program = vec![0xa9, 0xff, 0xaa, 0xe8, 0xe8, 0x00]; // LDA #$FF, TAX, INX, INX, BRK
        let rom = create_test_rom_with_program(program);
        let mut cpu = CPU::new(Bus::new(rom));
        cpu.reset();
        cpu.run();

        assert_eq!(cpu.register_x, 1); // 0xFF + 1 + 1 = 0x01 (wraps around)
    }

    // ============================================================================
    // Multi-Instruction Tests
    // ============================================================================

    #[test]
    fn test_5_ops_working_together() {
        let program = vec![0xa9, 0xc0, 0xaa, 0xe8, 0x00]; // LDA #$C0, TAX, INX, BRK
        let rom = create_test_rom_with_program(program);
        let mut cpu = CPU::new(Bus::new(rom));
        cpu.reset();
        cpu.run();

        assert_eq!(cpu.register_x, 0xc1);
    }

    #[test]
    fn test_lda_from_memory() {
        // Store 0x55 at address 0x10 in RAM, then load it into A
        let program = vec![
            0xa9, 0x55, // LDA #$55
            0x85, 0x10, // STA $10
            0xa5, 0x10, // LDA $10
            0x00, // BRK
        ];
        let rom = create_test_rom_with_program(program);
        let mut cpu = CPU::new(Bus::new(rom));
        cpu.reset();
        cpu.run();

        assert_eq!(cpu.register_a, 0x55);
    }

    // ============================================================================
    // Trace/Debug Format Tests
    // ============================================================================

    #[test]
    fn test_format_trace() {
        let mut bus = Bus::new(test_rom());
        bus.mem_write(100, 0xa2); // LDX #$01
        bus.mem_write(101, 0x01);
        bus.mem_write(102, 0xca); // DEX
        bus.mem_write(103, 0x88); // DEY
        bus.mem_write(104, 0x00); // BRK

        let mut cpu = CPU::new(bus);
        cpu.program_counter = 0x64;
        cpu.register_a = 1;
        cpu.register_x = 2;
        cpu.register_y = 3;

        let mut result: Vec<String> = vec![];
        cpu.run_with_callback(|cpu| {
            result.push(trace(cpu));
        });

        assert_eq!(
            "0064  A2 01     LDX #$01                        A:01 X:02 Y:03 P:24 SP:FD",
            result[0]
        );
        assert_eq!(
            "0066  CA        DEX                             A:01 X:01 Y:03 P:24 SP:FD",
            result[1]
        );
        assert_eq!(
            "0067  88        DEY                             A:01 X:00 Y:03 P:26 SP:FD",
            result[2]
        );
    }

    #[test]
    fn test_format_mem_access() {
        let mut bus = Bus::new(test_rom());
        // ORA ($33), Y
        bus.mem_write(100, 0x11);
        bus.mem_write(101, 0x33);

        // Set up indirect addressing data
        bus.mem_write(0x33, 0x00); // Low byte of target address
        bus.mem_write(0x34, 0x04); // High byte of target address

        // Target cell value
        bus.mem_write(0x400, 0xAA);

        let mut cpu = CPU::new(bus);
        cpu.program_counter = 0x64;
        cpu.register_y = 0;

        let mut result: Vec<String> = vec![];
        cpu.run_with_callback(|cpu| {
            result.push(trace(cpu));
        });

        assert_eq!(
            "0064  11 33     ORA ($33),Y = 0400 @ 0400 = AA  A:00 X:00 Y:00 P:24 SP:FD",
            result[0]
        );
    }
}
