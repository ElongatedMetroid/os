use pic8259::ChainedPics;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode};
use lazy_static::lazy_static;
use crate::{println, print, gdt, hlt_loop};

/// The default configuration of the PICs is not usable because it sends interrupt
/// vector numbers in the range of 0–15 to the CPU. These numbers are already 
/// occupied by CPU exceptions. For example, number 8 corresponds to a double 
/// fault. The actual range doesn’t matter as long as it does not overlap with 
/// the exceptions, but typically the range of 32–47 is chosen, because these are
/// the first free numbers after the 32 exception slots.
pub const PIC_1_OFFSET: u8 = 32;
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
///                      ____________                          ____________
/// Real Time Clock --> |            |   Timer -------------> |            |
/// ACPI -------------> |            |   Keyboard-----------> |            |      _____
/// Available --------> | Secondary  |----------------------> | Primary    |     |     |
/// Available --------> | Interrupt  |   Serial Port 2 -----> | Interrupt  |---> | CPU |
/// Mouse ------------> | Controller |   Serial Port 1 -----> | Controller |     |_____|
/// Co-Processor -----> |            |   Parallel Port 2/3 -> |            |
/// Primary ATA ------> |            |   Floppy disk -------> |            |
/// Secondary ATA ----> |____________|   Parallel Port 1----> |____________|
pub enum InterruptIndex {
    Timer = PIC_1_OFFSET,
    Keyboard,
}

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();

        idt.breakpoint.set_handler_fn(breakpoint_handler);
        unsafe {
            idt.double_fault.set_handler_fn(double_fault_handler)
                // set the stack the double_fault exception will use
                .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
        }
        idt.page_fault.set_handler_fn(page_fault_handler);
        // Add handler function for the timer interrupt
        idt[InterruptIndex::Timer as usize]
            .set_handler_fn(timer_interrupt_handler);
        // Add handler function for keyboard interrupt
        idt[InterruptIndex::Keyboard as usize]
            .set_handler_fn(keyboard_interrupt_handler);

        idt
    };
}

pub static PICS: spin::Mutex<ChainedPics> = 
    spin::Mutex::new(unsafe { 
        ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) 
    });

pub fn init_idt() {
    IDT.load();
}

/// Handler for the breakpoint exception, pause a program when the breakpoint
/// instruction int3 is executed.
extern "x86-interrupt" fn breakpoint_handler(
    stack_frame: InterruptStackFrame
) {
    println!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn double_fault_handler(
    stack_frame: InterruptStackFrame, _error_code: u64
) -> ! {
    panic!("EXCEPTION: DOUBLE FAULT\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn page_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: PageFaultErrorCode,
) {
    use x86_64::registers::control::Cr2;

    println!("EXCEPTION: PAGE FAULT");
    // The CR2 register is automatically set by the CPU on a 
    // page fault and contains the accessed virtual address that
    // caused the page fault.
    println!("Accessed Address: {:?}", Cr2::read());
    println!("Error Code: {:?}", error_code);
    println!("{:#?}", stack_frame);
    hlt_loop();
}

extern "x86-interrupt" fn timer_interrupt_handler(
    _stack_frame: InterruptStackFrame
) {
    print!(".");

    // Notify the controller that the interrupt was processed and that the system
    // is ready to recieve the next interrupt.
    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Timer as u8);
    }
}

extern "x86-interrupt" fn keyboard_interrupt_handler(
    _stack_frame: InterruptStackFrame
) {
    use pc_keyboard::{layouts, DecodedKey, HandleControl, Keyboard, ScancodeSet1};
    use spin::Mutex;
    use x86_64::instructions::port::Port;

    lazy_static! {
        static ref KEYBOARD: Mutex<Keyboard<layouts::Us104Key, ScancodeSet1>> = 
            Mutex::new(Keyboard::new(layouts::Us104Key, ScancodeSet1, HandleControl::Ignore));
    }

    // Lock the mutex
    let mut keyboard = KEYBOARD.lock();
    let mut port = Port::new(0x60);
    // Read a byte from the keyboards data port (the scancodess)
    let scancode: u8 = unsafe { port.read() };
    // Pass the scancode to the add_byte method, which will 
    // translate the scancode into an Option<KeyEvent>, the
    // KeyEvent contains the key which caused the event and if
    // it was a press or release event.
    if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
        // Produce a DecodedKey from a KeyEvent
        if let Some(key) = keyboard.process_keyevent(key_event) {
            match key {
                DecodedKey::Unicode(character) => print!("{}", character),
                DecodedKey::RawKey(key) => print!("{:?}", key),
            }
        }
    }

    // Notify the controller that the interrupt was processed and that the system
    // is ready to recieve the next interrupt.
    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Keyboard as u8);
    }
}

#[test_case]
fn test_breakpoint_exception() {
    x86_64::instructions::interrupts::int3();
}