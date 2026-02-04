use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
    collections::HashMap,
    env,
    fs::OpenOptions,
    io::{self, Read, Write},
    path::PathBuf,
};
use tiny_http::{Server, Response, Method, StatusCode};

const ADDR: &str = "127.0.0.1";
const PORT: u16 = 19633;
const YOMITAN_API_VERSION: u8 = 1;

#[derive(Serialize)]
struct OutgoingMessage {
    action: String,
    params: HashMap<String, Vec<String>>,
    body: String,
}

#[derive(Deserialize)]
struct IncomingMessage {
    #[serde(rename = "responseStatusCode")]
    response_status_code: u16,
    data: Value,
}

fn client_dir() -> PathBuf {
    env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(PathBuf::from))
        .unwrap_or_else(|| ".".into())
}

fn log(message: &str) {
    // TODO: redirect stderr to error.log, then use eprintln. that way panics also get sent
    let path = client_dir().join("yomitan-api.log");
    if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(path) {
        let ts = chrono::Utc::now().format("%Y-%m-%d_%H-%M-%S");
        let _ = writeln!(file, "{ts} UTC: {message}");
    }
}

fn send_native(msg: &OutgoingMessage) -> io::Result<()> {
    let json = serde_json::to_vec(msg)?;
    let len = (json.len() as u32).to_ne_bytes();
    let mut out = io::stdout().lock();
    out.write_all(&len)?;
    out.write_all(&json)?;
    out.flush()
}

fn recv_native() -> io::Result<IncomingMessage> {
    let mut len_buf = [0u8; 4];
    io::stdin().read_exact(&mut len_buf)?;
    let len = u32::from_ne_bytes(len_buf) as usize;
    let mut buf = vec![0; len];
    io::stdin().read_exact(&mut buf)?;
    Ok(serde_json::from_slice(&buf)?)
}

fn parse_query_string(query: &str) -> HashMap<String, Vec<String>> {
    let mut params = HashMap::new();
    if query.is_empty() {
        return params;
    }
    for pair in query.split('&') {
        if pair.is_empty() {
            continue;
        }
        let (key, value) = pair.split_once('=').unwrap_or((pair, ""));
        let key = urlencoding::decode(key).unwrap_or_default().to_string();
        let value = urlencoding::decode(value).unwrap_or_default().to_string();
        params.entry(key).or_insert_with(Vec::new).push(value);
    }
    params
}

fn response(value: Option<Value>, status: u16) -> Response<std::io::Cursor<Vec<u8>>> {
    let (json, content_type) = match value {
        Some(v) => (serde_json::to_string(&v).unwrap(), "application/json"),
        None => (String::new(), "text/plain"),
    };
    
    Response::from_string(json)
        .with_status_code(StatusCode(status))
        .with_header(tiny_http::Header::from_bytes(&b"Content-Type"[..], content_type.as_bytes()).unwrap())
        .with_header(tiny_http::Header::from_bytes(&b"Access-Control-Allow-Origin"[..], &b"*"[..]).unwrap())
        .with_header(tiny_http::Header::from_bytes(&b"Cache-Control"[..], &b"no-store, no-cache, must-revalidate"[..]).unwrap())
}

fn handle_request(path: &str, query: HashMap<String, Vec<String>>, body: Vec<u8>) -> Response<std::io::Cursor<Vec<u8>>> {
    let path = path.trim_start_matches('/');
    
    if path == "serverVersion" || path.is_empty() {
        return response(Some(json!({ "version": YOMITAN_API_VERSION })), 200);
    }

    let outgoing = OutgoingMessage {
        action: path.to_string(),
        params: query,
        body: String::from_utf8_lossy(&body).to_string(),
    };

    if let Err(e) = send_native(&outgoing) {
        log(&format!("native send failed: {}", e));
        return response(None, 500);
    }

    match recv_native() {
        Ok(resp) => response(Some(resp.data), resp.response_status_code),
        Err(e) => {
            log(&format!("native recv failed: {}", e));
            response(None, 500)
        }
    }
}

fn main() -> std::io::Result<()> {
    log(&format!("yomitan-api-rs {} - starting up", env!("CARGO_PKG_VERSION")));
    
    let server = Server::http(format!("{}:{}", ADDR, PORT))
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    
    for mut request in server.incoming_requests() {
        let resp = if request.method() == &Method::Post {
        let url = request.url().to_string(); 
        let (path, query_str) = url.split_once('?').unwrap_or((&url, ""));
        let query = parse_query_string(query_str);
        let mut body = Vec::new();
        let _ = request.as_reader().read_to_end(&mut body);
        handle_request(path, query, body)
    } else {
        response(None, 400)
    };

    let _ = request.respond(resp);
}

    Ok(())
}