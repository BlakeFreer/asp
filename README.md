# ASP

An assembler (and disassembler) for stepper motor ASIP code, written in Rust.

- [Installation](#installation)
- [Usage](#usage)
  - [Assembly to MIF](#assembly-to-mif)
  - [Assembly to HEX](#assembly-to-hex)
  - [HEX to Assembly](#hex-to-assembly)
  - [Example](#example)
- [Assembly Commands](#assembly-commands)

## Installation

Requires [`cargo`](https://doc.rust-lang.org/cargo/getting-started/installation.html).

```bash
cargo install --git
```

## Usage

### Assembly to MIF

```bash
$ asp file.s
Output saved to out.mif
```

Add `-o <FILENAME>` to change the output file.

### Assembly to HEX

Use `-f` or `--fmt` to change the output format.

```bash
$ asp file.s -f hex
Output saved to out.hex
```

You can use `xxd -b out.hex` to view the raw machine code.

### HEX to Assembly

```bash
$ asp --hex file.hex -f asm
Output saved to out.asm
```

### Example

Download the `example.s` and `example.hex` files. They represent the same program written in assembly and machine code.

```bash
$ asp example.s -f hex
Output saved to out.hex
$ diff out.hex example.hex -s
Files out.hex and example.hex are identical
```

```bash
$ asp --hex example.hex -f asm
Output saved to out.s
$ diff out.s example.s -s
Files out.s and example.s are identical
```

## Assembly Commands

There are 4 registers `r0 r1 r2 r3`:

- `r0`, `r1`: General purpose
- `r2` Stepper motor position
- `r3` Delay period

The immediates are either signed `In` or unsigned `Un`, where `n` is the number of bits.

Comments start with a semicolon.

```asm
BR i        ; jump i (I5) ops 
BRZ i       ; jump i (I5) ops if r0 == 0, else continue
ADDI rx, u  ; add u (U3) to reg rx
SUBI rx, u  ; subtract (U3) from reg rx
SR0 u       ; set the lower 4 bits of r0 to u (U4)
SRH0 u      ; set the upper 4 bits of r0 to u (U4)
CLR rx      ; clear reg rx
MOV rd, rs  ; copy the content of reg rs into rd
MOVR rx     ; move  the number of full-steps specified by rx
MOVRHS rx   ; move the number of half-steps specified by rx
MOVA rx     ; move to the absolute position specified in rx
PAUSE       ; wait for the amount of time specified by r3
```
