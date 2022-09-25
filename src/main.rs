#![no_std]
#![no_main]

use core::panic::PanicInfo;

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
    loop {}
}