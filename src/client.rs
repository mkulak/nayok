#![allow(dead_code)]

pub mod data;

use bytes::buf::BufExt as _;
use hyper::{Client, Method, Response};
use hyper_tls::HttpsConnector;
use data::Event;
use url::Url;
use hyper::Uri;
use hyper::Body;
use hyper::Request;
use hyper::client::connect::Connect;
use base64;
use std::{time, thread};
use clap::{Arg, App};


type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

#[tokio::main]
async fn main() -> Result<()> {
    let matches = App::new("Nayok client")
        .version("0.2.0")
        .about("Periodically retrieves saved http requests from Nayok server and re-sends them to specified address")
        .arg(
            Arg::with_name("SRC_URL")
                .help("Sets base url of nayok server")
                .short("s")
                .long("src-base-url")
                .takes_value(true)
                .default_value("https://kvarto.net")
        )
        .arg(
            Arg::with_name("DST_URL")
                .help("Sets base url of receiver")
                .short("d")
                .long("dst-base-url")
                .takes_value(true)
                .default_value("http://localhost:8080")
        )
        .arg(
            Arg::with_name("INTERVAL")
                .help("Sets polling interval in seconds")
                .short("i")
                .long("interval")
                .takes_value(true)
                .default_value("5")
        )
        .arg(
            Arg::with_name("FROM_DATE")
                .help("Only send events created after specified date (RFC3339, example: 2020-06-09T23:59:59Z)")
                .short("f")
                .long("from-date")
                .takes_value(true)
        )
        .arg(
            Arg::with_name("FROM_ID")
                .help("Only send events with ids bigger than specified")
                .long("from-id")
                .default_value("0")
                .takes_value(true)
        )
        .arg(
            Arg::with_name("TOKEN")
                .help("Auth token for authorizing requests to /notification-results")
                .short("t")
                .long("auth-token")
                .env("NAYOK_TOKEN")
                .takes_value(true)
                .required(true)
        )
        .get_matches();


    let src_base_url = matches.value_of("SRC_URL").unwrap().parse::<Url>().expect("invalid src base url");
    let dst_base_url = matches.value_of("DST_URL").unwrap().parse::<Url>().expect("invalid dst base url");
    let interval = time::Duration::from_secs(matches.value_of("INTERVAL").unwrap().parse::<u64>().expect("invalid interval"));
    let now = chrono::Utc::now().to_rfc3339();
    let from_date = matches.value_of("FROM_DATE").unwrap_or(now.as_str());
    let src_url_with_path = src_base_url.join("/notification-results").unwrap();
    let mut from_id: u32 = matches.value_of("FROM_ID").unwrap().parse::<u32>().expect("invalid from_id");
    let token = matches.value_of("TOKEN").unwrap().to_owned();

    let https = HttpsConnector::new();
    let client = Client::builder().build::<_, Body>(https);

    loop {
        let mut src_url = src_url_with_path.clone();
        src_url.query_pairs_mut()
            .append_pair("from_date", from_date)
            .append_pair("from_id", from_id.to_string().as_str());

        let events = fetch_events(&client, src_url, token.as_str()).await?;
        for event in events {
            let res = send(&client, dst_base_url.clone(), &event).await;
            let status = if res.map(|r| r.status().is_success()).unwrap_or(false) { "success" } else { "fail" };
            println!("{}: {} {} {}", status, event.id, event.method.as_str(), event.relative_uri.as_str());
            from_id = from_id.max(event.id);
        }
        thread::sleep(interval);
    }
}

async fn fetch_events<C>(client: &Client<C, Body>, url: Url, token: &str) -> Result<Vec<Event>>
    where C: Connect + Clone + Send + Sync + 'static {
        let req = Request::builder()
            .method(Method::GET)
            .header("Authorization", token)
            .uri(url.into_string().parse::<Uri>()?)
            .body(Body::empty())?;
        let res = client.request(req).await?;
        let body = hyper::body::aggregate(res).await?;
        let events = serde_json::from_reader(body.reader())?;
        Ok(events)
    }

async fn send<C>(client: &Client<C, Body>, base_url: Url, event: &Event) -> Result<Response<Body>>
    where C: Connect + Clone + Send + Sync + 'static {
    let dst_url = base_url.clone().join(event.relative_uri.as_str())?;
    let body = base64::decode(event.body_base64.to_owned())?;
    let mut builder = Request::builder()
        .method(event.method.as_str())
        .uri(dst_url.into_string().parse::<Uri>()?);
    for (k, v) in &event.headers {
        if k != "host" {
            builder = builder.header(k, v);
        }
    }
    let req = builder.body(Body::from(body))?;
    client.request(req).await.map_err(|e| e.into())
}