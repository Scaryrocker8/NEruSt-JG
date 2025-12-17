#[cfg(test)]
mod tests {
    use nerust_jg::cartridge::{CHR_ROM_PAGE_SIZE, Mirroring, PRG_ROM_PAGE_SIZE, Rom};

    // ============================================================================
    // Test ROM Builder
    // ============================================================================

    /// Helper structure for building test ROM files
    struct TestRom {
        header: Vec<u8>,
        trainer: Option<Vec<u8>>,
        prg_rom: Vec<u8>,
        chr_rom: Vec<u8>,
    }

    /// Creates a complete ROM file from components
    fn create_rom(rom: TestRom) -> Vec<u8> {
        let mut result = Vec::with_capacity(
            rom.header.len()
                + rom.trainer.as_ref().map_or(0, |t| t.len())
                + rom.prg_rom.len()
                + rom.chr_rom.len(),
        );

        result.extend(&rom.header);
        if let Some(t) = rom.trainer {
            result.extend(t);
        }
        result.extend(&rom.prg_rom);
        result.extend(&rom.chr_rom);

        result
    }

    // ============================================================================
    // Valid ROM Format Tests
    // ============================================================================

    #[test]
    fn test_ines_no_version() {
        let test_rom = create_rom(TestRom {
            header: vec![
                0x4E, 0x45, 0x53, 0x1A, // NES magic number
                0x02, // 2 PRG ROM pages
                0x01, // 1 CHR ROM page
                0x31, // Mapper and mirroring flags
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            ],
            trainer: None,
            prg_rom: vec![1; 2 * PRG_ROM_PAGE_SIZE],
            chr_rom: vec![2; 1 * CHR_ROM_PAGE_SIZE],
        });

        let rom = Rom::new(&test_rom).unwrap();

        assert_eq!(rom.prg_rom.len(), 2 * PRG_ROM_PAGE_SIZE);
        assert_eq!(rom.chr_rom.len(), 1 * CHR_ROM_PAGE_SIZE);
        assert_eq!(rom.mapper, 3);
        assert_eq!(rom.screen_mirroring, Mirroring::Vertical);
    }

    #[test]
    fn test_ines_with_trainer() {
        let test_rom = create_rom(TestRom {
            header: vec![
                0x4E,
                0x45,
                0x53,
                0x1A,         // NES magic number
                0x02,         // 2 PRG ROM pages
                0x01,         // 1 CHR ROM page
                0x31 | 0b100, // Mapper flags with trainer bit set
                0x00,
                0x00,
                0x00,
                0x00,
                0x00,
                0x00,
                0x00,
                0x00,
                0x00,
            ],
            trainer: Some(vec![0; 512]), // 512-byte trainer
            prg_rom: vec![1; 2 * PRG_ROM_PAGE_SIZE],
            chr_rom: vec![2; 1 * CHR_ROM_PAGE_SIZE],
        });

        let rom = Rom::new(&test_rom).unwrap();

        assert_eq!(rom.prg_rom.len(), 2 * PRG_ROM_PAGE_SIZE);
        assert_eq!(rom.chr_rom.len(), 1 * CHR_ROM_PAGE_SIZE);
        assert_eq!(rom.mapper, 3);
        assert_eq!(rom.screen_mirroring, Mirroring::Vertical);
    }

    // ============================================================================
    // Invalid ROM Format Tests
    // ============================================================================

    #[test]
    fn test_ines_invalid_format() {
        let test_rom = create_rom(TestRom {
            header: vec![
                0x4E, 0x45, 0x53, 0x1B, // Invalid magic number (0x1B instead of 0x1A)
                0x02, 0x01, 0x31, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            ],
            trainer: None,
            prg_rom: vec![1; 2 * PRG_ROM_PAGE_SIZE],
            chr_rom: vec![2; 1 * CHR_ROM_PAGE_SIZE],
        });

        let rom = Rom::new(&test_rom);

        assert_eq!(
            rom.err(),
            Some("File is not an iNES file format".to_string())
        );
    }

    #[test]
    fn test_ines_unsupported_version() {
        let test_rom = create_rom(TestRom {
            header: vec![
                0x4E, 0x45, 0x53, 0x1A, // NES magic number
                0x01, // 1 PRG ROM page
                0x01, // 1 CHR ROM page
                0x31, // Mapper flags
                0x08, // NES 2.0 format indicator (unsupported)
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            ],
            trainer: None,
            prg_rom: vec![1; 1 * PRG_ROM_PAGE_SIZE],
            chr_rom: vec![2; 1 * CHR_ROM_PAGE_SIZE],
        });

        let rom = Rom::new(&test_rom);

        assert_eq!(
            rom.err(),
            Some("NES2.0 format is not supported".to_string())
        );
    }
}
