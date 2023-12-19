use http::{Method, StatusCode};

use http_body_util::{BodyExt, Full};
use hyper::body::Bytes;
use hyper::client::conn::http1::SendRequest;
use hyper::Request;
use hyper_util::rt::TokioIo;
use tokio::net::TcpStream;

// A simple type alias so as to DRY.
type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

async fn prepare_client(url: &str) -> Result<(hyper::Uri, SendRequest<Full<Bytes>>)> {
    // Parse url
    let url = url.parse::<hyper::Uri>().unwrap();
    if url.scheme_str() != Some("http") {
        return Err("This example only works with 'http' URLs.".into());
    }

    // Establish a TCP connection to the remote host.
    let host = url.host().expect("uri has no host");
    let port = url.port_u16().unwrap_or(80);
    let addr = format!("{}:{}", host, port);
    let stream = TcpStream::connect(addr).await?;
    let io = TokioIo::new(stream);

    // Start a task to drive the connection.
    let (sender, conn) = hyper::client::conn::http1::handshake::<_, Full<Bytes>>(io).await?;
    tokio::task::spawn(async move {
        if let Err(err) = conn.await {
            println!("Connection failed: {:?}", err);
        }
    });

    Ok((url, sender))
}

pub async fn fetch_url(
    url: &str,
    method: Method,
    data: Vec<u8>,
    content_type: Option<String>,
) -> Result<(StatusCode, Bytes, Option<String>)> {
    // Parse url
    let (url, mut sender) = prepare_client(url).await?;

    // Prepare the HTTP request.
    let authority = url.authority().unwrap().clone();

    // Build the request
    let req = Request::builder()
        .uri(url)
        .method(method)
        .header(hyper::header::HOST, authority.as_str())
        .header(
            hyper::header::CONTENT_TYPE,
            content_type.unwrap_or("application/octet-stream".to_string()),
        )
        .body(Full::new(Bytes::from(data)))?;

    // Send it and wait for the response.
    let res = sender.send_request(req).await?;

    // Analyze the response
    let status = res.status();
    let content_type = res
        .headers()
        .get("Content-Type")
        .map(|v| v.to_str().unwrap().to_string());
    let bytes = res.collect().await?.to_bytes();
    Ok((status, bytes, content_type))
}
