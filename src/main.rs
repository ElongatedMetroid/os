#![no_std] // Do not link the Rust stdlib
#![no_main] // Disable all Rust-level entry points

use core::panic::PanicInfo;

mod vga_buffer;

use vga_buffer::Writer;

/// This function is called on panic.
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

// Disable name mangling to ensure that Rust really
// outputs a function with the name _start, without
// the attribute the compiler would generate some 
// random name.
#[no_mangle]
// Mark as `extern "C"` to tell the compiler that 
// it should use the C calling convention for this
// function
pub extern "C" fn _start() -> ! {
    // This function is the entry point, since the
    // linker looks for a function named `_start` 
    // by default.

   Writer::print_something();

    loop {}
}