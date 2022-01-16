#![no_std]
#![no_main]
#[macro_use(block)]
extern crate nb;

use bitbang_hal;
use embedded_graphics::{egrectangle, egtext, egline};
use embedded_graphics::fonts::Text;
use embedded_graphics::image::Image;
use embedded_graphics::primitives::Rectangle;
use embedded_graphics::style::{PrimitiveStyleBuilder, TextStyleBuilder};
use embedded_graphics::{pixelcolor::BinaryColor, prelude::*};
use embedded_picofont::FontPico;
use esp8266_hal::ehal::digital::v2::InputPin;
use esp8266_hal::flash::ESPFlash;
use esp8266_hal::gpio::{
    Gpio0, Gpio12, Gpio13, Gpio14, Gpio15, Gpio16, Gpio2, Input, Output, PullDown, PullUp, PushPull,
};
use esp8266_hal::prelude::*;
use esp8266_hal::target::Peripherals;
use esp8266_hal::time::{KiloHertz, Nanoseconds};
use esp8266_hal::timer::Timer1;
use minicbor::decode;
use panic_halt as _;
use pokemon::pokedex::Pokemon;
use sh1106::{prelude::*, Builder};
use tinybmp::Bmp;

enum Buttons {
    LEFT,
    UP,
    RIGHT,
    DOWN,
    A,
    B,
}

impl Buttons {
    fn to_index(&self) -> usize {
        match self {
            Buttons::LEFT => 0,
            Buttons::UP => 1,
            Buttons::RIGHT => 2,
            Buttons::DOWN => 3,
            Buttons::A => 4,
            Buttons::B => 5,
        }
    }
}

struct ButtonStates {
    pressed: [bool; 6],
    was_consumed: [bool; 6],
}

impl ButtonStates {
    fn new() -> ButtonStates {
        return ButtonStates {
            pressed: [false; 6],
            was_consumed: [false; 6],
        };
    }
    fn is_pressed(&self, button: Buttons) -> bool {
        return self.pressed[button.to_index()] && !self.was_consumed[button.to_index()];
    }
    fn consume(&mut self, button: Buttons) -> bool {
        if self.pressed[button.to_index()] && !self.was_consumed[button.to_index()] {
            self.was_consumed[button.to_index()] = true;
            return true;
        }
        return false;
    }
    fn update(&mut self, button: Buttons, pressed: bool) {
        if pressed && !self.pressed[button.to_index()] {
            self.pressed[button.to_index()] = true;
            self.was_consumed[button.to_index()] = false;
        } else if !pressed && self.pressed[button.to_index()] {
            self.pressed[button.to_index()] = false;
        }
    }
}

fn update_buttons(
    states: &mut ButtonStates,
    mut left: Gpio16<Input<PullDown>>,
    up: &Gpio14<Input<PullUp>>,
    right: &Gpio13<Input<PullUp>>,
    down: &Gpio12<Input<PullUp>>,
    a: &Gpio2<Input<PullUp>>,
    b: &Gpio0<Input<PullUp>>,
) -> Gpio16<Input<PullDown>> {
    // Easy pins first.
    states.update(Buttons::UP, up.is_low().unwrap_or(false));
    states.update(Buttons::RIGHT, right.is_low().unwrap_or(false));
    states.update(Buttons::DOWN, down.is_low().unwrap_or(false));
    states.update(Buttons::A, a.is_low().unwrap_or(false));
    states.update(Buttons::B, b.is_low().unwrap_or(false));

    // Gpio16 is a bit weird. It does low when pressed, and we need to pull
    // it up afterwards ourselves. GPIO pins are modelled using ownership
    // rules, so we need to take the action pin as a move in our args, then
    // give it back once we're done with it.
    let pressed = left.is_low().unwrap();
    states.update(Buttons::LEFT, pressed);
    if pressed {
        let mut output = left.into_push_pull_output();
        output.set_high().ok();
        left = output.into_floating_input();
    }
    return left;
}

