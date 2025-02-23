use embedded_graphics::{
    mono_font::{
        MonoTextStyle,
        ascii::{FONT_6X10, FONT_10X20},
    },
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
    SpidevDevice, SysfsPin,
    spidev::{SpiModeFlags, SpidevOptions},
    sysfs_gpio::{Direction, Pin},
};
use serde::{Deserialize, Serialize};

extern crate embedded_graphics;
extern crate embedded_hal;
extern crate epd_waveshare;
extern crate linux_embedded_hal;
extern crate reqwest;
extern crate serde;
extern crate serde_json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // create spi

    let mut spi = SpidevDevice::open("/dev/spidev0.0").unwrap();

    let spi_options = SpidevOptions::new()
        .bits_per_word(8)
        .max_speed_hz(4_000_000)
        .mode(SpiModeFlags::SPI_MODE_0)
        .build();

    spi.configure(&spi_options);

    let delay = linux_embedded_hal::Delay {};

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

    // Set partial mode:

    // Clear the display
    epd.clear_frame(&mut spi, &mut delay);
    epd.display_frame(&mut spi, &mut delay);

    let mut display = Display2in13::default();

    display.set_rotation(DisplayRotation::Rotate90);

    let binance_url = "https://api.binance.com/api/v3/ticker/24hr?symbol=BTCUSDT";

    loop {
        let res = reqwest::get(binance_url)
            .await
            .expect("Failed to do get request")
            .text()
            .await
            .expect("Failed to parse response");

        let res: Ticker = serde_json::from_str(res.as_str()).unwrap();

        println!("res: {:?}", res.last_price);
        let last_price = res.last_price.to_string();
        let last_price = last_price.as_str();
        epd.wait_until_idle(&mut spi, &mut delay);
        epd.clear_frame(&mut spi, &mut delay);
        epd.display_frame(&mut spi, &mut delay);

        let text_style = MonoTextStyle::new(&FONT_10X20, Color::White);
        Text::with_baseline(
            last_price,
            Point::new(10, 120),
            text_style,
            Baseline::Bottom,
        )
        .draw(&mut display)
        .unwrap();

        epd.update_frame(&mut spi, display.buffer(), &mut delay);
        epd.display_frame(&mut spi, &mut delay)?;
        //NOTE: sleep for 1 second
        let duration = tokio::time::Duration::from_secs(1);
        tokio::time::sleep(duration).await;
    }

    // Draw some text

    // Put the display to sleep when done
    //epd.sleep(&mut spi, &mut delay)?;

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

// Custom serializer/deserializer for handling string-formatted numbers
mod string_as_f64 {
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(value: &f64, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&value.to_string())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<f64, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        s.parse::<f64>().map_err(serde::de::Error::custom)
    }
}
