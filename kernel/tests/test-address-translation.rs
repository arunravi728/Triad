#![no_std]
#![no_main]

use bootloader_api::{config::Mapping, BootloaderConfig};
use core::panic::PanicInfo;
use kernel::memory::{translate::translate, vaddr::VirtualAddress};
use kernel::{exit_qemu, serial_print, serial_println, QemuExitCode};

pub static BOOTLOADER_CONFIG: BootloaderConfig = {
    let mut config = BootloaderConfig::new_default();
    config.mappings.physical_memory = Some(Mapping::Dynamic);
    config
};

bootloader_api::entry_point!(test_main, config = &BOOTLOADER_CONFIG);

fn test_main(boot_info: &'static mut bootloader_api::BootInfo) -> ! {
    serial_print!("test_address_translation...\t");

    let physical_memory_offset: u64 = match boot_info.physical_memory_offset.into_option() {
        Some(address) => address,
        None => panic!("Physical memory offset not enabled in the bootloader"),
    };

    // Test virtual address corresponding to physical address 0x0
    let vaddr_zero = VirtualAddress::new(physical_memory_offset);
    let paddr_zero = translate(vaddr_zero, physical_memory_offset);
    assert!(paddr_zero.is_some());
    assert_eq!(paddr_zero.unwrap().address(), 0x0);

    // Test VGA buffer address translation
    let vga_paddr = 0xb8000;
    let vga_vaddr = VirtualAddress::new(vga_paddr + physical_memory_offset);
    let translated_vga = translate(vga_vaddr, physical_memory_offset);
    assert!(translated_vga.is_some());
    assert_eq!(translated_vga.unwrap().address(), vga_paddr);

    serial_println!("[ok]");
    exit_qemu(QemuExitCode::Success);
    kernel::hlt()
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    kernel::test_panic_handler(info);
}
