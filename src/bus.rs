use crate::cpu::Memory;

const RAM: u16 = 0x0000;
const RAM_MIRRORS_END: u16 = 0x1fff;
const PPU_REGISTERS: u16 = 0x2000;
const PPU_REGISTERS_MIRRORS_END: u16 = 0x3fff;

const ROM: u16 = 0x8000;
const ROM_END: u16 = 0xFFFF;

pub struct Bus {
    cpu_vram: [u8; 2048],
    prg_rom: [u8; 0x8000],
}

impl Bus {
    pub fn new() -> Self {
        Bus {
            cpu_vram: [0; 2048],
            prg_rom: [0; 0x8000],
        }
    }
}

impl Memory for Bus {
    fn mem_read(&self, addr: u16) -> u8 {
        match addr {
            RAM..=RAM_MIRRORS_END => {
                let mirror_down_addr = addr & 0b00000111_11111111;
                self.cpu_vram[mirror_down_addr as usize]
            }
            PPU_REGISTERS..=PPU_REGISTERS_MIRRORS_END => {
                let _mirror_down_addr = addr & 0b00100000_00000111;
                todo!("PPU_REGISTERS not implemented");
            }
            ROM..=ROM_END => {
                let map_addr = addr - ROM;
                self.prg_rom[map_addr as usize]
            }
            _ => {
                println!("Ignoring memory address at {}", addr);
                0
            }
        }
    }

    fn mem_write(&mut self, addr: u16, value: u8) {
        match addr {
            RAM..=RAM_MIRRORS_END => {
                let mirror_down_addr = addr & 0b11111111111;
                self.cpu_vram[mirror_down_addr as usize] = value;
            }
            PPU_REGISTERS..=PPU_REGISTERS_MIRRORS_END => {
                let _mirror_down_addr = addr & 0b00100000_00000111;
                todo!("PPU_REGISTERS not implemented");
            }
            ROM..=ROM_END => {
                let map_addr = addr - ROM;
                self.prg_rom[map_addr as usize] = value;
            }
            _ => {
                println!("Ignoring memory write-address at {}", addr);
            }
        }
    }
}
