//! Ayo — typed GPIO abstractions for no_std environments.
//!
//! This crate provides a small, dependency-free, zero-cost, type-safe API to manage GPIO
//! pins using compile-time direction markers (`Input`/`Output`) and
//! platform `Bank`/`GpioRegisters` abstractions.
#![doc = include_str!("../doc/mock_example.md")]
#![no_std]

mod low;

use core::{marker::PhantomData, ops::Not};
pub use low::{Bank, io::Gpio, register::GpioRegisters};

mod private {
    use crate::ActiveState;

    // Sealed trait to prevent external implementations of `Direction`.
    pub trait Sealed {}
    impl Sealed for super::Input {}
    impl<S: ActiveState> Sealed for super::Output<S> {}
    impl Sealed for super::High {}
    impl Sealed for super::Low {}
}

use self::private::Sealed;

/// Trait implemented by direction marker types (`Input`, `Output`).
///
/// This trait is sealed to keep direction implementations local to the
/// crate and to allow the API to rely on the two known directions.
#[doc(hidden)]
pub trait Direction: Sealed {
    /// Set the hardware direction for `pin` on the provided GPIO bank handle.
    fn init<R>(gpio: &mut Gpio<R>, pin: u32)
    where
        R: GpioRegisters;
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
#[derive(Copy, Clone, Debug)]
pub enum Level {
    /// Logical low / 0.
    Low,
    /// Logical high / 1.
    High,
}

/// Invert a `Level`.
impl Not for Level {
    type Output = Level;

    fn not(self) -> Self::Output {
        match self {
            Level::Low => Level::High,
            Level::High => Level::Low,
        }
    }
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
pub struct Io<B, const N: u32, R, D>
where
    B: Bank<R>,
    R: GpioRegisters,
{
    dir: PhantomData<fn() -> D>,
    bank: PhantomData<fn() -> B>,
    register: PhantomData<fn() -> R>,
}
/// Trait implemented by types that provide a default output level for
/// output marker types. The `Output<S>` marker uses this to determine the
/// level to drive when the pin is initialized.
#[doc(hidden)]
pub trait ActiveState: Sealed {
    fn active_state() -> Level;
}

/// Marker type for an input pin.
///
/// Use `Io::<N, Bank, Regs, Input>` to obtain a typed input handle. Inputs
/// are initialized with interrupts disabled by default and can be configured
/// via `set_interrupt`.
pub struct Input;

/// Marker type representing a default output state to be high.
///
/// Use as `Output<High>` to request that the pin be driven high when
/// initialized.
pub struct High;
impl ActiveState for High {
    fn active_state() -> Level {
        Level::High
    }
}

/// Marker type representing an default output state to be low.
///
/// Use as `Output<Low>` to request that the pin be driven low when
/// initialized.
pub struct Low;
impl ActiveState for Low {
    fn active_state() -> Level {
        Level::Low
    }
}

/// Marker type for an output pin.
///
/// `Output<S>` carries a phantom type parameter `S` which implements
/// `ActiveState` and selects the level the pin should assume when
/// initialized. Example: `Io::<3, MyBank, MyRegs, Output<Active>>`.
pub struct Output<S: ActiveState> {
    default: PhantomData<fn() -> S>,
}

impl Direction for Input {
    fn init<R>(gpio: &mut Gpio<R>, pin: u32)
    where
        R: GpioRegisters,
    {
        gpio.set_dir(pin, IoDir::In);
        gpio.set_interrupt(pin, Interrupt::Off);
    }
}

impl<S: ActiveState> Direction for Output<S> {
    fn init<R>(gpio: &mut Gpio<R>, pin: u32)
    where
        R: GpioRegisters,
    {
        let active_state = S::active_state();
        gpio.set_dir(pin, IoDir::Out);
        gpio.set_active_state(pin, active_state);
        // Ensure the pin starts low regardless of active state
        gpio.write(pin, Level::Low);
    }
}

impl<B, const N: u32, R, D> Io<B, N, R, D>
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
impl<B, const N: u32, R> Io<B, N, R, Input>
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

impl<B, const N: u32, R, S: ActiveState> Io<B, N, R, Output<S>>
where
    B: Bank<R>,
    R: GpioRegisters,
{
    /// Write a logical level to the pin.
    fn write(&mut self, level: Level) {
        let mut bank = <B as Bank<R>>::get_handle();
        bank.write(N, level);
    }

    /// Activate the pin (drive to active state).
    #[inline]
    pub fn activate(&mut self) {
        self.write(S::active_state());
    }

    /// Deactivate the pin (drive to inactive state).
    #[inline]
    pub fn deactivate(&mut self) {
        self.write(!S::active_state());
    }
}
