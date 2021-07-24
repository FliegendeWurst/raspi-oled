use embedded_graphics::image::{Image, ImageRaw};
use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::{Circle, Line, PrimitiveStyle, Rectangle};

use embedded_graphics::{
    mono_font::{ascii::FONT_6X10, MonoTextStyleBuilder},
    text::Text,
};
use ssd1306::{prelude::*, I2CDisplayInterface, Ssd1306};
use linux_embedded_hal::I2cdev;
use machine_ip;
use std::thread::sleep;
use std::time::Duration;

static IMG_DATA: &[u8; 512] = include_bytes!("../rust.raw");

fn main() {
    let i2c = I2cdev::new("/dev/i2c-1").unwrap();

    let interface = I2CDisplayInterface::new(i2c);
    let mut disp = Ssd1306::new(
        interface,
        DisplaySize128x64,
        DisplayRotation::Rotate0,
    ).into_buffered_graphics_mode();

    disp.init().unwrap();
    disp.flush().unwrap();

    let text_style = MonoTextStyleBuilder::new()
        .font(&FONT_6X10)
        .text_color(BinaryColor::On)
        .build();
    
    let text = "0123456789012345678901";
    Text::new(&text, Point::new(0, 10), text_style)
            .draw(&mut disp)
            .unwrap();
    disp.flush().unwrap();
    sleep(Duration::from_secs(2));
    disp.clear();

    loop {
        Line::new(Point::new(8, 16 + 16), Point::new(8 + 16, 16 + 16))
            .into_styled(PrimitiveStyle::with_stroke(BinaryColor::On, 1))
            .draw(&mut disp).unwrap();
        Line::new(Point::new(8, 16 + 16), Point::new(8 + 8, 16))
            .into_styled(PrimitiveStyle::with_stroke(BinaryColor::On, 1))
            .draw(&mut disp).unwrap();
        
        Line::new(Point::new(8 + 16, 16 + 16), Point::new(8 + 8, 16))
            .into_styled(PrimitiveStyle::with_stroke(BinaryColor::On, 1))
            .draw(&mut disp).unwrap();

        Rectangle::new(Point::new(48, 16), Size::new(16, 16))
            .into_styled(PrimitiveStyle::with_stroke(BinaryColor::On, 1))
            .draw(&mut disp).unwrap();
            

        Circle::new(Point::new(88, 16), 17)
            .into_styled(PrimitiveStyle::with_stroke(BinaryColor::On, 1))
            .draw(&mut disp).unwrap();

        let local_addr = machine_ip::get().unwrap();

        Text::new(&format!("IP: {}", local_addr.to_string()), Point::new(0, 56), text_style)
            .draw(&mut disp)
            .unwrap();
        disp.flush().unwrap();

        sleep(Duration::from_secs(2));

        disp.clear();

        let im: ImageRaw<BinaryColor> = ImageRaw::new(IMG_DATA, 64);
        let img = Image::new(&im, Point::new(32, 0));
        img.draw(&mut disp).unwrap();
        disp.flush().unwrap();

        sleep(Duration::from_secs(2));
        disp.clear();
    }
}