#[entry]
fn main() -> ! {
    let dp = Peripherals::take().unwrap();
    let pins = dp.GPIO.split();

    // Button configuration!
    let mut btns = ButtonStates::new();
    let mut btn_left = pins.gpio16.into_floating_input();
    let btn_up = pins.gpio14.into_pull_up_input();
    let btn_right = pins.gpio13.into_pull_up_input();
    let btn_down = pins.gpio12.into_pull_up_input();
    let btn_a = pins.gpio2.into_pull_up_input();
    let btn_b = pins.gpio0.into_pull_up_input();

    // Neopixel configuration!
    let mut neopixel = pins.gpio15.into_push_pull_output();
    let (mut timer1, mut timer2) = dp.TIMER.timers();
    neopixel.set_low().unwrap();

    // Flash configuration!
    let mut storage = dp.SPI0.flash();
    let mut magic_buf = [0u8; 8];
    storage.read(0x200000, &mut magic_buf).unwrap();
    let magic1 = u32::from_be_bytes(magic_buf[0..4].try_into().unwrap());
    let magic2 = u32::from_be_bytes(magic_buf[4..8].try_into().unwrap());

    // Display configuration!
    let scl = pins.gpio5.into_open_drain_output(); // d1
    let sda = pins.gpio4.into_open_drain_output(); // d2
                                                   // Timer must be double the rate of the desired clock rate. Standard clock rates
                                                   // are 100KHz, and 400 KHz.
    timer2.start(KiloHertz(200));
    let i2c = bitbang_hal::i2c::I2cBB::new(scl, sda, timer2);
    let mut display: GraphicsMode<_> = Builder::new().connect_i2c(i2c).into();

    display.init().unwrap();
    display.flush().ok();

    let black_solid = PrimitiveStyleBuilder::new()
        .fill_color(BinaryColor::Off)
        .build();
    let white_solid = PrimitiveStyleBuilder::new()
        .fill_color(BinaryColor::On)
        .build();
    let white_text = TextStyleBuilder::new(FontPico)
        .text_color(BinaryColor::On)
        .build();
    let black_text = TextStyleBuilder::new(FontPico)
        .text_color(BinaryColor::Off)
        .build();

    display.flush().ok();

    let mut pokemon_id = 1;
    let mut pokemon: Pokemon;
    let mut has_drawn = false;
    loop {
        btn_left = update_buttons(
            &mut btns, btn_left, &btn_up, &btn_right, &btn_down, &btn_a, &btn_b,
        );

        if btns.consume(Buttons::UP) {
            pokemon_id += 1;
            has_drawn = false
        }
        if btns.consume(Buttons::DOWN) {
            pokemon_id -= 1;
            has_drawn = false
        }
        if pokemon_id <= 0 {
            pokemon_id += 151;
        } else if pokemon_id > 151 {
            pokemon_id -= 151;
        }

        if !has_drawn {
            has_drawn = true;
            egrectangle!(top_left = (0, 0), bottom_right = (128, 64), style = black_solid).draw(&mut display).ok();

            let res = read_pokemon(pokemon_id, &mut storage);
            if res.is_err() {
                Text::new("ERROR", Point::new(0, 0))
                    .into_styled(white_text)
                    .draw(&mut display)
                    .ok();
                let ReadError(text) = res.err().unwrap();
                Text::new(text, Point::new(0, 6))
                    .into_styled(white_text)
                    .draw(&mut display)
                    .ok();
                display.flush().ok();
                continue;
            }
            pokemon = res.unwrap();

            egrectangle!(top_left = (0, 0), bottom_right = (128, 6), style = white_solid).draw(&mut display).ok();
            egtext!(text = core::str::from_utf8(&pokemon.name).unwrap(), top_left = (1, 1), style = black_text).draw(&mut display).ok();
            egline!(start = (57, 7), end = (57, 64), style = white_solid).draw(&mut display).ok();

            egtext!(text = pokemon.type_primary.name(), top_left = (58, 10), style = white_text).draw(&mut display).ok();
            pokemon.type_secondary.and_then(|t| egtext!(text = t.name(), top_left = (58, 16), style = white_text).draw(&mut display).ok());

            egtext!(text = "HP", top_left = (58, 28), style = white_text).draw(&mut display).ok();
            egtext!(text = "ATK", top_left = (58, 36), style = white_text).draw(&mut display).ok();
            egtext!(text = "DEF", top_left = (58, 44), style = white_text).draw(&mut display).ok();
            egtext!(text = "SPD", top_left = (93, 28), style = white_text).draw(&mut display).ok();
            egtext!(text = "SATK", top_left = (93, 36), style = white_text).draw(&mut display).ok();
            egtext!(text = "SDEF", top_left = (93, 44), style = white_text).draw(&mut display).ok();

            egtext!(text = core::str::from_utf8(&num_to_str(pokemon.hp.base_value.into())).unwrap(), top_left = (75, 28), style = white_text).draw(&mut display).ok();
            egtext!(text = core::str::from_utf8(&num_to_str(pokemon.attack.base_value.into())).unwrap(), top_left = (75, 36), style = white_text).draw(&mut display).ok();
            egtext!(text = core::str::from_utf8(&num_to_str(pokemon.defense.base_value.into())).unwrap(), top_left = (75, 44), style = white_text).draw(&mut display).ok();
            egtext!(text = core::str::from_utf8(&num_to_str(pokemon.speed.base_value.into())).unwrap(), top_left = (113, 28), style = white_text).draw(&mut display).ok();
            egtext!(text = core::str::from_utf8(&num_to_str(pokemon.special_attack.base_value.into())).unwrap(), top_left = (113, 36), style = white_text).draw(&mut display).ok();
            egtext!(text = core::str::from_utf8(&num_to_str(pokemon.special_defense.base_value.into())).unwrap(), top_left = (113, 44), style = white_text).draw(&mut display).ok();

            egrectangle!(top_left = (56, 57), bottom_right = (128, 64), style = white_solid).draw(&mut display).ok();
            egtext!(text = "A:MOVES    B:BACK", top_left = (58, 58), style = black_text).draw(&mut display).ok();

            let image = Bmp::from_slice(pokemon.sprite.as_ref()).unwrap();
            let mut real_image = Image::new(&image, Point::zero());
            real_image.translate_mut(Point::new(0, 7));
            real_image.draw(&mut display).ok();
            display.flush().ok();
            timer1.delay_ms(200);
        }
    }

    /*     loop {
        configuring = button.into_push_pull_output();
        configuring.set_high().ok();
        button = configuring.into_floating_input();

        if button.is_high().unwrap() {
            show_colour(&mut timer1, &mut neopixel, 0x0f, 0x00, 0x00);
        } else {
            show_colour(&mut timer1, &mut neopixel, 0x00, 0x00, 0x0f);
        }
    } */
}

