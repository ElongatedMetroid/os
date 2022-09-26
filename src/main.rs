#![no_std] // Do not link the Rust stdlib
#![no_main] // Disable all Rust-level entry points

use core::panic::PanicInfo;

/// This function is called on panic.
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

static HELLO: &[u8] = b"Hello World";

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

    // VGA buffer starts at 0xb8000
    let vga_buffer = 0xb8000 as *mut u8;

    // Foreach byte in HELLO
    for (i, &byte) in HELLO.iter().enumerate() {
        unsafe {
            // Write the string byte
            *vga_buffer.offset(i as isize * 2) = byte;
            // Write the color byte (i + 1 will be the colors)
            *vga_buffer.offset(i as isize * 2 + 1) = i as u8 + 1;
        }
    }

    loop {}
}