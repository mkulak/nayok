#![allow(dead_code)]

use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Method, Request, Response, Server, StatusCode};
use hyper::http::Version;
use bytes::Bytes;
use rusqlite::{Connection, Result, ToSql};
use rusqlite::NO_PARAMS;
use std::collections::HashMap;
use base64;
use chrono::{DateTime, NaiveDateTime, Utc};

static SCHEMA_SQL: &'static str = include_str!("schema.sql");

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let addr = ([127, 0, 0, 1], 3000).into();
    let service = make_service_fn(|_| async { Ok::<_, hyper::Error>(service_fn(routes)) });
    let server = Server::bind(&addr).serve(service);
    println!("Listening on http://{}", addr);
    server.await?;
    Ok(())
}

async fn save_notification(req: Request<Body>) -> Result<Response<Body>, hyper::Error> {
    let method = req.method().clone();
    let uri = req.uri().path_and_query().unwrap().clone();
    let mut headers = String::new();
    req.headers().iter().for_each(|h| {
        let res = format!("{}:{};", h.0.as_str(), h.1.to_str().unwrap());
        headers.push_str(res.as_str())
    });

    let body = req.into_body();
    let body_bytes = hyper::body::to_bytes(body).await?;
    let body_vector = body_bytes.iter().cloned().collect::<Vec<u8>>();
    let body_base64 = base64::encode(&body_vector[..]);

    let conn = Connection::open("events.db").unwrap();
    conn.execute(
        "INSERT INTO events (relative_uri, method, headers, body) values (?1, ?2, ?3, ?4)",
        &[uri.as_str(), method.as_str(), headers.as_str(), body_base64.as_str()],
    ).unwrap();
    let last_id = conn.last_insert_rowid();
    println!("last_id: {}", last_id);

    Ok(Response::new(Body::from("OK")))
}

async fn routes(req: Request<Body>) -> Result<Response<Body>, hyper::Error> {
    let uri = req.uri().clone();
    let method = req.method().clone();
    match uri.path() {
        "/" => Ok(Response::new(Body::from("Hello"))),
        "/notifications" => save_notification(req).await,
        "/notification-results" if method == Method::GET => Ok(Response::new(Body::from("OK"))),
        _ => not_found()
    }
}

fn not_found() -> Result<Response<Body>, hyper::Error> {
    let mut not_found = Response::default();
    *not_found.status_mut() = StatusCode::NOT_FOUND;
    Ok(not_found)
}

fn main2() -> Result<()> {
    let conn = Connection::open("events.db")?;
    let mut stmt = conn.prepare(
        "SELECT id, relative_uri, method, headers, body, created_at from events WHERE id > ?1 AND created_at > ?2"
    )?;
    let date = DateTime::parse_from_rfc3339("2020-05-22T19:28:00+00:00").unwrap();
    let d = date.format("%Y-%m-%d %H:%M:%S").to_string();
    let params: &[&dyn ToSql] = &[&2, &d];

    let events = stmt.query_map(params, |row| {
        Ok(Event {
            id: row.get(0)?,
            relative_uri: row.get(1)?,
            method: row.get(2)?,
            headers: row.get(3)?,
            body_base64: row.get(4)?,
            created_at: row.get(5)?,
        })
    })?.map(|res| { res.unwrap() });

    for event in events {
        println!("{:?}", event.created_at.to_rfc3339());
    }

    Ok(())
}

#[derive(Debug)]
struct Event {
    id: u32,
    relative_uri: String,
    method: String,
    headers: String,
    body_base64: String,
    created_at: DateTime<Utc>
}