fn num_to_str(mut num: u32) -> [u8; 3] {
    let mut buf = [0u8; 3];
    let mut base: u32 = 100;
    let mut index_modifier = 0;
    let mut has_non_zero = false;

    for i in 0..3 {
        let digit: u8 = (num / base).try_into().unwrap();
        if digit == 0 && !has_non_zero {
            index_modifier += 1;
        } else {
            has_non_zero = true;
            buf[i - index_modifier] = digit + 48u8; // trust me babe :)
        }
        num %= base;
        base /= 10;
    }
    return buf
}

const DATA_OFFSET: u32 = 0x200000;
const DATA_BUFFER_SIZE: usize = 2048;

#[derive(Debug)]
struct ReadError(&'static str);

fn read_pokemon(id: u8, flash: &mut ESPFlash) -> Result<Pokemon, ReadError> {
    // First read this id's row in the lookup table.
    let mut table_entry = [0u8; 8];
    let row_offset = match DATA_OFFSET.checked_add(u32::from(id) * 8) {
        Some(i) => i,
        None => return Err(ReadError("row offset out of bounds")),
    };
    if flash.read(row_offset, &mut table_entry).is_err() {
        return Err(ReadError("read row failed"));
    }

    // Parse the row data.
    let offset = u32::from_be_bytes(table_entry[0..4].try_into().unwrap());
    let size: usize = match u32::from_be_bytes(table_entry[4..8].try_into().unwrap()).try_into() {
        Ok(i) => i,
        Err(_) => return Err(ReadError("failed to convert size")),
    };

    // Prefer to return an error than to panic.
    if size > DATA_BUFFER_SIZE {
        return Err(ReadError("data larger than buffer"));
    }

    // Now we can read the data that the row was pointing to.
    let mut buf = [0u8; DATA_BUFFER_SIZE];
    let data_offset = match DATA_OFFSET.checked_add(offset) {
        Some(i) => i,
        None => return Err(ReadError("data offset overflowed")),
    };
    flash.read(data_offset, &mut buf[..size]);

    match minicbor::decode(&buf) {
        Ok(data) => return Ok(data),
        Err(_) => return Err(ReadError("failed to decode data")),
    }
}

fn show_colour(
    timer: &mut Timer1,
    pin: &mut Gpio15<Output<PushPull>>,
    red: u8,
    green: u8,
    blue: u8,
) {
    timer.start(Nanoseconds(400));
    show_component(timer, pin, green);
    show_component(timer, pin, red);
    show_component(timer, pin, blue);
}

fn show_component(timer: &mut Timer1, pin: &mut Gpio15<Output<PushPull>>, component: u8) {
    let mut mask = 0b10000000;
    for _i in 0..8 {
        let res = (component & mask) != 0;
        if res {
            block!(timer.wait()).ok();
            pin.set_high().ok();
            block!(timer.wait()).ok();
            block!(timer.wait()).ok();
            pin.set_low().ok();
        } else {
            block!(timer.wait()).ok();
            pin.set_high().ok();
            pin.set_low().ok();
            block!(timer.wait()).ok();
            block!(timer.wait()).ok();
        }
        mask >>= 1;
    }
}
