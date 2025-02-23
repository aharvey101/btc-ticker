use embedded_graphics::{
    draw_target, geometry,
    mono_font::{MonoTextStyle, ascii::FONT_10X20},
    prelude::*,
    primitives::Rectangle,
    text::{Baseline, Text},
};
use embedded_hal::delay::DelayNs;
use epd_waveshare::{
    epd2in13_v2::{self, Display2in13},
    prelude::*,
};
use linux_embedded_hal::{
    SpidevDevice, SysfsPin,
    spidev::{SpiModeFlags, SpidevOptions},
    sysfs_gpio::Direction,
};
use profont::{PROFONT_18_POINT, PROFONT_24_POINT};
use serde::Deserialize;

//extern crate embedded_graphics;
//extern crate embedded_hal;
//extern crate epd_waveshare;
//extern crate linux_embedded_hal;
//extern crate reqwest;
//extern crate serde;
//extern crate serde_json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // create spi

    let mut spi = SpidevDevice::open("/dev/spidev0.0").unwrap();

    let spi_options = SpidevOptions::new()
        .bits_per_word(8)
        .max_speed_hz(4_000_000)
        .mode(SpiModeFlags::SPI_MODE_0)
        .build();

    spi.configure(&spi_options)?;

    let busy = SysfsPin::new(536);
    //
    let dc = SysfsPin::new(537);
    let rst = SysfsPin::new(529);

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

    // Clear the display
    epd.clear_frame(&mut spi, &mut delay)?;
    epd.display_frame(&mut spi, &mut delay)?;

    let mut display = Display2in13::default();

    let mut display_buffer = [0u8; 250 * 122 / 8];
    display.set_rotation(DisplayRotation::Rotate90);

    let binance_url = "https://api.binance.com/api/v3/ticker/24hr?symbol=BTCUSDT";
    let (price_x, price_width) = get_aligned_coords(16, 96);
    let price_y = 20;
    let price_height = 10;

    let mut last_price = 0.0;
    loop {
        let res = reqwest::get(binance_url)
            .await
            .expect("Failed to do get request")
            .text()
            .await
            .expect("Failed to parse response");

        let res: Ticker = serde_json::from_str(res.as_str()).unwrap();

        let last_price = res.last_price.to_string();
        let last_price = format!("${:.8}", last_price);
        let last_price = last_price.as_str();

        let text_style = MonoTextStyle::new(&PROFONT_24_POINT, Color::White);
        // Create a drawable area for this buffer

        // Clear price area
        let price_area = embedded_graphics::primitives::Rectangle::new(
            Point::new(price_x, price_y),
            Size::new(price_width, price_height),
        );

        let mut price_box = display.cropped(&price_area);

        let buffer_size = (price_width * price_height / 8) as usize;
        let mut partial_buffer = vec![0u8; buffer_size];

        Text::with_baseline(
            last_price,
            Point::new(price_x, price_y),
            text_style,
            Baseline::Bottom,
        )
        .draw(&mut price_box)
        .unwrap();

        epd.wait_until_idle(&mut spi, &mut delay)?;

        epd.update_partial_frame(
            &mut spi,
            &mut delay,
            &partial_buffer,
            price_x.try_into().unwrap(),
            price_y.try_into().unwrap(),
            price_width,
            price_height,
        )?;
        epd.display_frame(&mut spi, &mut delay)?;

        //NOTE: sleep for 1 second
        let duration = tokio::time::Duration::from_secs(5);
        tokio::time::sleep(duration).await;
    }

    // Draw some text

    // Put the display to sleep when done
    //epd.sleep(&mut spi, &mut delay)?;

    // Cleanup GPIO
    //busy.unexport()?;
    //dc.unexport()?;
    //rst.unexport()?;
}
struct MyDelay;

impl DelayNs for MyDelay {
    fn delay_ns(&mut self, ns: u32) {
        std::thread::sleep(std::time::Duration::from_nanos(ns as u64));
    }
}
#[derive(Default, Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Ticker {
    pub symbol: String,
    pub price_change: String,
    pub price_change_percent: String,
    pub weighted_avg_price: String,
    pub prev_close_price: String,
    pub last_price: String,
    pub last_qty: String,
    pub bid_price: String,
    pub bid_qty: String,
    pub ask_price: String,
    pub ask_qty: String,
    pub open_price: String,
    pub high_price: String,
    pub low_price: String,
    pub volume: String,
    pub quote_volume: String,
    pub open_time: i64,
    pub close_time: i64,
    pub first_id: i64,
    pub last_id: i64,
    pub count: i64,
}

// Helper function to get coordinates aligned to 8 pixels
fn get_aligned_coords(x: i32, width: u32) -> (i32, u32) {
    let aligned_x = (x / 8) * 8; // Round down to nearest multiple of 8
    let end_x = x + width as i32;
    let aligned_end_x = ((end_x + 7) / 8) * 8;
    let aligned_width = (aligned_end_x - aligned_x) as u32;
    (aligned_x, aligned_width)
}
