use backend::input::Button;
use backend::system::System;
use backend::{SCREEN_H, SCREEN_W};
use minifb::{Key, Scale, ScaleMode, Window, WindowOptions};

const DMG: [u32; 4] = [0xFFE0F8D0, 0xFF88C070, 0xFF346856, 0xFF081820];

const KEYMAP: [(Key, Button); 8] = [
    (Key::Z, Button::A),
    (Key::X, Button::B),
    (Key::Backspace, Button::Select),
    (Key::Enter, Button::Start),
    (Key::Right, Button::Right),
    (Key::Left, Button::Left),
    (Key::Up, Button::Up),
    (Key::Down, Button::Down),
];

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let rom = std::fs::read(&args[1]).expect("rom");
    let boot = args.get(2).and_then(|p| std::fs::read(p).ok());

    let mut sys = System::new(boot, rom, false);
    let mut win = Window::new(
        "gb-emu",
        SCREEN_W,
        SCREEN_H,
        WindowOptions {
            scale: Scale::X4,
            scale_mode: ScaleMode::AspectRatioStretch,
            resize: true,
            ..Default::default()
        },
    )
    .expect("window creation shouldn't fail");
    win.set_target_fps(60);

    let mut buf = vec![0u32; SCREEN_W * SCREEN_H];
    while win.is_open() && !win.is_key_down(Key::Escape) {
        for (k, b) in KEYMAP {
            sys.set_button(b, win.is_key_down(k));
        }
        sys.run_frame();
        for (px, &i) in buf.iter_mut().zip(sys.get_framebuffer().iter()) {
            *px = DMG[(i & 3) as usize];
        }
        win.update_with_buffer(&buf, SCREEN_W, SCREEN_H).unwrap();
    }
}
