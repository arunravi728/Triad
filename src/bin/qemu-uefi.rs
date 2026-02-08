use std::{
    env,
    process::{self, Command},
};

use ovmf_prebuilt::ovmf_pure_efi;

fn main() {
    let ovmf_code = ovmf_pure_efi();

    let mut qemu = Command::new("qemu-system-x86_64");
    qemu.args([
        "-drive",
        &format!(
            "format=raw,if=pflash,readonly=on,file={}",
            ovmf_code.display()
        ),
        "-drive",
        &format!("format=raw,if=pflash,file={}", ovmf_code.display()),
        "-drive",
        &format!("format=raw,file={}", env!("UEFI_IMAGE")),
        "-serial",
        "stdio",
    ]);
    let exit_status = qemu.status().unwrap();
    process::exit(exit_status.code().unwrap_or(-1));
}
