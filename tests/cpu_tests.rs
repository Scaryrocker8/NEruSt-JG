use nerust_jg::CPU;
use nerust_jg::cpu::CpuFlags;

#[test]
fn test_0xa9_lda_immediate_load_data() {
    let mut cpu = CPU::new();
    cpu.interpret(vec![0xa9, 0x05, 0x00]);
    assert_eq!(cpu.register_a, 0x05);
    assert!(!cpu.status.contains(CpuFlags::ZERO));
    assert!(!cpu.status.contains(CpuFlags::NEGATIVE));
}

#[test]
fn test_0xa9_lda_zero_flag() {
    let mut cpu = CPU::new();
    cpu.interpret(vec![0xa9, 0x00, 0x00]);
    assert!(cpu.status.contains(CpuFlags::ZERO));
}

#[test]
fn test_0xaa_tax_transfer_a_to_x() {
    let mut cpu = CPU::new();
    cpu.register_a = 10;
    cpu.interpret(vec![0xaa, 0x00]);
    assert_eq!(cpu.register_x, 10);
}

#[test]
fn test_0xe8_inx_increment_x() {
    let mut cpu = CPU::new();
    cpu.register_x = 10;
    cpu.interpret(vec![0xe8, 0x00]);
    assert_eq!(cpu.register_x, 11);
}

#[test]
fn test_5_ops_working_together() {
    let mut cpu = CPU::new();
    cpu.interpret(vec![0xa9, 0xc0, 0xaa, 0xe8, 0x00]);

    assert_eq!(cpu.register_x, 0xc1);
}

#[test]
fn test_inx_overflow() {
    let mut cpu = CPU::new();
    cpu.register_x = 0xff;
    cpu.interpret(vec![0xe8, 0xe8, 0x00]);

    assert_eq!(cpu.register_x, 1)
}

#[test]
fn test_lda_from_memory() {
    let mut cpu = CPU::new();
    cpu.mem_write(0x10, 0x55);

    cpu.interpret(vec![0xa5, 0x10, 0x00]);

    assert_eq!(cpu.register_a, 0x55);
}
#[test]
fn test_sta_from_memory() {
    let mut cpu = CPU::new();
    cpu.register_a = 0x66;
    cpu.interpret(vec![0x85, 0x10, 0x00]);
    assert_eq!(cpu.mem_read(0x10), 0x66);
}
