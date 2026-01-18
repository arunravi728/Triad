use lazy_static::lazy_static;

use crate::interrupts::idt::IdtIndex;
use crate::interrupts::segment::{Segment, SegmentSelector, CS};
use crate::interrupts::tss::load_tss;

use x86_64::addr::VirtAddr;

pub mod gdt;
pub mod idt;
pub mod privilege;
pub mod segment;
pub mod tss;

#[derive(Debug)]
#[repr(C)]
struct ExceptionStackFrame {
    instruction_pointer: VirtAddr,
    code_segment: SegmentSelector,
    cpu_flags: u64,
    stack_pointer: VirtAddr,
    stack_segment: SegmentSelector,
}

// The function prologue is a few lines of code at the beginning of a function, which prepare the
// stack and registers for use within the function. Thus, the prologue generally tends to write
// information before we can access the interrupt stack frame. To access the correct addresses,
// we need to run functions without a prologue. The [naked] attribute helps with this. Note we
// cannot call anything but a naked_asm! call from a naked function. Hence, link Rust functions
// to handle specific exceptions.
macro_rules! handler {
    ($name: ident) => {{
        #[unsafe(naked)]
        extern "C" fn wrapper() -> ! {
            core::arch::naked_asm!(
                // Save state of all mutable registers before interrupt handler call.
                // We need to do this to not get a page fault when the ISR returns.
                "push rax",
                "push rcx",
                "push rdx",
                "push rsi",
                "push rdi",
                "push r8",
                "push r9",
                "push r10",
                "push r11",

                // Calculate pointer to ExceptionStackFrame
                // The frame is pushed by the CPU before the pushes above.
                // 9 registers were pushed (9 * 8 bytes = 72).
                "mov rdi, rsp",
                "add rdi, 72",

                // Align stack to 16-bytes - x86_64 ABI requires 16-byte alignment before calls.
                "sub rsp, 8",
                "call {handler_fn}",
                "add rsp, 8",

                // Restore state before returning from the ISR. This will allow the return
                // address to properly populated and will prevent page faults on program resumption.
                "pop r11",
                "pop r10",
                "pop r9",
                "pop r8",
                "pop rdi",
                "pop rsi",
                "pop rdx",
                "pop rcx",
                "pop rax",

                // Interrupt Return
                "iretq",
                handler_fn = sym $name,
            );
        }
        wrapper
    }}
}

macro_rules! handler_with_error_code {
    ($name: ident) => {{
        #[unsafe(naked)]
        extern "C" fn wrapper() -> ! {
            core::arch::naked_asm!(
                // Save state of all mutable registers before interrupt handler call.
                // We need to do this to not get a page fault when the ISR returns.
                "push rax",
                "push rcx",
                "push rdx",
                "push rsi",
                "push rdi",
                "push r8",
                "push r9",
                "push r10",
                "push r11",

                // Load error code into rsi
                "mov rsi, [rsp + 9*8]",

                // Calculate pointer to ExceptionStackFrame
                // The frame is pushed by the CPU before the pushes above.
                // 9 registers were pushed (9 * 8 bytes = 72)
                // 1 error code was pushed (1 * 8 bytes = 8)
                "mov rdi, rsp",
                "add rdi, 80",

                // Align stack to 16-bytes - x86_64 ABI requires 16-byte alignment before calls.
                "sub rsp, 8",
                "call {handler_fn}",
                "add rsp, 8",

                // Restore state before returning from the ISR. This will allow the return
                // address to properly populated and will prevent page faults on program resumption.
                "pop r11",
                "pop r10",
                "pop r9",
                "pop r8",
                "pop rdi",
                "pop rsi",
                "pop rdx",
                "pop rcx",
                "pop rax",

                // Pop error code from stack
                "add rsp, 8",

                // Interrupt Return
                "iretq",
                handler_fn = sym $name,
            );
        }
        wrapper
    }}
}

lazy_static! {
    static ref IDT: idt::InterruptDescriptorTable = {
        let mut idt = idt::InterruptDescriptorTable::new();
        idt.add_interrupt_handler(
            IdtIndex::DivideErrorInterruptIndex,
            handler!(divide_error_handler),
        );

        idt.add_interrupt_handler(
            IdtIndex::InvalidOpcodeInterruptIndex,
            handler!(invalid_opcode_handler),
        );

        idt.add_interrupt_handler(
            IdtIndex::BreakpointInterruptIndex,
            handler!(breakpoint_interrupt_handler),
        );

        // As soon as our TSS is loaded, the CPU has access to a valid interrupt stack table (IST).
        // Then we can tell the CPU that it should use our new double fault stack by modifying our
        // double fault IDT entry.
        idt.add_interrupt_handler(
            IdtIndex::DoubleFaultInterruptIndex,
            handler_with_error_code!(double_fault_interrupt_handler),
        )
        .set_interrupt_stack_table_offset((DOUBLE_FAULT_IST_INDEX + 1) as u8);

        idt
    };
}

struct Selectors {
    code_selector: SegmentSelector,
    tss_selector: SegmentSelector,
}

struct GdtContainer {
    table: gdt::GlobalDescriptorTable,
    selectors: Selectors,
}

lazy_static! {
    static ref GDT: GdtContainer = {
        let mut gdt = gdt::GlobalDescriptorTable::new();
        let code_selector = gdt.add(gdt::Descriptor::kernel_code_segment());
        let tss_selector = gdt.add(gdt::Descriptor::tss_segment(&TSS));

        GdtContainer {
            table: gdt,
            selectors: Selectors {
                code_selector,
                tss_selector,
            },
        }
    };
}

pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;

lazy_static! {
    static ref TSS: tss::TaskStateSegment = {
        let mut tss = tss::TaskStateSegment::new();
        tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = {
            const STACK_SIZE: usize = 4096 * 5;
            // This needs to be a static mut. If this was immutable, the bootloader would make it
            // a read only page.
            static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];

            let stack_start = VirtAddr::from_ptr(&raw const STACK);
            let stack_end = stack_start + STACK_SIZE;
            stack_end
        };
        tss
    };
}

pub fn init() {
    GDT.table.load();

    unsafe {
        // We changed our GDT, so we should reload the code segment register. This is required
        // since the old segment selector could now point to a different GDT descriptor.
        CS::set_reg(GDT.selectors.code_selector);

        // We loaded a GDT that contains a TSS selector, but we still need to tell the CPU that it
        // should use that TSS.
        load_tss(GDT.selectors.tss_selector);
    }

    IDT.load();
}

extern "C" fn divide_error_handler(stack_frame: &ExceptionStackFrame) -> ! {
    crate::println!("\nEXCEPTION: DIVIDE BY ZERO\n{:#?}", &*stack_frame);
    loop {}
}

extern "C" fn invalid_opcode_handler(stack_frame: &ExceptionStackFrame) -> ! {
    crate::println!("\nEXCEPTION: INVALID OPCODE\n{:#?}", &*stack_frame);
    loop {}
}

extern "C" fn breakpoint_interrupt_handler(stack_frame: &ExceptionStackFrame) {
    crate::println!("\nEXCEPTION: BREAKPOINT\n{:#?}", &*stack_frame);
}

// The double fault error code is always 0. x86 expects the double fault handler to be diverging.
extern "C" fn double_fault_interrupt_handler(
    stack_frame: &ExceptionStackFrame,
    error_code: u64,
) -> ! {
    crate::println!(
        "\nEXCEPTION: DOUBLE FAULT with error code {:?}\n{:#?}",
        error_code,
        &*stack_frame
    );

    loop {}
}
