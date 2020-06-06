#![allow(dead_code)]

pub mod data;

use bytes::buf::BufExt as _;
use hyper::Client;
use hyper_tls::HttpsConnector;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, NaiveDateTime, Utc, FixedOffset};
use data::Event;
use url::Url;
use hyper::Uri;
use hyper::Body;
use hyper::client::connect::Connect;


type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;
// type HyperC = Connect + Clone + Send + Sync + 'static;

#[tokio::main]
async fn main() -> Result<()> {
    let base_src_url = "https://kvarto.net";
    let dst_url = "http://localhost:8080";
    // let from_date = Some(chrono::Utc::now());
    // let from_id = Some(0);
    // let url = Url::parse_with_params(base_url, &[("lang", "rust"), ("browser", "servo")])?;
    let url = "https://kvarto.net/notification-results".parse().unwrap();

    let https = HttpsConnector::new();
    let client = Client::builder().build::<_, Body>(https);

    let events = fetch_events(&client, url).await?;

    // println!("events: {:#?}", events);
    events.iter().for_each(|event| {
        println!("event: {:?}", event)
    });

    Ok(())
}

async fn fetch_events<C>(client: &Client<C, Body>, url: Uri) -> Result<Vec<Event>>
    where C: Connect + Clone + Send + Sync + 'static {
        let res = client.get(url).await?;
        let body = hyper::body::aggregate(res).await?;
        let events = serde_json::from_reader(body.reader())?;
        Ok(events)
    }
