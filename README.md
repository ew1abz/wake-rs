# wake-rs

`wake-rs` is a library written in Rust for encoding/decoding Wake protocol.

`Wake` is a serial communication protocol highly optimized for **microcontrollers**. It based on SLIP protocol (<https://datatracker.ietf.org/doc/html/rfc1055>).

![debug_print](images/debug_print.png)

## Main features

- unique start symbol
- 7-bit addressing (optional)
- CRC (8 or 16 bits)
- low overhead

The protocol doesn't support:

- ~~error correction~~
- ~~compression~~

Frame structure:

![Frame structure](images/wake.png)

## Integrations

There are many architecture-specific implementations:

- MCS-51
- AVR
- STM32
- x86

in many languages:

- C
- C++
- C#
- Python
- Rust

## Examples

1. Demo - basic usage
2. Serial - how to use with serial port
3. Relay shield - real device communication

## Build

### Library

```bash
cargo build --release
```

### Examples

```bash
cargo build --examples
```

## Resources

Protocol description, libraries, and tools: <http://www.leoniv.diod.club/articles/wake/wake.html>

## TODO

- Use this library with a microcontroller (Rust project)
- Add a stream decoder (one byte per time with internal buffer)

## License

Code released under the MIT License.
