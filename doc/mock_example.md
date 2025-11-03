### Mock GPIO example

The following example shows a minimal, host-friendly mock implementation of
the register block, a `GpioRegisters` implementation, and a `Bank` type. It
is illustrative — adapt it to your hardware.

```rust
use mayio::{Io, Level, Bank, IoDir, GpioRegisters, Input, Output, Interrupt, PushPull};

// A tiny mock of the register block. On real hardware this would be 
// a struct with volatile register accessors.
#[repr(C)]
pub struct MyGpioRegs {
    input: u32,
    output: u32,
    dir: u32,
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
        /// Nothing to do
        ()
    }

    fn read(ptr: *const Self, pin: u32) -> Level {
        // SAFETY: the pointer originates from a Rust value; converting it to
        // a reference for reading is valid when the memory is initialized and
        // aligned. Use volatile accessors on real hardware as needed.
        let regs = unsafe { &*ptr };
        if (regs.input & (1 << pin)) != 0 {
            Level::High
        }
        else {
            Level::Low
        }
    }

    fn write(ptr: *mut Self, pin: u32, level: Level) {
        // SAFETY: see notes above — the pointer originates from a Rust value
        // and converting it to a mutable reference is valid when alignment,
        // initialization and exclusivity requirements are met.
        let regs = unsafe { &mut *ptr };
        let output = regs.output;
         match level {
            Level::High => regs.output = output | (1 << pin),
            Level::Low => regs.output = output & (u32::MAX ^ (1 << pin))
        }
    }

    fn interrupt_pending(ptr: *mut Self, pin: u32) -> bool {
        false
    }
}

// Provide a `Bank` implementation that returns a (mock) base address.
pub struct MyBank;

impl Bank<MyGpioRegs> for MyBank {
    // Unused here
    const BASE_ADDRESS: usize = 0;

    fn addr() -> *mut MyGpioRegs {
        // In real hardware this would be a fixed peripheral address. For a
        // host mock you could point to a static instance.
        static mut MOCK_REGS: MyGpioRegs = MyGpioRegs {
            input: 0,
            output: 0,
            dir: 0,
        };

        unsafe { &raw mut MOCK_REGS }
    }
}

// Once Bank and GpioRegisters are implemented, a type alias is to be used 
// as it is more ergonomic
type Pin<const N: u32, D> = Io<MyBank, N, MyGpioRegs, D>;

// Usage
// At init, output should not be set
let mut out = Pin::<3, Output<PushPull>>::init();
assert_eq!(unsafe { (*MyBank::addr()).output & (1 << 3) }, 0);

// Activate 
out.activate();
assert_eq!(unsafe { (*MyBank::addr()).output & (1 << 3) }, 8);


// Assert that the output register bit for pin 3 was unset by the driver.
// We read the mock register directly via the bank address returned by
// `MyBank::addr()`; this mirrors what real hardware would contain.
out.deactivate();
assert_eq!(unsafe { (*MyBank::addr()).output & (1 << 3) }, 0);

// Prepare the input register for pin 4 and verify the typed API reads it.
unsafe { (*MyBank::addr()).input = 1 << 4; }

// Usage
let input = Pin::<4, Input>::init();
let level = input.read();

// Final check: ensure the typed API reports the pin as `PushPull`.
// If this assertion fails, the example/driver did not set the mock input
// register as expected.
assert!(matches!(level, Level::High));
```
