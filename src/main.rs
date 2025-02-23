use embedded_graphics::{
    mono_font::{ascii::FONT_6X10, MonoTextStyle},
    pixelcolor::BinaryColor,
    prelude::*,
    primitives::{Circle, Line, PrimitiveStyle},
    text::{Baseline, Text},
};
use embedded_hal::{
    delay::{self, DelayNs},
    digital::{self, InputPin, OutputPin},
    spi,
};
use epd_waveshare::{
    epd2in13_v2::{self, Display2in13},
    graphics::Display,
    prelude::*,
};
use linux_embedded_hal::{
    spidev::{SpiModeFlags, SpidevOptions},
    sysfs_gpio::{Direction, Pin},
    SpidevDevice, SysfsPin,
};
extern crate embedded_graphics;
extern crate embedded_hal;
extern crate epd_waveshare;
extern crate linux_embedded_hal;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // create spi

    let mut spi = SpidevDevice::open("/dev/spidev0.0").unwrap();

    let spi_options = SpidevOptions::new()
        .bits_per_word(8)
        .max_speed_hz(4_000_000)
        .mode(SpiModeFlags::SPI_MODE_0)
        .build();

    spi.configure(&spi_options);

    let delay = linux_embedded_hal::Delay {};

    let busy = SysfsPin::new(536); // Busy pin
                                   //
    let dc = SysfsPin::new(537); // Data/Command pin
    let rst = SysfsPin::new(529); // Reset pin
                                  // Export the pins (makes them available for use)
    busy.export()?;
    dc.export()?;
    rst.export()?;

    // Set pin directions
    busy.set_direction(Direction::In)?; // Busy is input
    dc.set_direction(Direction::Out)?; // DC is output
    rst.set_direction(Direction::Out)?; // Reset is output
    let mut delay = MyDelay {};

    // Initialize the EPD
    let mut epd = epd2in13_v2::Epd2in13::new(&mut spi, busy, dc, rst, &mut delay, None).unwrap();
    // After initializing the EPD
    //epd.set_lut(&mut spi, &mut delay, None)?; // Use default LUT

    // Clear the display
    epd.clear_frame(&mut spi, &mut delay);
    epd.display_frame(&mut spi, &mut delay);

    // Create a display buffer
    let mut display = Display2in13::default();
    // Draw some graphics using embedded-graphics

    // Draw a circle
    Circle::new(Point::new(20, 20), 30)
        .into_styled(PrimitiveStyle::with_stroke(Color::White, 2))
        .draw(&mut display)
        .unwrap();

    // Draw a line
    println!("Drawing a line");
    Line::new(Point::new(10, 10), Point::new(100, 100))
        .into_styled(PrimitiveStyle::with_stroke(Color::White, 1))
        .draw(&mut display)
        .unwrap();

    // Draw some text
    println!("Drawing some text");
    let text_style = MonoTextStyle::new(&FONT_6X10, Color::White);
    Text::with_baseline(
        "Hello Waveshare!",
        Point::new(10, 120),
        text_style,
        Baseline::Top,
    )
    .draw(&mut display)
    .unwrap();
    epd.wait_until_idle(&mut spi, &mut delay);

    // After drawing your graphics but before updating the frame, add:

    // Transfer the frame to the display
    epd.update_frame(&mut spi, display.buffer(), &mut delay);
    epd.display_frame(&mut spi, &mut delay)?;

    epd.wait_until_idle(&mut spi, &mut delay);
    // Put the display to sleep when done
    epd.sleep(&mut spi, &mut delay)?;

    // Cleanup GPIO
    //busy.unexport()?;
    //dc.unexport()?;
    //rst.unexport()?;
    Ok(())
}
struct MyDelay;

impl DelayNs for MyDelay {
    fn delay_ns(&mut self, ns: u32) {
        std::thread::sleep(std::time::Duration::from_nanos(ns as u64));
    }
}
