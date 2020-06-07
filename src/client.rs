#![allow(dead_code)]
// #![feature(async_await, async_closure)]

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
use hyper::Request;
use hyper::client::connect::Connect;
use base64;
use std::{time, thread};


type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

#[tokio::main]
async fn main() -> Result<()> {
    let src_base_url = "https://kvarto.net".parse::<Url>().expect("invalid src base url");
    let dst_base_url = "http://localhost:8080".parse::<Url>().expect("invalid dst base url");
    let interval = time::Duration::from_secs(3);
    let from_date = chrono::Utc::now().to_rfc3339();
    let mut from_id: u32 = 0;
    let src_url_with_path = src_base_url.join("/notification-results").unwrap();

    let https = HttpsConnector::new();
    let client = Client::builder().build::<_, Body>(https);

    loop {
        let mut src_url = src_url_with_path.clone();
        src_url.query_pairs_mut()
            // .append_pair("from_date", from_date_str.as_str())
            .append_pair("from_id", from_id.to_string().as_str());

        println!("{}", src_url);
        let events = fetch_events(&client, src_url).await?;
        for event in events {
            let res = send(&client, dst_base_url.clone(), &event).await;
            let status = if res.is_ok() { "success" } else { "fail" };
            println!("{}: {} {} {}", status, event.id, event.method.as_str(), event.relative_uri.as_str());
            from_id = from_id.max(event.id);
        }
        thread::sleep(interval);
    }

    Ok(())
}

async fn fetch_events<C>(client: &Client<C, Body>, url: Url) -> Result<Vec<Event>>
    where C: Connect + Clone + Send + Sync + 'static {
        let res = client.get(url.into_string().parse::<Uri>()?).await?;
        let body = hyper::body::aggregate(res).await?;
        let events = serde_json::from_reader(body.reader())?;
        Ok(events)
    }

async fn send<C>(client: &Client<C, Body>, base_url: Url, event: &Event) -> Result<()>
    where C: Connect + Clone + Send + Sync + 'static {
    let dst_url = base_url.clone().join(event.relative_uri.as_str())?;
    let body = base64::decode(event.body_base64.to_owned())?;
    let mut builder = Request::builder()
        .method(event.method.as_str())
        .uri(dst_url.into_string().parse::<Uri>()?);
    for (k, v) in &event.headers {
        builder = builder.header(k, v);
    }
    let req = builder.body(Body::from(body))?;
    client.request(req).await.map(|response| ()).map_err(|e| e.into())
}