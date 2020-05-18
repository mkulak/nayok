use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Method, Request, Response, Server, StatusCode};
use hyper::http::Version;
use bytes::{Bytes};

trait Signed {
    fn foo(self) -> i32;
}

impl Signed for i32 {
    fn foo(self) -> i32 {
        self.abs()
    }
}

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

fn to_str(v: Version) -> &'static str {
    match v {
        Version::HTTP_10 => "HTTP 1.0",
        Version::HTTP_11 => "HTTP 1.1",
        Version::HTTP_2 => "HTTP 2.0",
        Version::HTTP_3 => "HTTP 3.0",
        _ => panic!("Unknown http version")
    }
}

async fn routes(req: Request<Body>) -> Result<Response<Body>, hyper::Error> {
    let version = req.version();
    // let version_str = to_str(version).to_string();
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

    match (method, uri.path()) {
        (Method::GET, "/") => Ok(Response::new(Body::from("Hello"))),
        (Method::POST, "/notifications") => Ok(Response::new(Body::from("OK"))),
        (Method::GET, "/notification-results") => {
            // let chunk_stream = whole_body.map_ok(|chunk| {
            //     chunk
            //         .iter()
            //         .map(|byte| byte.to_ascii_uppercase())
            //         .collect::<Vec<u8>>()
            // });
            // Ok(Response::new(Body::wrap_stream(chunk_stream)))
            Ok(Response::new(Body::from("OK")))
        }

        (Method::POST, "/echo/reversed") => {
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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("{}", -345.foo());
    let addr = ([127, 0, 0, 1], 3000).into();
    let service = make_service_fn(|_| async { Ok::<_, hyper::Error>(service_fn(routes)) });
    let server = Server::bind(&addr).serve(service);
    println!("Listening on http://{}", addr);
    server.await?;
    Ok(())
}