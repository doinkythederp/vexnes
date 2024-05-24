#![no_main]
#![no_std]
#![feature(error_in_core)]

extern crate alloc;

use alloc::{boxed::Box, vec::Vec};
use core::{error::Error, time::Duration};

use tetanes_core::{
    control_deck::{Config, ControlDeck, HeadlessMode},
    input::{JoypadBtnState, Player},
    ppu::Ppu,
    time::Instant,
};
use vexide::{core::io::Cursor, devices::screen::RenderMode, prelude::*};

static ROM: &[u8] = include_bytes!("../game.nes");

#[vexide::main]
async fn main(peripherals: Peripherals) -> Result<(), Box<dyn Error>> {
    println!("Loading");
    let mut screen = peripherals.screen;
    let controller = peripherals.primary_controller;
    let mut control_deck = ControlDeck::with_config(Config {
        headless_mode: HeadlessMode::NO_AUDIO,
        ..Default::default()
    });
    screen.set_render_mode(RenderMode::DoubleBuffered);

    let mut rom = Cursor::new(ROM);

    control_deck.load_rom("Legend of Zelda, The (U) (PRG1) [!]", &mut rom)?;
    // control_deck.load_rom("Super Mario Bros. (World)", &mut rom)?;
    println!("ROM loaded");

    let period = Duration::from_millis(16);
    let mut last_frame = Instant::now();
    while control_deck.is_running() {
        let mut buttons = JoypadBtnState::empty();
        if controller.button_a.is_pressed().unwrap_or_default() {
            buttons |= JoypadBtnState::A;
        }
        if controller.button_b.is_pressed().unwrap_or_default() {
            buttons |= JoypadBtnState::B;
        }
        if controller.button_up.is_pressed().unwrap_or_default() {
            buttons |= JoypadBtnState::UP;
        }
        if controller.button_down.is_pressed().unwrap_or_default() {
            buttons |= JoypadBtnState::DOWN;
        }
        if controller.button_left.is_pressed().unwrap_or_default() {
            buttons |= JoypadBtnState::LEFT;
        }
        if controller.button_right.is_pressed().unwrap_or_default() {
            buttons |= JoypadBtnState::RIGHT;
        }
        if controller.button_y.is_pressed().unwrap_or_default() {
            buttons |= JoypadBtnState::SELECT;
        }
        if controller.button_x.is_pressed().unwrap_or_default() {
            buttons |= JoypadBtnState::START;
        }
        control_deck.joypad_mut(Player::One).buttons = buttons;
        // println!("Running frame");
        // See also: `ControlDeck::clock_frame_output` and `ControlDeck::clock_frame_into`
        control_deck.clock_frame()?;

        let frame_buffer = control_deck.frame_buffer();
        let bgr_buffer: &[u32] = bytemuck::cast_slice(frame_buffer);
        let rgb_buffer: Vec<u32> = bgr_buffer
            .iter()
            .map(|&bgr| {
                let b = (bgr >> 16) & 0xFF;
                let g = (bgr >> 8) & 0xFF;
                let r = bgr & 0xFF;
                (r << 16) | (g << 8) | b
            })
            .collect();

        const LEFT_HORIZONTAL_PADDING: i16 = Screen::HORIZONTAL_RESOLUTION / 2 - 128;

        unsafe {
            let buffer_ptr = rgb_buffer.as_ptr().cast_mut();
            vex_sdk::vexDisplayCopyRect(
                LEFT_HORIZONTAL_PADDING as i32,
                Screen::HEADER_HEIGHT as i32,
                (Ppu::WIDTH as i32) + (LEFT_HORIZONTAL_PADDING as i32),
                (Ppu::HEIGHT as i32) + (Screen::HEADER_HEIGHT as i32),
                buffer_ptr,
                256,
            );
        }
        screen.render();
        let elapsed = last_frame.elapsed();
        if elapsed < period {
            sleep(period - elapsed).await;
        } else {
            sleep(Duration::from_millis(1)).await;
        }
        last_frame = Instant::now();
    }
    Ok(())
}
