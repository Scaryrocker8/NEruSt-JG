use crate::cartridge::Mirroring;
use bitflags::bitflags;

pub struct PPU {
    pub chr_rom: Vec<u8>,
    pub pallete_table: [u8; 32],
    pub vram: [u8; 2048],
    pub oam: [u8; 256],
    pub mirroring: Mirroring,

    addr_reg: AddressRegister,
    control_reg: ControlRegister,
    internal_data_buffer: u8,
}

impl PPU {
    pub fn new(chr_rom: Vec<u8>, mirroring: Mirroring) -> PPU {
        PPU {
            chr_rom,
            mirroring,
            pallete_table: [0; 32],
            vram: [0; 2048],
            oam: [0; 256],
            addr_reg: AddressRegister::new(),
            control_reg: ControlRegister::new(),
            internal_data_buffer: 0,
        }
    }

    // Horizontal:
    //   [ A ] [ a ]
    //   [ B ] [ b ]
    // Vertical:
    //   [ A ] [ B ]
    //   [ a ] [ b ]
    pub fn mirror_vram_addr(&self, addr: u16) -> u16 {
        let mirrored_vram = addr & 0b10111111111111; // mirror down 0x3000-0x3eff to 0x2000 - 0x2eff
        let vram_index = mirrored_vram - 0x2000; // to vram vector
        let name_table = vram_index / 0x400; // to the name table index
        match (&self.mirroring, name_table) {
            (Mirroring::Vertical, 2) | (Mirroring::Vertical, 3) => vram_index - 0x800,
            (Mirroring::Horizontal, 2) => vram_index - 0x400,
            (Mirroring::Horizontal, 1) => vram_index - 0x400,
            (Mirroring::Horizontal, 3) => vram_index - 0x800,
            _ => vram_index,
        }
    }

    pub fn write_to_addr_reg(&mut self, value: u8) {
        self.addr_reg.update(value);
    }

    pub fn write_to_control_reg(&mut self, value: u8) {
        self.control_reg.update(value);
    }

    fn increment_vram_addr(&mut self) {
        self.addr_reg
            .increment(self.control_reg.vram_addr_increment());
    }

    pub fn read_data(&mut self) -> u8 {
        let addr = self.addr_reg.get();
        self.increment_vram_addr();

        match addr {
            0x0000..=0x1FFF => {
                let result = self.internal_data_buffer;
                self.internal_data_buffer = self.chr_rom[addr as usize];
                result
            }
            0x2000..=0x2FFF => {
                let result = self.internal_data_buffer;
                self.internal_data_buffer = self.vram[self.mirror_vram_addr(addr) as usize];
                result
            }
            0x3000..=0x3EFF => {
                let result = self.internal_data_buffer;
                self.internal_data_buffer = self.vram[self.mirror_vram_addr(addr) as usize];
                result
            }
            0x3F00..=0x3FFF => self.pallete_table[(addr & 0x1f) as usize],
            _ => panic!("PPU read from unknown address {}", addr),
        }
    }

    pub fn write_to_data_reg(&mut self, value: u8) {
        let addr = self.addr_reg.get();
        match addr {
            0x0000..=0x1FFF => panic!("attempt to write to PPU address {:x}", addr),
            0x2000..=0x2FFF => {
                self.vram[self.mirror_vram_addr(addr) as usize] = value;
            }
            0x3000..=0x3EFF => {
                self.vram[self.mirror_vram_addr(addr) as usize] = value;
            }

            //Addresses $3F10/$3F14/$3F18/$3F1C are mirrors of $3F00/$3F04/$3F08/$3F0C
            0x3F10 | 0x3F14 | 0x3F18 | 0x3F1C => {
                let add_mirror = addr - 0x10;
                self.pallete_table[(add_mirror & 0x1f) as usize] = value;
            }
            0x3F00..=0x3FFF => {
                self.pallete_table[(addr & 0x1f) as usize] = value;
            }
            _ => panic!("PPU write to unknown address {:x}", addr),
        }
        self.increment_vram_addr();
    }
}

pub struct AddressRegister {
    value: (u8, u8),
    hi_ptr: bool,
}

impl AddressRegister {
    pub fn new() -> AddressRegister {
        AddressRegister {
            value: (0, 0),
            hi_ptr: true,
        }
    }

    pub fn set(&mut self, value: u16) {
        self.value.0 = (value >> 8) as u8;
        self.value.1 = (value & 0xFF) as u8;
    }

    pub fn get(&self) -> u16 {
        (self.value.0 as u16) << 8 | self.value.1 as u16
    }

    pub fn increment(&mut self, inc: u8) {
        let lo = self.value.1;
        self.value.1 = self.value.1.wrapping_add(inc);
        if lo > self.value.1 {
            self.value.0 = self.value.0.wrapping_add(1);
        }
    }

    pub fn update(&mut self, data: u8) {
        if self.hi_ptr {
            self.value.0 = data;
        } else {
            self.value.1 = data;
        }

        if self.get() > 0x3FFF {
            self.set(self.get() & 0x3FFF);
        }
        self.hi_ptr = !self.hi_ptr;
    }

    pub fn reset_latch(&mut self) {
        self.hi_ptr = true;
    }
}

bitflags! {

   pub struct ControlRegister: u8 {
       const NAMETABLE1              = 0b00000001;
       const NAMETABLE2              = 0b00000010;
       const VRAM_ADD_INCREMENT      = 0b00000100;
       const SPRITE_PATTERN_ADDR     = 0b00001000;
       const BACKROUND_PATTERN_ADDR  = 0b00010000;
       const SPRITE_SIZE             = 0b00100000;
       const MASTER_SLAVE_SELECT     = 0b01000000;
       const GENERATE_NMI            = 0b10000000;
   }
}

impl ControlRegister {
    pub fn new() -> Self {
        ControlRegister::from_bits_truncate(0b00000000)
    }

    pub fn vram_addr_increment(&self) -> u8 {
        if !self.contains(ControlRegister::VRAM_ADD_INCREMENT) {
            1
        } else {
            32
        }
    }

    pub fn update(&mut self, data: u8) {
        *self = ControlRegister::from_bits_truncate(data);
    }
}
