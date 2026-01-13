use axum::{
    body::Body,
    extract::{Path, Request, State},
    http::{header, StatusCode, Response, Uri},
    response::IntoResponse,
    routing::any,
    Router,
};
use rust_embed::RustEmbed;
use std::net::SocketAddr;
use std::sync::Arc;

const PORT: u16 = 80;
const GPIO_API_URL: &str = "http://127.0.0.1:8080";

#[derive(RustEmbed)]
#[folder = "static/"]
struct Asset;

struct AppState {
    http_client: reqwest::Client,
}

async fn proxy_handler(
    State(state): State<Arc<AppState>>,
    Path(path): Path<String>,
    req: Request,
) -> impl IntoResponse {
    // Reconstructia query string-ul (?param=value)
    let query = req.uri().query().map(|q| format!("?{}", q)).unwrap_or_default();
    
    let target_url = format!("{}/api/{}{}", GPIO_API_URL, path, query);

    let method = req.method().clone();
    let headers = req.headers().clone();
    
    let body_stream = req.into_body().into_data_stream();
    let reqwest_body = reqwest::Body::wrap_stream(body_stream);

    let response = state.http_client
        .request(method, &target_url)
        .headers(headers)
        .body(reqwest_body)
        .send()
        .await;

    match response {
        Ok(res) => {
            let mut client_resp = Response::builder().status(res.status());
            
            for (name, value) in res.headers() {
                client_resp = client_resp.header(name, value);
            }

            client_resp
                .body(Body::from_stream(res.bytes_stream()))
                .unwrap()
        }
        Err(e) => {
            eprintln!("Proxy error: {}", e);
            Response::builder()
                .status(StatusCode::BAD_GATEWAY)
                .body(Body::from("GPIO Service Unavailable"))
                .unwrap()
        }
    }
}

async fn static_handler(uri: Uri) -> impl IntoResponse {
    let mut path = uri.path().trim_start_matches('/').to_string();
    if path.is_empty() { path = "index.html".to_string(); }

    match Asset::get(&path) {
        Some(content) => {
            let mime = mime_guess::from_path(&path).first_or_octet_stream();
            Response::builder()
                .header(header::CONTENT_TYPE, mime.as_ref())
                .body(Body::from(content.data))
                .unwrap()
        }
        None => Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::from("404 Not Found"))
            .unwrap(),
    }
}

#[tokio::main]
async fn main() {
    let state = Arc::new(AppState {
        http_client: reqwest::Client::new(),
    });

    let app = Router::new()
        .route("/api/{*path}", any(proxy_handler))
        .fallback(static_handler)
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], PORT));
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
