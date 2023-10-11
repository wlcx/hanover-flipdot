# A Rust implementation of the Hanover Displays Flipdot serial protocol

A basic implementation of the serial protocol used by Hanover Displays flipdots.

## `embedded-graphics` support
`HanoverFlipdot` implements the `DrawTarget` trait from [embedded-graphics]. This means you can easily draw text and other 2d graphics using that ecosystem.

[embedded-graphics]: https://docs.rs/embedded-graphics