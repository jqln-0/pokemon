#![no_std]
#![no_main]
#[macro_use(block)]
extern crate nb;

use embedded_graphics::fonts::Text;
use embedded_graphics::image::Image;
use embedded_graphics::style::TextStyleBuilder;
use esp8266_hal::ehal::digital::v2::InputPin;
use esp8266_hal::gpio::{PushPull, Gpio15, Output};
use esp8266_hal::prelude::*;
use esp8266_hal::target::Peripherals;
use esp8266_hal::time::{Nanoseconds, KiloHertz};
use esp8266_hal::timer::Timer1;
use panic_halt as _;
use bitbang_hal;
use sh1106::{prelude::*, Builder};
use embedded_picofont::FontPico;
use embedded_graphics::{
    pixelcolor::BinaryColor,
    prelude::*,
};
use tinybmp::Bmp;

#[entry]
fn main() -> ! {
    let dp = Peripherals::take().unwrap();
    let pins = dp.GPIO.split();

    // Button configuration!
    // 16 is left. it must be floating, and pulled high after input.
    // 0 is b
    // 2 is a
    // 14 is up
    // 12 is down
    // 13 is right
    let mut configuring = pins.gpio16.into_push_pull_output();
    configuring.set_low().ok();
    let mut button = configuring.into_floating_input();

    // Neopixel configuration!
    let mut neopixel = pins.gpio15.into_push_pull_output();
    let (mut timer1, mut timer2) = dp.TIMER.timers();
    neopixel.set_low().unwrap();

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

    let text_style = TextStyleBuilder::new(FontPico).text_color(BinaryColor::On).build();
    Text::new("I AM GAY!", Point::new(0, 0)).into_styled(text_style).draw(&mut display).ok();

    let image_data = include_bytes!("../mockup.bmp");
    let image = Bmp::from_slice(image_data).unwrap();
    let mut real_image = Image::new(&image, Point::zero());
    real_image.translate_mut(Point::new(0,0));
    real_image.draw(&mut display).ok();
    display.flush().ok();

    loop {}

/*     loop {
        configuring = button.into_push_pull_output();
        configuring.set_high().ok();
        button = configuring.into_floating_input();

        if button.is_high().unwrap() {
            show_colour(&mut timer1, &mut neopixel, 0x0f, 0x00, 0x00);
        } else {
            show_colour(&mut timer1, &mut neopixel, 0x00, 0x00, 0x0f);
        }
        timer1.delay_ms(50);
    } */
}

fn show_colour(timer: &mut Timer1, pin: &mut Gpio15<Output<PushPull>>, red: u8, green: u8, blue: u8) {
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