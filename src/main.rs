#![allow(dead_code)]

use std::collections::HashMap;
use std::convert::Infallible;
use std::env;
use std::path::Path;
use base64;
use chrono::{DateTime, FixedOffset};
use hyper::{Body, Method, Request, Response, Server, StatusCode};
use hyper::service::{make_service_fn, service_fn};
use rusqlite::{Connection, Result, ToSql};
use rusqlite::NO_PARAMS;
use serde::{Deserialize, Serialize};

use data::Event;

mod data;

static SCHEMA_SQL: &'static str = include_str!("schema.sql");
static INSERT_EVENT_SQL: &'static str =
    "INSERT INTO events (relative_uri, method, headers, body) values (?1, ?2, ?3, ?4)";
static SELECT_EVENTS_SQL: &'static str =
    "SELECT id, relative_uri, method, headers, body, created_at from events WHERE id > ?1 AND created_at > ?2";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    if !Path::new("events.db").exists() {
        println!("Creating events.db from scratch");
        let conn = Connection::open("events.db")?;
        conn.execute(SCHEMA_SQL, NO_PARAMS)?;
    } else {
        println!("Found events.db");
    }
    let port = env::var("NAYOK_PORT").unwrap_or("8080".to_owned()).parse::<u16>()
        .expect("NAYOK_PORT should contain port number");
    let addr = ([0, 0, 0, 0], port).into();
    let service = make_service_fn(|_| async { Ok::<_, hyper::Error>(service_fn(routes)) });
    let server = Server::bind(&addr).serve(service);
    println!("Listening on http://{}", addr);
    server.await?;
    Ok(())
}

async fn routes(req: Request<Body>) -> Result<Response<Body>, Infallible> {
    let uri = req.uri().clone();
    let method = req.method().clone();
    let path = uri.path();
    let response = if path == "/" {
        Response::new(Body::from("Nayok operational"))
    } else if path.starts_with("/n/") {
        save_notification(req).await
    } else if path == "/notification-results" && method == Method::GET {
        load_notifications(req).await
    } else {
        not_found()
    };
    Ok(response)
}

fn not_found() -> Response<Body> {
    Response::builder().status(StatusCode::NOT_FOUND).body(Body::empty()).unwrap()
}

fn bad_request(message: &str) -> Response<Body> {
    Response::builder().status(StatusCode::BAD_REQUEST).body(Body::from(message.to_owned())).unwrap()
}

async fn save_notification(req: Request<Body>) -> Response<Body> {
    let method = req.method().clone();
    let uri = req.uri().path_and_query().unwrap().clone();
    let headers: HashMap<String, String> = req.headers().iter().map(|h| {
        (h.0.as_str().to_owned(), h.1.to_str().unwrap().to_owned())
    }).collect();
    let body = req.into_body();
    hyper::body::to_bytes(body).await.map(|body_bytes| {
        let body_vector = body_bytes.iter().cloned().collect::<Vec<u8>>();
        let body_base64 = base64::encode(&body_vector[..]);
        let data = EventCreationData {
            relative_uri: uri.to_string()[2..].to_owned(),
            method: method.to_string(),
            headers,
            body_base64,
        };
        println!("received {:?}", data);
        save_impl(&data).expect("Cannot save data");
        Response::new(Body::from("OK"))
    }).unwrap_or(bad_request("can't read body"))
}

async fn load_notifications(req: Request<Body>) -> Response<Body> {
    let params: HashMap<String, String> = req.uri().query().map(|query| {
        url::form_urlencoded::parse(query.as_bytes()).into_owned().collect()
    }).unwrap_or_else(HashMap::new);
    let default_id = "0".to_owned();
    let id_str = params.get("from_id").unwrap_or(&default_id);
    let default_date = "2000-01-01T00:00:00+00:00".to_owned();
    let date_str = params.get("from_date").unwrap_or(&default_date);
    let from_id = id_str.parse::<u32>();
    let from_date = DateTime::parse_from_rfc3339(date_str);
    match (from_id, from_date) {
        (Ok(id), Ok(date)) => {
            let events = load_impl(id, &date).unwrap();
            let result = serde_json::to_string(&events).unwrap();
            Response::new(Body::from(result))
        }
        (Err(err), _) => {
            let msg = format!("Parameter 'from_id' should be positive integer: {} {}", id_str, err);
            bad_request(&msg)
        }
        (_, Err(err)) => {
            let msg = format!("Parameter 'from_date' should be frc3339 date: {} {}", date_str, err);
            bad_request(&msg)
        }
    }
}

fn save_impl(data: &EventCreationData) -> Result<(), rusqlite::Error> {
    let conn = Connection::open("events.db").unwrap();
    let headers = serde_json::to_string(&data.headers).unwrap();
    let args = [
        data.relative_uri.as_str(),
        data.method.as_str(),
        headers.as_str(),
        data.body_base64.as_str()
    ];
    conn.execute(INSERT_EVENT_SQL, &args).unwrap();
    Ok(())
}

fn load_impl(id: u32, date: &DateTime<FixedOffset>) -> Result<Vec<Event>, rusqlite::Error> {
    let conn = Connection::open("events.db")?;
    let mut stmt = conn.prepare(SELECT_EVENTS_SQL)?;
    let date_str = date.format("%Y-%m-%d %H:%M:%S").to_string();
    let params: &[&dyn ToSql] = &[&id, &date_str];

    let events: Vec<Event> = stmt.query_map(params, |row| {
        let headers_str: String = row.get(3)?;
        Ok(Event {
            id: row.get(0)?,
            relative_uri: row.get(1)?,
            method: row.get(2)?,
            headers: serde_json::from_str(&headers_str).unwrap(),
            body_base64: row.get(4)?,
            created_at: row.get(5)?,
        })
    })?.map(|res| res.unwrap()).collect();

    Ok(events)
}

#[derive(Debug, Serialize, Deserialize)]
struct EventCreationData {
    relative_uri: String,
    method: String,
    headers: HashMap<String, String>,
    body_base64: String,
}

