#![no_std] // Do not link the Rust stdlib
#![cfg_attr(test, no_main)] // Disable all Rust-level entry points
// Enable custom_test_frameworks
#![feature(custom_test_frameworks)] 
// Set the test_runner function
#![test_runner(crate::test_runner)] 
// Change the name of the generated function (for running tests)
// to something different than main
#![reexport_test_harness_main = "test_main"]
#![feature(abi_x86_interrupt)]

pub mod vga_buffer;
pub mod serial;
pub mod interrupts;
pub mod gdt;

use core::panic::PanicInfo;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
// Represented as u32, since the port size is four bytes
#[repr(u32)]
pub enum QemuExitCode {
    Success = 0x10,
    Failed = 0x11,
}

/// Initialize the GDT and IDT
pub fn init() {
    gdt::init();
    interrupts::init_idt();
    // Initialize PICS
    unsafe { interrupts::PICS.lock().initialize() };
    // Enable interupts
    x86_64::instructions::interrupts::enable();
}

pub fn exit_qemu(exit_code: QemuExitCode) {
    use x86_64::instructions::port::Port;

    unsafe {
        // Create io port with the port number 0xf4 since we have
        // a device (isa-debug-exit) on that port in QEMU
        let mut port = Port::new(0xf4);
        port.write(exit_code as u32);
    }
}

pub trait Testable {
    fn run(&self) -> ();
}

impl<T> Testable for T
where
    T: Fn(),
{
    fn run(&self) -> () {
        // Print the function name
        serial_print!("{}...\t", core::any::type_name::<T>());
        self();
        serial_println!("[OK]");
    }
}

/// This function runs tests.
pub fn test_runner(tests: &[&dyn Testable]) {
    serial_println!("Running {} tests", tests.len());
    for test in tests {
        test.run();
    }
    exit_qemu(QemuExitCode::Success);
}

pub fn test_panic_handler(info: &PanicInfo) -> ! {
    serial_println!("[failed]\n");
    serial_println!("Error: {}\n", info);
    exit_qemu(QemuExitCode::Failed);
    loop {}
}

#[cfg(test)]
#[panic_handler]
/// This function is called on panic when in test mode.
fn panic(info: &PanicInfo) -> ! {
    test_panic_handler(info);
}

#[cfg(test)]
#[no_mangle]
pub extern "C" fn _start() -> ! {
    init();
    test_main();
    loop {}
}