use axum::extract::{DefaultBodyLimit, FromRequest, Multipart, Request, State};
use axum::http::{header, HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use miq_render::{
    encode_png, render_quote, set_assets_dir, FontCatalog, QuoteRequest, RenderOptions,
};
use serde::Serialize;
use std::net::SocketAddr;
use std::sync::Arc;

struct AppState {
    catalog: FontCatalog,
    token: Option<String>,
}

#[derive(Serialize)]
struct ErrorBody {
    error: &'static str,
    message: String,
}

fn err(status: StatusCode, code: &'static str, message: impl Into<String>) -> Response {
    (
        status,
        Json(ErrorBody {
            error: code,
            message: message.into(),
        }),
    )
        .into_response()
}

async fn healthz() -> &'static str {
    "ok"
}

fn check_auth(headers: &HeaderMap, token: &Option<String>) -> Result<(), Response> {
    let Some(expected) = token else {
        return Ok(());
    };
    let Some(auth) = headers
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
    else {
        return Err(err(
            StatusCode::UNAUTHORIZED,
            "unauthorized",
            "missing bearer token",
        ));
    };
    let got = auth.strip_prefix("Bearer ").unwrap_or("");
    if got != expected {
        return Err(err(
            StatusCode::UNAUTHORIZED,
            "unauthorized",
            "invalid token",
        ));
    }
    Ok(())
}

fn render_response(state: &AppState, opts: RenderOptions, avatar: Option<&[u8]>) -> Response {
    if opts.text.is_empty() || opts.text.chars().count() > 4000 {
        return err(
            StatusCode::BAD_REQUEST,
            "invalid_field",
            "text length out of range",
        );
    }
    match render_quote(&state.catalog, &opts, avatar) {
        Ok(out) => match encode_png(&out.image) {
            Ok(png) => {
                let mut res = Response::new(png.into());
                *res.status_mut() = StatusCode::OK;
                let h = res.headers_mut();
                h.insert(header::CONTENT_TYPE, "image/png".parse().unwrap());
                h.insert(header::CACHE_CONTROL, "no-store".parse().unwrap());
                h.insert(
                    "x-quote-font-size",
                    out.layout.size.to_string().parse().unwrap(),
                );
                h.insert(
                    "x-quote-lines",
                    out.layout.lines.len().to_string().parse().unwrap(),
                );
                h.insert(
                    "x-quote-truncated",
                    out.layout.truncated.to_string().parse().unwrap(),
                );
                res
            }
            Err(e) => err(
                StatusCode::INTERNAL_SERVER_ERROR,
                "render_failed",
                e.to_string(),
            ),
        },
        Err(e) => err(
            StatusCode::UNPROCESSABLE_ENTITY,
            "avatar_decode_failed",
            e.to_string(),
        ),
    }
}

fn prefer_discord_avatar_512(url: &str) -> String {
    let Ok(mut parsed) = reqwest::Url::parse(url) else {
        return url.to_string();
    };
    let host = parsed.host_str().unwrap_or("").to_ascii_lowercase();
    let is_discord = host == "cdn.discordapp.com"
        || host == "media.discordapp.net"
        || host.ends_with(".discordapp.com")
        || host.ends_with(".discordapp.net");
    if !is_discord {
        return url.to_string();
    }
    let path = parsed.path().to_ascii_lowercase();
    if !(path.contains("/avatars/") || path.contains("/icons/") || path.contains("/a_")) {
        return url.to_string();
    }
    let mut pairs: Vec<(String, String)> = parsed
        .query_pairs()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .filter(|(k, _)| k != "size")
        .collect();
    pairs.push(("size".into(), "512".into()));
    parsed.set_query(None);
    parsed
        .query_pairs_mut()
        .extend_pairs(pairs.iter().map(|(k, v)| (k.as_str(), v.as_str())));
    parsed.to_string()
}

async fn fetch_avatar(url: &str) -> Result<Vec<u8>, Response> {
    if !url.starts_with("https://") {
        return Err(err(
            StatusCode::BAD_REQUEST,
            "invalid_field",
            "avatar_url must be https",
        ));
    }
    let url = prefer_discord_avatar_512(url);
    let parsed = reqwest::Url::parse(&url)
        .map_err(|e| err(StatusCode::BAD_REQUEST, "invalid_field", e.to_string()))?;
    if let Some(host) = parsed.host_str() {
        if let Ok(list) = std::env::var("MIQ_AVATAR_HOSTS") {
            let allowed: Vec<_> = list
                .split(',')
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
                .collect();
            if !allowed.is_empty() && !allowed.iter().any(|h| *h == host) {
                return Err(err(
                    StatusCode::BAD_REQUEST,
                    "invalid_field",
                    "avatar host not allowed",
                ));
            }
        }
        if looks_local_host(host) {
            return Err(err(
                StatusCode::BAD_REQUEST,
                "invalid_field",
                "avatar host blocked",
            ));
        }
    }

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .redirect(reqwest::redirect::Policy::limited(3))
        .build()
        .map_err(|e| {
            err(
                StatusCode::BAD_GATEWAY,
                "avatar_fetch_failed",
                e.to_string(),
            )
        })?;

    let resp = client.get(parsed).send().await.map_err(|e| {
        err(
            StatusCode::BAD_GATEWAY,
            "avatar_fetch_failed",
            e.to_string(),
        )
    })?;
    if !resp.status().is_success() {
        return Err(err(
            StatusCode::BAD_GATEWAY,
            "avatar_fetch_failed",
            "non-success status",
        ));
    }
    let bytes = resp.bytes().await.map_err(|e| {
        err(
            StatusCode::BAD_GATEWAY,
            "avatar_fetch_failed",
            e.to_string(),
        )
    })?;
    if bytes.len() > 8 * 1024 * 1024 {
        return Err(err(
            StatusCode::PAYLOAD_TOO_LARGE,
            "payload_too_large",
            "avatar too large",
        ));
    }
    Ok(bytes.to_vec())
}

