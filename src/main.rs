use embedded_graphics::{
    mono_font::{ascii::FONT_6X10, MonoTextStyle},
    pixelcolor::BinaryColor,
    prelude::*,
    primitives::{Circle, Line, PrimitiveStyle},
    text::{Baseline, Text},
};
use embedded_hal::{
    delay,
    digital::{self, OutputPin},
    spi,
};
use epd_waveshare::{
    epd2in13_v2::{self, Display2in13},
    graphics::Display,
    prelude::*,
};
use linux_embedded_hal::{
    spidev::{SpiModeFlags, SpidevOptions},
    SpidevDevice,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // create spi

    let mut spi = SpidevDevice::open("/dev/spidev0.0").unwrap();

    let spiOptions = SpidevOptions::new()
        .bits_per_word(8)
        .max_speed_hz(4_000_000)
        .mode(SpiModeFlags::SPI_MODE_0)
        .build();

    spi.configure(&spiOptions);

    // Initialize the EPD
    let mut epd = epd2in13_v2::Epd2in13::new(
        &mut spi,
        digital::InputPin,
        digital::OutputPin,
        digital::OutputPin,
        embedded_hal::delay,
        None,
    )
    .unwrap();

    // Clear the display
    epd.clear_frame(&mut spi, delay);
    epd.display_frame(&mut spi, delay);

    // Create a display buffer
    let mut display = Display2in13::default();

    // Draw some graphics using embedded-graphics

    // Draw a circle
    Circle::new(Point::new(50, 50), 30)
        .into_styled(PrimitiveStyle::with_stroke(BinaryColor::On, 2))
        .draw(&mut display)
        .unwrap();

    // Draw a line
    Line::new(Point::new(10, 10), Point::new(100, 100))
        .into_styled(PrimitiveStyle::with_stroke(BinaryColor::On, 1))
        .draw(&mut display)
        .unwrap();

    // Draw some text
    let text_style = MonoTextStyle::new(&FONT_6X10, BinaryColor::On);
    Text::with_baseline(
        "Hello Waveshare!",
        Point::new(10, 120),
        text_style,
        Baseline::Top,
    )
    .draw(&mut display)
    .unwrap();

    // Transfer the frame to the display
    //
    //epd.update_color_frame(&mut spi, display.black_buffer(), display.chromatic_buffer())?;
    //epd.display_frame(&mut spi, &mut delay)?;

    // Put the display to sleep when done
    //epd.sleep(&mut spi, &mut delay)?;

    // Cleanup GPIO

    Ok(())
}
