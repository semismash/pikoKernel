use core::panic::PanicInfo;
use core::arch::asm;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    crate::sys::console::clear!();
    crate::sys::console::println!("You are seeing this message as the OS panicked!", crate::drivers::display_old::ForegroundColor::Red);
    crate::sys::console::write_and_flush!("Panic message: {}", info.message());
    loop {
        unsafe {
            asm!("cli", "hlt", options(nomem, nostack, preserves_flags));
        }
    }
}