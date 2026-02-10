# Triad
A toy operating system written in Rust. The name is a homage to [Operating Systems: Three Easy Pieces (OSTEP)](https://pages.cs.wisc.edu/~remzi/OSTEP/), a foundational book on operating systems. The project is structured around the book's three fundamental pillars -

1.  Virtualization
2.  Concurrency
3.  Persistence

## Feature Set 

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

## Acknowledgements
This Rust OS was created with the help of the following resources - 
1. https://os.phil-opp.com/
2. https://wiki.osdev.org/Expanded_Main_Page
3. https://osblog.stephenmarz.com/
