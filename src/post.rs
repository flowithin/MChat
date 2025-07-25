use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let resp = reqwest::get("https://httpbin.org/ip").await?.text().await?;
    println!("{:#?}", resp);
    Ok(())
}
