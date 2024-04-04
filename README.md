# YARGEM : Yet Another Rust Gameboy Emulator
Very early gameboy emulator written in rust. This repo is purely educational to familiarize myself with Rust and learn how emulation works with a relatively *simple* device to emulate.

Currently, all opcodes *should* be implemented but there are probably tons of untested bugs at the moment. 
The emulator backend is a simple match/case interpreter inspired by [Nekronos's rust emulator](https://github.com/nekronos/gbc_rs/tree/master), [P4ddy1's emulator](https://github.com/p4ddy1/gbemulator/tree/master) and [YushiOMOTE's rgy](https://github.com/YushiOMOTE/rgy/tree/master).

As of the latest commit, no MBC has been implemented, but games that don't use MBCs (Tetris, Dr. Mario...) can be loaded on the emulator. Blaarg CPU instruction tests were all tested and all passed.
