#![no_std]
#![no_main]

mod ping_pong;
use ping_pong::*;

extern crate alloc;

use uefi::{
    prelude::*,
    proto::{
        console::{gop::GraphicsOutput, text::Input},
        rng::Rng,
    },
};

#[entry]
fn main(_image_handle: Handle, system_table: SystemTable<Boot>) -> Status {
    uefi::helpers::init().unwrap();
    let bs = system_table.boot_services();
    bs.set_watchdog_timer(0, 69696, None).unwrap(); // Убрать пятиминутный таймаут

    let rng_handle = bs.get_handle_for_protocol::<Rng>().unwrap();
    let mut rng = bs.open_protocol_exclusive::<Rng>(rng_handle).unwrap();

    let gop_handle = bs.get_handle_for_protocol::<GraphicsOutput>().unwrap();
    let mut gop = bs
        .open_protocol_exclusive::<GraphicsOutput>(gop_handle)
        .unwrap();
    let (width, height) = gop.current_mode_info().resolution();

    let input_handle = bs.get_handle_for_protocol::<Input>().unwrap();
    let mut input = bs.open_protocol_exclusive::<Input>(input_handle).unwrap();

    let mut ping_pong = PingPong::new(width as _, height as _);

    loop {
        ping_pong.update(&mut input, &mut rng);
        ping_pong.draw(&mut gop);

        bs.stall(16_500); // ~60 кадров в секунду
    }
}
