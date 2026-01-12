use axum::{
    body::Body,
    http::{header, StatusCode, Response},
    response::IntoResponse,
    routing::get,
    Router,
};
use rust_embed::RustEmbed;
use std::net::SocketAddr;

// Includem fisierele din folderul "static" direct in binar
#[derive(RustEmbed)]
#[folder = "static/"]
struct Asset;

#[tokio::main]
async fn main() {
    let app = Router::new()
        .fallback(static_handler);

    let addr = SocketAddr::from(([0, 0, 0, 0], 8000));
    println!("Serverul ruleaza la http://{}", addr);
    println!("Se asteapta API-ul la http://localhost:8080");

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    ax_server(listener, app).await;
}

async fn ax_server(listener: tokio::net::TcpListener, app: Router) {
    axum::serve(listener, app).await.unwrap();
}

async fn static_handler(uri: axum::http::Uri) -> impl IntoResponse {
    let mut path = uri.path().trim_start_matches('/').to_string();

    if path.is_empty() {
        path = "index.html".to_string();
    }

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
