use core::time;
use std::thread::sleep;

use backend::system::System;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let f_rom = std::fs::read(&args[1]);
    let mut rom = match f_rom {
        Ok(r) => r,
        Err(e) => panic!("Unable to open file {}: {}", args[1], e),
    };
    let mut boot_rom = None;
    if args.len() > 2 {
        let f_boot_rom = std::fs::read(&args[2]);
        boot_rom = match f_boot_rom {
            Ok(br) => Some(br),
            Err(_) => None,
        };
    }
    for byte in rom[0x100..=0x14F].iter() {
        print!("0x{:02X} ", byte);
    }
    rom[0x147] = 0;
    print!("\n");
    let mut sys = System::new(boot_rom, rom);
    loop {
        sys.step();
        // sleep(time::Duration::from_millis(0));
    }
}
