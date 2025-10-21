# mayo

mayo is a small no_std Rust crate that provides typed GPIO abstractions.

The crate offers a type-safe API for working with GPIO pins using compile-time
direction markers (`Input` / `Output`) and a bank/register abstraction exposed
by the `low` module.

## Goals

- Small, zero-std dependency surface for embedded contexts.
- Type-level safety to prevent writing to input pins and similar mistakes.
- Minimal runtime overhead.

### Mock GPIO example

The concrete mock example is maintained in `doc/mock_example.md` and is
included in the crate documentation. To view the full example, open the
file directly:

[Mock example](doc/mock_example.md)

If you prefer the example embedded in this README, tell me and I can inline
it here instead.

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

This project is licensed under the MIT License. 
