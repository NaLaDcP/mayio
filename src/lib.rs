//! Ayo — typed GPIO abstractions for no_std environments.
//!
//! This crate provides a small, dependance-free, type-safe API to manage GPIO pins using
//! compile-time direction markers (Input/Output) and provided bank/register
//! abstractions.
//!
//! Detailed example (mocked)
//! The example below is included from an external markdown file and shows a
//! mocked register block, `GpioRegisters` implementation and `Bank` type.
#![doc = include_str!("../doc/mock_example.md")]
#![no_std]

mod low;

use core::marker::PhantomData;
pub use low::{Bank, io::Gpio, register::GpioRegisters};

mod private {
    // Sealed trait to prevent external implementations of `Direction`.
    pub trait Sealed {}
    impl Sealed for super::Input {}
    impl Sealed for super::Output {}
}

use self::private::Sealed;

/// Trait implemented by direction marker types (`Input`, `Output`).
///
/// This trait is sealed to keep direction implementations local to the
/// crate and to allow the API to rely on the two known directions.
pub trait Direction: Sealed {
    /// Set the hardware direction for `pin` on the provided GPIO bank handle.
    fn set_dir(gpio: &mut Gpio<impl GpioRegisters>, pin: u32);
}

/// Interrupt configuration for a GPIO pin.
pub enum Interrupt {
    /// Disable interrupts for the pin.
    Off,
    /// Interrupt on rising edge.
    RisingEdge,
    /// Interrupt on falling edge. (Typo in original name preserved.)
    FallingEgdge,
    /// Interrupt while the pin is low.
    Low,
    /// Interrupt while the pin is high.
    High,
}

/// Logical level of a GPIO pin.
pub enum Level {
    /// Logical low / 0.
    Low,
    /// Logical high / 1.
    High,
}

/// Direction for a single GPIO pin.
///
/// The value is forwarded to the platform register implementation which is
/// responsible for applying the direction in hardware.
pub enum IoDir {
    /// Configure the pin as input.
    In,
    /// Configure the pin as output.
    Out,
}

/// Typed GPIO pin handle.
///
/// Generic parameters:
/// - `N`: constant pin index within the bank.
/// - `B`: bank type which implements `Bank<R>`.
/// - `R`: register block implementing `GpioRegisters`.
/// - `D`: direction marker type (`Input` or `Output`).
pub struct Io<const N: u32, B, R, D>
where
    B: Bank<R>,
    R: GpioRegisters,
{
    dir: PhantomData<fn() -> D>,
    bank: PhantomData<fn() -> B>,
    register: PhantomData<fn() -> R>,
}

/// Marker type for an input pin.
pub struct Input;
/// Marker type for an output pin.
pub struct Output;

impl Direction for Input {
    fn set_dir(gpio: &mut Gpio<impl GpioRegisters>, pin: u32) {
        gpio.set_dir(pin, IoDir::In);
    }
}
impl Direction for Output {
    fn set_dir(gpio: &mut Gpio<impl GpioRegisters>, pin: u32) {
        gpio.set_dir(pin, IoDir::Out);
    }
}

impl<B, R, const N: u32, D> Io<N, B, R, D>
where
    B: Bank<R>,
    R: GpioRegisters,
    D: Direction,
{
    /// Initialize the typed IO for pin `N`.
    ///
    /// This configures the hardware direction using the marker types `Input` and `Output`.
    pub fn init() -> Self {
        let mut bank = <B as Bank<R>>::get_handle();
        <Input as Direction>::set_dir(&mut bank, N);
        Self {
            dir: PhantomData,
            bank: PhantomData,
            register: PhantomData,
        }
    }
}
impl<B, R, const N: u32> Io<N, B, R, Input>
where
    B: Bank<R>,
    R: GpioRegisters,
{
    /// Set the interrupt configuration for this input pin.
    pub fn set_interrupt(&mut self, interrupt: Interrupt) {
        let mut bank = <B as Bank<R>>::get_handle();
        bank.set_interrupt(N, interrupt);
    }

    /// Read the current logical level of the pin.
    pub fn read(&self) -> Level {
        let bank = <B as Bank<R>>::get_handle();
        let value = bank.read();
        if (value & (1 << N)) != 0 {
            Level::High
        } else {
            Level::Low
        }
    }
}

impl<B, R, const N: u32> Io<N, B, R, Output>
where
    B: Bank<R>,
    R: GpioRegisters,
{
    /// Write a logical level to the pin.
    fn write(&mut self, level: Level) {
        let mut bank = <B as Bank<R>>::get_handle();
        let mask = match level {
            Level::Low => u32::MAX ^ (1 << N),
            Level::High => 1 << N,
        };
        bank.write(mask);
    }

    /// Drive the pin low.
    #[inline]
    pub fn set_low(&mut self) {
        self.write(Level::Low);
    }

    /// Drive the pin high.
    #[inline]
    pub fn set_high(&mut self) {
        self.write(Level::High);
    }
}
