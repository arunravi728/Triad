# Triad
A toy operating system written in Rust.

The reasons I started developing Triad are three fold - 

1. Learn how an Operating System is built.
2. Get an in-depth understanding of concurrency.
3. Understand the Rust programming language.

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

## Test

```
# Run all kernel unit tests
cargo ktest --lib

# Run specific kernel integration test
cargo ktest --test <test-name>
```
## AI Use

I used AI chatbots mainly as a supercharged search engine to look up concepts and work through ideas. Use of agentic AI harnesses to generate code was kept to a minimum. Almost every line of code was read, understood, and written by hand.

## Acknowledgements
This Rust OS was created with the help of the following resources - 
1. https://os.phil-opp.com/
2. https://wiki.osdev.org/Expanded_Main_Page
3. https://osblog.stephenmarz.com/
