# rv32-emu

A small RISC-V (RV32I) emulator written in Rust. It fetches, decodes, and executes RV32I instructions, and exists mainly as a reference model to check my [Kestrel](https://github.com/humoudalharthy/kestrel) CPU core against.

## Why

Kestrel is a superscalar, out-of-order CPU I'm building in Verilog. It's hard to know if an out-of-order core is correct just by reading the RTL, so I built this emulator to act as an answer key: run the same program on both, and if the final register values differ, Kestrel has a bug. This is the same idea behind reference models used in real chip verification.

Building it from scratch was also a good way to properly learn the RISC-V instruction encoding rather than just reading about it.

## What it does

- Implements a subset of the RV32I base integer instruction set:
  - `LUI`, `JAL`
  - `BEQ`, `BNE`
  - `LW`, `SW`
  - `ADDI`
  - `ADD`, `SUB`
  - `ECALL` (used as a halt signal)
- Decodes raw 32-bit instructions by hand (no external RISC-V crate), pulling out opcode, registers, and sign-extended immediates for each instruction format (R, I, S, B, U, J).
- Ships with a tiny in-Rust "assembler" (`addi`, `add`, `bne`, etc.) so test programs can be written directly in `main.rs` instead of hand-assembling machine code.

## Example

The included program sums 10 down to 1 in a loop and halts:

```rust
let program = [
    addi(11, 0, 10),  // x11 = 10
    add(10, 10, 11),  // loop: x10 += x11
    addi(11, 11, -1), // x11 -= 1
    bne(11, 0, -8),   // if x11 != 0, jump back to loop
    ecall(),          // halt
];
```

Running it prints: halted after 31 instructions
                    x10 = 55


## Running it

Requires [Rust](https://rustup.rs).

```bash
git clone https://github.com/YOUR_USERNAME/rv32-emu.git
cd rv32-emu
cargo run
```

## How it works

- `Cpu` holds 32 general-purpose registers, a program counter (`pc`), and a flat byte array as memory.
- `step()` runs one fetch-decode-execute cycle: read the 32-bit instruction at `pc`, pull out its fields, match on the opcode, perform the operation, and advance `pc` (unless a jump or branch overrides it).
- `x0` is hardwired to zero, matching the RISC-V spec — writes to it are silently dropped.
- Loads and stores work on a `Vec<u8>` memory using little-endian byte order, as RISC-V specifies.

## Status / roadmap

This is an early, work-in-progress project. Planned next steps:

- [ ] Remaining RV32I instructions: `AND`, `OR`, `XOR`, shifts, `SLT`/`SLTU`, `LB`/`LH`/`SB`/`SH`, `JALR`, remaining branches (`BLT`, `BGE`, etc.)
- [ ] Unit tests per instruction (`cargo test`)
- [ ] Load programs from a raw binary or ELF file instead of hardcoding them in Rust
- [ ] Per-instruction execution trace, to diff against Kestrel's simulation output
- [ ] Basic CSR / exception support

## License

MIT