fn looks_local_host(host: &str) -> bool {
    let h = host.to_ascii_lowercase();
    h == "localhost"
        || h.ends_with(".localhost")
        || h == "127.0.0.1"
        || h == "::1"
        || h.starts_with("10.")
        || h.starts_with("192.168.")
        || h.starts_with("169.254.")
}

async fn quote_json(state: &AppState, headers: &HeaderMap, req: QuoteRequest) -> Response {
    if let Err(e) = check_auth(headers, &state.token) {
        return e;
    }
    let avatar = match &req.avatar_url {
        Some(url) => match fetch_avatar(url).await {
            Ok(b) => Some(b),
            Err(e) => return e,
        },
        None => None,
    };
    render_response(state, req.into(), avatar.as_deref())
}

async fn quote_multipart(
    state: &AppState,
    headers: HeaderMap,
    mut multipart: Multipart,
) -> Response {
    if let Err(e) = check_auth(&headers, &state.token) {
        return e;
    }
    let mut text = None;
    let mut display_name = None;
    let mut username = String::new();
    let mut watermark = String::new();
    let mut avatar_url = None;
    let mut avatar_bytes = None;

    while let Ok(Some(field)) = multipart.next_field().await {
        let name = field.name().unwrap_or("").to_string();
        match name.as_str() {
            "text" => text = field.text().await.ok(),
            "display_name" => display_name = field.text().await.ok(),
            "username" => username = field.text().await.unwrap_or_default(),
            "watermark" => watermark = field.text().await.unwrap_or_default(),
            "avatar_url" => avatar_url = field.text().await.ok(),
            "avatar" => {
                let bytes = field.bytes().await.unwrap_or_default();
                if bytes.len() > 8 * 1024 * 1024 {
                    return err(
                        StatusCode::PAYLOAD_TOO_LARGE,
                        "payload_too_large",
                        "avatar too large",
                    );
                }
                avatar_bytes = Some(bytes.to_vec());
            }
            _ => {}
        }
    }

    let (Some(text), Some(display_name)) = (text, display_name) else {
        return err(
            StatusCode::BAD_REQUEST,
            "invalid_field",
            "text and display_name required",
        );
    };

    if avatar_bytes.is_none() {
        if let Some(url) = avatar_url {
            match fetch_avatar(&url).await {
                Ok(b) => avatar_bytes = Some(b),
                Err(e) => return e,
            }
        }
    }

    let opts = RenderOptions {
        text,
        display_name,
        username,
        watermark,
    };
    render_response(state, opts, avatar_bytes.as_deref())
}

async fn quote_any(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    req: Request,
) -> Response {
    let ct = headers
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_string();
    if ct.starts_with("multipart/") {
        match Multipart::from_request(req, &state).await {
            Ok(mp) => quote_multipart(&state, headers, mp).await,
            Err(e) => err(StatusCode::BAD_REQUEST, "invalid_field", e.to_string()),
        }
    } else {
        match Json::<QuoteRequest>::from_request(req, &state).await {
            Ok(Json(body)) => quote_json(&state, &headers, body).await,
            Err(e) => err(StatusCode::BAD_REQUEST, "invalid_field", e.to_string()),
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();
    if let Ok(dir) = std::env::var("MIQ_ASSETS") {
        set_assets_dir(dir);
    }
    let catalog = FontCatalog::load_default()?;
    let state = Arc::new(AppState {
        catalog,
        token: std::env::var("MIQ_TOKEN").ok().filter(|s| !s.is_empty()),
    });

    let app = Router::new()
        .route("/healthz", get(healthz))
        .route("/v1/quote", post(quote_any))
        .layer(DefaultBodyLimit::max(10 * 1024 * 1024))
        .with_state(state);

    let addr: SocketAddr = std::env::var("MIQ_LISTEN")
        .unwrap_or_else(|_| "0.0.0.0:8080".into())
        .parse()?;
    tracing::info!("listening on {addr}");
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
