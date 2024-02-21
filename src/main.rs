use std::env;
use std::fs::File;
use std::io::Read;
use sdl2::{event::Event, keyboard::Keycode};
use sdl2::pixels::Color;
use sdl2::rect::{Rect};
use sdl2::render::Canvas;
use sdl2::video::Window;
use chip8::{SCREEN_COLS, SCREEN_ROWS, Emulator};

const WINDOW_WIDTH: u32 = SCREEN_COLS as u32 * SCREEN_SCALE;
const WINDOW_HEIGHT: u32 = SCREEN_ROWS as u32 * SCREEN_SCALE;
const WINDOW_COLOR: Color = Color::WHITE;
const SCREEN_SCALE: u32 = 20;
const TICKS_PER_FRAME: u32 = 20;

fn main() -> Result<(), String> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        println!("Usage: {} {{path to rom file}}", args[0]);
        return Err(format!("Usage: {} {{path to rom file}}", args[0]));
    }

    // Init sdl2
    let ctx = sdl2::init()?;
    let vid_subsystem = ctx.video()?;

    let win = vid_subsystem
        .window("Chip-8 Emulator", WINDOW_WIDTH, WINDOW_HEIGHT)
        .opengl()
        .build()
        .unwrap();

    let mut canvas = win
        .into_canvas()
        .present_vsync()
        .build()
        .unwrap();

    canvas.clear();
    canvas.present();

    // Init emulator
    let mut emu = Emulator::new();

    let mut rom = File::open(&args[1]).unwrap();
    let mut buf = vec![];

    rom.read_to_end(&mut buf).unwrap();
    emu.load_to_memory(&buf);

    let mut event_pump = ctx.event_pump()?;
    'main: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => {
                    break 'main;
                },
                Event::KeyDown { keycode: Some(key_code), .. } => {
                    if let Some(key) = key2chip8(key_code) {
                        emu.keypress(key, true);
                    }
                },
                Event::KeyUp { keycode: Some(key_code), .. } => {
                    if let Some(key) = key2chip8(key_code) {
                        emu.keypress(key, false);
                    }
                },
                _ => {}
            }
        }

        for _ in 0..TICKS_PER_FRAME {
            emu.tick();
        }
        emu.timer_tick();
        draw_screen(&emu, &mut canvas);
    }

    Ok(())
}

fn draw_screen(emu: &Emulator, canvas: &mut Canvas<Window>) {
    // clear canvas
    canvas.set_draw_color(Color::BLACK);
    canvas.clear();

    let screen_buffer = emu.get_display();
    canvas.set_draw_color(WINDOW_COLOR);
    for (i, pixel) in screen_buffer.iter().enumerate() {
        if *pixel {
            let x = (i % SCREEN_COLS) as u32;
            let y = (i / SCREEN_COLS) as u32;

            let rect = Rect::new((x * SCREEN_SCALE) as i32, (y * SCREEN_SCALE) as i32, SCREEN_SCALE, SCREEN_SCALE);
            canvas.fill_rect(rect).unwrap();
        }
    }
    canvas.present();
}

fn key2chip8(key_code: Keycode) -> Option<usize> {
    match key_code {
        Keycode::Num1   =>    Some(0x1),
        Keycode::Num2   =>    Some(0x2),
        Keycode::Num3   =>    Some(0x3),
        Keycode::Num4   =>    Some(0xC),
        Keycode::Q      =>    Some(0x4),
        Keycode::W      =>    Some(0x5),
        Keycode::E      =>    Some(0x6),
        Keycode::R      =>    Some(0xD),
        Keycode::A      =>    Some(0x7),
        Keycode::S      =>    Some(0x8),
        Keycode::D      =>    Some(0x9),
        Keycode::F      =>    Some(0xE),
        Keycode::Z      =>    Some(0xA),
        Keycode::X      =>    Some(0x0),
        Keycode::C      =>    Some(0xB),
        Keycode::V      =>    Some(0xF),
        _               =>    None,
    }
}
