use std::env;
use std::net::SocketAddr;

use http_body_util::{combinators::BoxBody, BodyExt};
use http_body_util::{Empty, Full};
use hyper::body::Bytes;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Method, StatusCode};
use hyper::{Request, Response};
use hyper_util::rt::TokioIo;
use serde_json::Value;
use tokio::net::{TcpListener, TcpStream};

async fn transform(
    req: Request<hyper::body::Incoming>,
) -> Result<Response<BoxBody<Bytes, hyper::Error>>, hyper::Error> {
    let base_url = match env::var("GRAPHQL_TRANSFORMER_BASE_URL") {
        Ok(value) => value,
        Err(_) => {
            return Ok(Response::new(full("GRAPHQL_TRANSFORMER_BASE_URL not set")));
        }
    };

    match req.method() {
        &Method::POST => {
            let body_bytes = req.collect().await?.to_bytes();
            let body_str = std::str::from_utf8(&body_bytes).unwrap();

            let json_result: Result<Value, serde_json::Error> = serde_json::from_str(body_str);

            let json = match json_result {
                Ok(value) => value,
                Err(err) => {
                    eprintln!("Error deserializing JSON: {}", err);

                    return Ok(Response::new(full("Error deserializing JSON")));
                }
            };

            let qs = serde_qs::to_string(&json).unwrap();
            let url = format!("{}?{}", base_url, qs);
            let url = url.parse::<hyper::Uri>().unwrap();

            fetch_url(url).await
        }
        _ => {
            let mut not_found = Response::new(empty());
            *not_found.status_mut() = StatusCode::NOT_FOUND;
            Ok(not_found)
        }
    }
}

async fn fetch_url(
    url: hyper::Uri,
) -> Result<Response<BoxBody<Bytes, hyper::Error>>, hyper::Error> {
    let host = url.host().expect("uri has no host");
    let port = url.port_u16().unwrap_or(80);
    let addr = format!("{}:{}", host, port);
    let stream = match TcpStream::connect(addr).await {
        Ok(value) => value,
        Err(err) => {
            eprintln!("Connection failed: {}", err);

            return Ok(Response::new(full("Connection failed")));
        }
    };
    let io = TokioIo::new(stream);

    let (mut sender, conn) = hyper::client::conn::http1::handshake(io).await?;

    tokio::task::spawn(async move {
        if let Err(err) = conn.await {
            println!("Connection failed: {:?}", err);
        }
    });

    let authority = url.authority().unwrap().clone();

    let req = Request::builder()
        .uri(url)
        .header(hyper::header::HOST, authority.as_str())
        .body(Empty::<Bytes>::new())
        .unwrap();

    let res = match sender.send_request(req).await {
        Ok(value) => value,
        Err(err) => {
            eprintln!("Could not send request: {}", err);

            return Ok(Response::new(full("Could not send request")));
        }
    };

    let body_bytes = res.collect().await?.to_bytes();

    Ok(Response::new(full(body_bytes)))
}

fn empty() -> BoxBody<Bytes, hyper::Error> {
    Empty::<Bytes>::new()
        .map_err(|never| match never {})
        .boxed()
}
fn full<T: Into<Bytes>>(chunk: T) -> BoxBody<Bytes, hyper::Error> {
    Full::new(chunk.into())
        .map_err(|never| match never {})
        .boxed()
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let addr = SocketAddr::from(([127, 0, 0, 1], 80));
    let listener = TcpListener::bind(addr).await?;

    loop {
        let (stream, _) = listener.accept().await?;

        let io = TokioIo::new(stream);

        tokio::task::spawn(async move {
            if let Err(err) = http1::Builder::new()
                .serve_connection(io, service_fn(transform))
                .await
            {
                println!("Error serving connection: {:?}", err);
            }
        });
    }
}
