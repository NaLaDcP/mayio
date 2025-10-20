//! Low-level GPIO building blocks used by the `ayo` crate.
//!
//! This module defines a small, platform-facing API that the higher-level
//! typed abstractions use. It intentionally exposes a minimal, unsafe
//! boundary (raw pointers to register blocks) so platform crates can provide
//! zero-cost wrappers over their hardware register maps.
//!
//! The documentation below includes short examples showing how a platform
//! crate might implement the required traits. The examples are illustrative
//! and marked `no_run` to avoid being executed as doctests — they should be
//! adapted to your MCU's actual register layout (for example an svd2rust
//! generated register block).
//!
//! Safety notes:
//! - The `Bank::addr()` pointer must be valid and point to the correct
//!   register block for the lifetime of operations. Dereferencing an invalid
//!   pointer is undefined behavior.
//! - Implementations should use volatile reads/writes (or generated accessors)
//!   to avoid compiler reordering and ensure side effects reach the hardware.
//! - Concurrent access (from interrupts or multiple cores) must be handled
//!   by the platform crate (e.g. with critical sections or atomic/lock
//!   mechanisms) if required by the hardware.

/// Abstraction representing a GPIO bank on the target platform.
///
/// A concrete bank type (provided by a platform-specific crate) must
/// implement `Bank<R>` for its register block `R`. The trait provides the
/// physical address of the registers and a convenience `get_handle()` that
/// returns an `io::Gpio<R>` wrapper around the register pointer.
pub trait Bank<R: register::GpioRegisters> {
    /// Return a new `Gpio` handle pointing at the bank's register block.
    ///
    /// This default implementation calls `Self::addr()` and constructs the
    /// thin `io::Gpio` wrapper. Platform crates may rely on this helper
    /// to obtain a typed handle.
    fn get_handle() -> io::Gpio<R> {
        io::Gpio::new(Self::addr())
    }

    /// Return the base pointer to the register block for this bank.
    fn addr() -> *mut R;
}

// Example: providing a `Bank` for `MyGpioRegs`.
//
// ```no_run
// pub struct MyBank;
//
// impl Bank<MyGpioRegs> for MyBank {
//     fn addr() -> *mut MyGpioRegs {
//         0x4002_0000 as *mut MyGpioRegs // platform-specific base address
//     }
// }
// ```

/// Register-level trait describing the operations a GPIO register block
/// must provide for `ayo` to operate.
///
/// Platform-specific register types should implement `GpioRegisters` to
/// expose a small set of operations used by the higher-level API. The
/// methods are kept simple and raw (u32 masks, pin indices) so they map
/// directly onto common hardware register patterns.
pub mod register {
    use crate::{Interrupt, IoDir};

    /// Represents the hardware register interface for a GPIO bank.
    ///
    /// Implementers must ensure that these functions perform the expected
    /// side effects on the hardware registers. The trait is intentionally
    /// small to make it easy to adapt to different MCUs or SoCs.
    pub unsafe trait GpioRegisters {
        /// Set the direction of a single pin.
        fn set_dir(ptr: *mut Self, pin: u32, dir: IoDir);

        /// Configure the interrupt mode for a single pin.
        fn set_interrupt(ptr: *mut Self, pin: u32, interrupt: Interrupt);

        /// Read the input register(s) for the bank; returns a raw bitmask.
        fn read(ptr: *mut Self) -> u32;

        /// Write to the output register(s) using a bitmask.
        fn write(ptr: *mut Self, mask: u32);
    }
}

/// Thin, unsafe wrapper around a raw pointer to a register block.
///
/// `io::Gpio<R>` stores a raw pointer to `R` and provides a small set of
/// safe(er) methods that call into the implementor's `GpioRegisters`
/// functions using `unsafe` internally. This isolates unsafe pointer
/// dereferencing in a single place.
pub mod io {
    use super::register::GpioRegisters;
    use crate::{Interrupt, IoDir};

    /// Opaque handle to a GPIO register block.
    ///
    /// The handle contains a raw pointer to the register block type `R`.
    /// Callers must ensure the pointer is valid for the lifetime of the
    /// operations. All methods on `Gpio` use `unsafe` internally to access
    /// the pointer and forward to the `GpioRegisters` implementation.
    pub struct Gpio<R: GpioRegisters> {
        registers: *mut R,
    }

    impl<R> Gpio<R>
    where
        R: GpioRegisters,
    {
        /// Create a new `Gpio` wrapper from a raw register pointer.
        ///
        /// This is `pub(super)` because only the crate (and implementors)
        /// should construct `Gpio` handles from raw addresses; external
        /// crates should provide a `Bank` implementation instead.
        pub(super) fn new(registers: *mut R) -> Self {
            Self { registers }
        }

        /// Set the direction for `pin` by delegating to the register impl.
        #[inline]
        pub fn set_dir(&mut self, pin: u32, dir: IoDir) {
            <R as GpioRegisters>::set_dir(self.registers, pin, dir);
        }

        /// Configure the interrupt mode for `pin`.
        #[inline]
        pub fn set_interrupt(&mut self, pin: u32, interrupt: Interrupt) {
            <R as GpioRegisters>::set_interrupt(self.registers, pin, interrupt);
        }

        /// Read the current input state bitmask for the bank.
        #[inline]
        pub fn read(&self) -> u32 {
            <R as GpioRegisters>::read(self.registers)
        }

        /// Write the provided `mask` to the bank output register(s).
        #[inline]
        pub fn write(&mut self, mask: u32) {
            <R as GpioRegisters>::write(self.registers, mask);
        }
    }
}
