#![allow(dead_code)]

pub mod data;

use bytes::buf::BufExt as _;
use hyper::Client;
use hyper_tls::HttpsConnector;
use serde::{Deserialize, Serialize};
use data::Event;


// A simple type alias so as to DRY.
type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

#[tokio::main]
async fn main() -> Result<()> {
    let url = "https://kvarto.net/notification-results".parse().unwrap();
    let events = fetch_json(url).await?;

    println!("events: {:#?}", events);

    Ok(())
}

async fn fetch_json(url: hyper::Uri) -> Result<Vec<Event>> {
    let https = HttpsConnector::new();
    let client = Client::builder().build::<_, hyper::Body>(https);
    let res = client.get(url).await?;
    let body = hyper::body::aggregate(res).await?;
    let events = serde_json::from_reader(body.reader())?;
    Ok(events)
}
