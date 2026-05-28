# Triad
A toy operating system written in Rust. The kernel currently supports - 

1. UEFI Bootloader
2. Framebuffer Support
3. Serial Logging
4. Kernel Logging
5. Hardware Interrupts via chained PICs
6. Keyboard & Timers

## Build & Run

```
cargo run --bin qemu-uefi
```

## Test

```
# Run all kernel unit tests
cargo ktest --lib

# Run specific kernel integration test
cargo ktest --test <test-name>
```

## Acknowledgements
This Rust OS was created with the help of the following resources - 
1. https://os.phil-opp.com/
2. https://wiki.osdev.org/Expanded_Main_Page
3. https://osblog.stephenmarz.com/
