### Mock GPIO example

The following example shows a minimal, host-friendly mock implementation of
the register block, a `GpioRegisters` implementation, and a `Bank` type. It
is illustrative — adapt it to your hardware (use svd2rust types or volatile
accessors for real MCUs).

```rust
use ayo::{Io, Level, Bank, IoDir, GpioRegisters, Input, Output, Interrupt, High};

// A tiny mock of the register block. On real hardware this would be the
// svd2rust-generated struct with volatile register accessors.
#[repr(C)]
pub struct MyGpioRegs {
    input: u32,
    output: u32,
    dir: u32,
    intcfg: u32,
}

unsafe impl GpioRegisters for MyGpioRegs {
    fn set_dir(ptr: *mut Self, pin: u32, dir: IoDir) {
        // SAFETY: the pointer was created from a Rust value (for example a
        // `static mut` or by converting a reference to a raw pointer). Because
        // it originates from a Rust object, it is valid to convert the raw
        // pointer back to a reference, provided alignment and initialization
        // are preserved and aliasing/exclusive access rules are respected.
        let regs = unsafe { &mut *ptr };

        match dir {
            IoDir::In => regs.dir &= !(1 << pin),
            IoDir::Out => regs.dir |= 1 << pin,
        }
    }

    fn set_active_state(ptr: *mut Self, pin: u32, level: Level) {
        /// Nothing to do
        ()
    }

    fn set_interrupt(ptr: *mut Self, pin: u32, interrupt: Interrupt) {
        // SAFETY: same as above — the pointer was created from a Rust value
        // so converting it back to a reference is valid when alignment,
        // initialization and aliasing/exclusivity rules are met.
        let regs = unsafe { &mut *ptr };

        // naive mapping for illustration
        regs.intcfg = (regs.intcfg & !(0b11 << (pin * 2)))
            | ((interrupt as u32) << (pin * 2));
    }

    fn read(ptr: *const Self) -> u32 {
        // SAFETY: the pointer originates from a Rust value; converting it to
        // a reference for reading is valid when the memory is initialized and
        // aligned. Use volatile accessors on real hardware as needed.
        let regs = unsafe { &*ptr };
        regs.input
    }

    fn write(ptr: *mut Self, mask: u32) {
        // SAFETY: see notes above — the pointer originates from a Rust value
        // and converting it to a mutable reference is valid when alignment,
        // initialization and exclusivity requirements are met.
        let regs = unsafe { &mut *ptr };
        regs.output = mask;
    }
}

// Provide a `Bank` implementation that returns a (mock) base address.
pub struct MyBank;

impl Bank<MyGpioRegs> for MyBank {
    fn addr() -> *mut MyGpioRegs {
        // In real hardware this would be a fixed peripheral address. For a
        // host mock you could point to a static instance.
        static mut MOCK_REGS: MyGpioRegs = MyGpioRegs {
            input: 0,
            output: 0,
            dir: 0,
            intcfg: 0,
        };

        unsafe { &raw mut MOCK_REGS }
    }
}

// Usage
let mut out: Io::<3, MyBank, MyGpioRegs, Output<High>> = Io::init();
// At init, output should not be set
assert_eq!(unsafe { (*MyBank::addr()).output & (1 << 3) }, 0);
// Active
out.activate();
assert_eq!(unsafe { (*MyBank::addr()).output & (1 << 3) }, 1 << 3);
out.deactivate();

// Assert that the output register bit for pin 3 was unset by the driver.
// We read the mock register directly via the bank address returned by
// `MyBank::addr()`; this mirrors what real hardware would contain.
assert_eq!(unsafe { (*MyBank::addr()).output & (1 << 3) }, 0);

// Prepare the input register for pin 4 and verify the typed API reads it.
unsafe { (*MyBank::addr()).input = 1 << 4; }

// Usage
let input: Io::<4, MyBank, MyGpioRegs, Input> = Io::init();
let level = input.read();

// Final check: ensure the typed API reports the pin as `High`.
// If this assertion fails, the example/driver did not set the mock input
// register as expected.
assert!(matches!(level, Level::High));
```
