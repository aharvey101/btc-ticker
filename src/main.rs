use reqwest;

struct Binance_Response {}

#[tokio::main]
async fn main() {
    let binance_url = "https://api.binance.com/api/v3/exchangeInfo?symbol=BTCUSDT";

    loop {
        let res = reqwest::get(binance_url)
            .await
            .expect("Failed to do get request");

        println!("res: {:?}", res);

        let duration = tokio::time::Duration::from_secs(1);
        tokio::time::sleep(duration).await;
    }
}
