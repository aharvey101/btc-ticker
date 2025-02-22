use reqwest;
use serde::{Deserialize, Serialize};
#[tokio::main]
async fn main() {
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

        // okay now that it talks to binance, we gotta display it

        let duration = tokio::time::Duration::from_secs(1);
        tokio::time::sleep(duration).await;
    }
}
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
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
