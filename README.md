# Ayo

Ayo is a small no_std Rust crate that provides typed GPIO abstractions.

The crate offers a type-safe API for working with GPIO pins using compile-time
direction markers (`Input` / `Output`) and a bank/register abstraction exposed
by the `low` module.

## Goals

- Small, zero-std dependency surface for embedded contexts.
- Type-level safety to prevent writing to input pins and similar mistakes.
- Minimal runtime overhead.

## Quick example

The API is generic over a bank type `B` and register block `R` provided by the
platform-specific `low` implementation. The following is a conceptual example:

```rust
use ayo::{Io, Level, Input, Output};

// Create a typed GPIO for pin 3 as an output.
let mut out: Io::<3, MyBank, MyRegs, Output> = Io::init();
out.set_high();

// Create a typed GPIO for pin 4 as an input.
let input: Io::<4, MyBank, MyRegs, Input> = Io::init();
let level = input.read();
match level {
    Level::High => { /* ... */ }
    Level::Low => { /* ... */ }
}
```

Note: `MyBank` and `MyRegs` are platform-specific and must implement the
traits re-exported from the `low` module (`Bank<R>` and `GpioRegisters`).

### Concrete example (mocked)

The following example shows a minimal, host-friendly mock implementation of
the register block, a `GpioRegisters` implementation, and a `Bank` type. It
is illustrative — adapt it to your hardware (use svd2rust types or volatile
accessors for real MCUs).

```rust
use ayo::{Io, Level, Bank, IoDir, GpioRegisters, Input, Output, Interrupt};

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
        // SAFETY: caller must ensure `ptr` is valid and properly aligned for
        // `MyGpioRegs`. This implementation uses `unsafe` to obtain a
        // mutable reference from the raw pointer because the `Bank::addr()`
        // contract guarantees the pointer points to a valid register block.
        let regs = unsafe { &mut *ptr };
        match dir {
            IoDir::In => regs.dir &= !(1 << pin),
            IoDir::Out => regs.dir |= 1 << pin,
        }
    }

    fn set_interrupt(ptr: *mut Self, pin: u32, interrupt: Interrupt) {
        // SAFETY: same contract as above — `ptr` must be a valid pointer to
        // `MyGpioRegs` and properly aligned.
        let regs = unsafe { &mut *ptr };
        // naive mapping for illustration
        regs.intcfg = (regs.intcfg & !(0b11 << (pin * 2))) | ((interrupt as u32) << (pin * 2));
    }

    fn read(ptr: *mut Self) -> u32 {
        // SAFETY: reading from a raw pointer is unsafe; the implementor must
        // ensure `ptr` points to initialized memory representing the register
        // block.
        let regs = unsafe { &*ptr };
        regs.input
    }

    fn write(ptr: *mut Self, mask: u32) {
        // SAFETY: see notes above — `ptr` must be valid/aligned and the
        // caller must ensure exclusive access if required by the hardware.
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
        static mut MOCK_REGS: MyGpioRegs = MyGpioRegs { input: 0, output: 0, dir: 0, intcfg: 0 };
        unsafe { &mut MOCK_REGS }
    }
}

// Usage
let mut out: Io::<3, MyBank, MyGpioRegs, Output> = Io::init();
out.set_high();

let input: Io::<4, MyBank, MyGpioRegs, Input> = Io::init();
let level = input.read();
match level {
    Level::High => { /* ... */ }
    Level::Low => { /* ... */ }
}
```


## API summary

- `Io<const N, B, R, D>` — typed handle for a GPIO pin.
- `Input` / `Output` — marker types used as the direction parameter `D`.
- `Io::init()` — initializes the pin and configures its hardware direction to input by default.
- `Io::<N, B, R, Input>::read()` — read the logical level.
- `Io::<N, B, R, Input>::set_interrupt()` — configure interrupts for the input pin.
- `Io::<N, B, R, Output>::set_high()` / `set_low()` — drive the output pin.

## Building

This crate is intended for `no_std` embedded targets. To run `cargo build`, use
an appropriate target and the platform's toolchain.

## License

(This project currently has no license file in the repository. Add one if you
intend to publish.)
