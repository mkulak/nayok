use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Method, Request, Response, Server, StatusCode};
use hyper::http::Version;
use bytes::Bytes;
use rusqlite::{Connection, Result};
use rusqlite::NO_PARAMS;
use std::collections::HashMap;

trait ToString {
    fn to_str(&self) -> &'static str;
}

impl ToString for Version {
    fn to_str(&self) -> &'static str {
        match *self {
            Version::HTTP_10 => "HTTP 1.0",
            Version::HTTP_11 => "HTTP 1.1",
            Version::HTTP_2 => "HTTP 2.0",
            Version::HTTP_3 => "HTTP 3.0",
            _ => panic!("Unknown http version")
        }
    }
}

static SCHEMA_SQL: &'static str = include_str!("schema.sql");

async fn routes(req: Request<Body>) -> Result<Response<Body>, hyper::Error> {
    let version = req.version();
    let version_str = (&version).to_str().to_string();
    let method = req.method().clone();
    let uri = req.uri().clone();
    println!("{} {} {}", &version_str, method.as_str(), uri);
    req.headers().iter().for_each(|h| {
        println!("{}: {}", h.0.as_str(), h.1.to_str().unwrap());
    });
    let body = req.into_body();
    let whole_body: Bytes = hyper::body::to_bytes(body).await?;
    let vec = whole_body.iter().cloned().collect::<Vec<u8>>();
    vec.iter().for_each(|b| {
        print!("{},", *b)
    });
    println!();

    match uri.path() {
        "/" => Ok(Response::new(Body::from("Hello"))),
        "/notifications" => Ok(Response::new(Body::from("OK"))),
        "/notification-results" if method == Method::GET => {
            // let chunk_stream = whole_body.map_ok(|chunk| {
            //     chunk
            //         .iter()
            //         .map(|byte| byte.to_ascii_uppercase())
            //         .collect::<Vec<u8>>()
            // });
            // Ok(Response::new(Body::wrap_stream(chunk_stream)))
            Ok(Response::new(Body::from("OK")))
        }

        "/echo/reversed" if method == Method::POST => {
            let reversed_body = whole_body.iter().rev().cloned().collect::<Vec<u8>>();
            Ok(Response::new(Body::from(reversed_body)))
        }

        // Return the 404 Not Found for other routes.
        _ => {
            let mut not_found = Response::default();
            *not_found.status_mut() = StatusCode::NOT_FOUND;
            Ok(not_found)
        }
    }
}

// #[tokio::main]
// async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
//     let addr = ([127, 0, 0, 1], 3000).into();
//     let service = make_service_fn(|_| async { Ok::<_, hyper::Error>(service_fn(routes)) });
//     let server = Server::bind(&addr).serve(service);
//     println!("Listening on http://{}", addr);
//     server.await?;
//     Ok(())
// }

#[derive(Debug)]
struct Cat {
    name: String,
    color: String,
}

fn main() -> Result<()> {
    let conn = Connection::open("events.db")?;
    conn.execute(SCHEMA_SQL, NO_PARAMS)?;
    let headers = "Foo: Bar;Bax: Boo";
    let method = "GET";
    let relative_uri = "/p1/p2?blah=bom";
    // let body: Vec<u8> = vec![1, 2, 3];
    conn.execute(
        "INSERT INTO events (relative_uri, method, headers) values (?1, ?2, ?3)",
        &[&relative_uri, &method, &headers],
    )? ;
    let last_id = conn.last_insert_rowid();
    println!("last_id: {}", last_id);


    // let mut cat_colors = HashMap::new();
    // cat_colors.insert(String::from("Blue"), vec!["Tigger", "Sammy"]);
    // cat_colors.insert(String::from("Black"), vec!["Oreo", "Biscuit"]);
    //
    // for (color, catnames) in &cat_colors {
    //     conn.execute(
    //         "INSERT INTO cat_colors (name) values (?1)",
    //         &[&color.to_string()],
    //     )?;
    //     let last_id: String = conn.last_insert_rowid().to_string();
    //
    //     for cat in catnames {
    //         conn.execute(
    //             "INSERT INTO cats (name, color_id) values (?1, ?2)",
    //             &[&cat.to_string(), &last_id],
    //         )?;
    //     }
    // }
    // let mut stmt = conn.prepare(
    //     "SELECT c.name, cc.name from cats c
    //      INNER JOIN cat_colors cc
    //      ON cc.id = c.color_id;",
    // )?;
    //
    // let cats = stmt.query_map(NO_PARAMS, |row| {
    //     Ok(Cat {
    //         name: row.get(0)?,
    //         color: row.get(1)?,
    //     })
    // })?;
    //
    // for cat in cats {
    //     println!("Found cat {:?}", cat);
    // }
    //

    Ok(())
}