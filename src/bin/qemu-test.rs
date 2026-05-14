use bootloader::DiskImageBuilder;
use ovmf_prebuilt::ovmf_pure_efi;
use std::{
    env,
    path::PathBuf,
    process::{self, Command},
};

fn main() {
    let kernel_binary = env::args().nth(1).expect("kernel binary path required");
    let kernel_path = PathBuf::from(&kernel_binary);

    let out_dir = kernel_path.parent().unwrap();
    let uefi_path = out_dir.join("test-uefi.img");

    DiskImageBuilder::new(kernel_path)
        .create_uefi_image(&uefi_path)
        .unwrap();

    let ovmf_code = ovmf_pure_efi();

    let exit_status = Command::new("qemu-system-x86_64")
        .args([
            "-drive",
            &format!(
                "format=raw,if=pflash,readonly=on,file={}",
                ovmf_code.display()
            ),
            "-drive",
            &format!("format=raw,if=pflash,file={}", ovmf_code.display()),
            "-drive",
            &format!("format=raw,file={}", uefi_path.display()),
            "-serial",
            "stdio",
            "-device",
            "isa-debug-exit,iobase=0xf4,iosize=0x04",
            "-display",
            "none",
        ])
        .status()
        .unwrap();

    // QemuExitCode::Success is 0x10, QEMU shifts it: (0x10 << 1) | 1 = 33
    // QemuExitCode::Failed  is 0x11, QEMU shifts it: (0x11 << 1) | 1 = 35
    match exit_status.code() {
        Some(33) => process::exit(0),
        Some(35) => process::exit(1),
        Some(code) => process::exit(code),
        None => process::exit(-1),
    }
}
