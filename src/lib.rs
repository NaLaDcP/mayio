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
    use crate::DefaultState;

    // Sealed trait to prevent external implementations of `Direction`.
    pub trait Sealed {}
    impl Sealed for super::Input {}
    impl<S: DefaultState> Sealed for super::Output<S> {}
    impl Sealed for super::Active {}
    impl Sealed for super::Inactive {}
}

use self::private::Sealed;

/// Trait implemented by direction marker types (`Input`, `Output`).
///
/// This trait is sealed to keep direction implementations local to the
/// crate and to allow the API to rely on the two known directions.
pub trait Direction: Sealed {
    /// Set the hardware direction for `pin` on the provided GPIO bank handle.
    fn init(gpio: &mut Gpio<impl GpioRegisters>, pin: u32);
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
pub trait DefaultState: Sealed {
    fn default_state() -> Level;
}
/// Marker type for an input pin.
pub struct Input;

pub struct Active;
impl DefaultState for Active {
    fn default_state() -> Level {
        Level::High
    }
}

pub struct Inactive;
impl DefaultState for Inactive {
    fn default_state() -> Level {
        Level::Low
    }
}
/// Marker type for an output pin.
pub struct Output<S: DefaultState> {
    default: PhantomData<fn() -> S>,
}

impl Direction for Input {
    fn init(gpio: &mut Gpio<impl GpioRegisters>, pin: u32) {
        gpio.set_dir(pin, IoDir::In);
        gpio.set_interrupt(pin, Interrupt::Off);
    }
}
impl<S: DefaultState> Direction for Output<S> {
    fn init(gpio: &mut Gpio<impl GpioRegisters>, pin: u32) {
        gpio.set_dir(pin, IoDir::Out);
        gpio.write(pin, <S as DefaultState>::default_state());
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
        D::init(&mut bank, N);
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
        bank.read(N)
    }
}

impl<B, R, const N: u32, S: DefaultState> Io<N, B, R, Output<S>>
where
    B: Bank<R>,
    R: GpioRegisters,
{
    /// Write a logical level to the pin.
    fn write(&mut self, level: Level) {
        let mut bank = <B as Bank<R>>::get_handle();
        bank.write(N, level);
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
