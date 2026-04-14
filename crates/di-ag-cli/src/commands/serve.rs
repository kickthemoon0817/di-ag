use serde::Deserialize;
use std::io::Read;
use std::sync::Arc;
use std::thread;

const INDEX_HTML: &str = include_str!("../../../../web/public/index.html");

#[derive(Deserialize)]
struct RenderRequest {
    dsl: String,
    #[serde(default)]
    theme: Option<String>,
    #[serde(default)]
    inspect: bool,
}

pub fn run(port: u16, open: bool) -> Result<(), String> {
    let addr = format!("127.0.0.1:{}", port);
    let server = tiny_http::Server::http(&addr)
        .map_err(|e| format!("Failed to start server: {}", e))?;
    let server = Arc::new(server);

    eprintln!("di-ag editor running at http://localhost:{}", port);

    if open {
        let _ = std::process::Command::new("open")
            .arg(format!("http://localhost:{}", port))
            .spawn()
            .or_else(|_| {
                std::process::Command::new("xdg-open")
                    .arg(format!("http://localhost:{}", port))
                    .spawn()
            })
            .or_else(|_| {
                std::process::Command::new("cmd")
                    .args(["/c", "start", &format!("http://localhost:{}", port)])
                    .spawn()
            });
    }

    // Small pool of worker threads so one slow render does not block others.
    let workers = 4;
    let mut handles = Vec::with_capacity(workers);
    for _ in 0..workers {
        let s = Arc::clone(&server);
        handles.push(thread::spawn(move || {
            for request in s.incoming_requests() {
                handle_request(request);
            }
        }));
    }
    for h in handles {
        let _ = h.join();
    }
    Ok(())
}

fn handle_request(mut request: tiny_http::Request) {
    let url = request.url().to_string();
    let method = request.method().to_string();
    // Content-Security-Policy for the single-page editor: no external scripts,
    // no framing, local style. Matches the inline <script> we ship in index.html.
    let csp = "Content-Security-Policy: default-src 'self'; style-src 'self' 'unsafe-inline'; script-src 'self' 'unsafe-inline'; img-src 'self' data:; connect-src 'self'; frame-ancestors 'none'"
        .parse::<tiny_http::Header>()
        .unwrap();

    match (method.as_str(), url.as_str()) {
        ("GET", "/") | ("GET", "/index.html") => {
            let response = tiny_http::Response::from_string(INDEX_HTML)
                .with_header(
                    "Content-Type: text/html; charset=utf-8"
                        .parse::<tiny_http::Header>()
                        .unwrap(),
                )
                .with_header(csp);
            if let Err(e) = request.respond(response) {
                eprintln!("Response error: {}", e);
            }
        }
        ("POST", "/api/render") => {
            let json_header = "Content-Type: application/json"
                .parse::<tiny_http::Header>()
                .unwrap();

            let mut body = String::new();
            match request.as_reader().take(1_048_577).read_to_string(&mut body) {
                Ok(n) if n > 1_048_576 => {
                    let err_body = serde_json::json!({"error": "Request body too large (max 1 MB)"}).to_string();
                    let _ = request.respond(
                        tiny_http::Response::from_string(err_body)
                            .with_status_code(413)
                            .with_header(json_header),
                    );
                    return;
                }
                Err(_) => {
                    let err_body =
                        serde_json::json!({"error": "Failed to read request body"}).to_string();
                    let _ = request.respond(
                        tiny_http::Response::from_string(err_body)
                            .with_status_code(400)
                            .with_header(json_header),
                    );
                    return;
                }
                Ok(_) => {}
            }

            let req: RenderRequest = match serde_json::from_str(&body) {
                Ok(r) => r,
                Err(_) => {
                    let err_body =
                        serde_json::json!({"error": "Invalid JSON request body"}).to_string();
                    let _ = request.respond(
                        tiny_http::Response::from_string(err_body)
                            .with_status_code(400)
                            .with_header(json_header),
                    );
                    return;
                }
            };

            match handle_render(&req) {
                Ok(json) => {
                    let response = tiny_http::Response::from_string(json).with_header(json_header);
                    if let Err(e) = request.respond(response) {
                        eprintln!("Response error: {}", e);
                    }
                }
                Err(e) => {
                    // Expose rendering errors back to the editor so users can see
                    // DSL parse messages — these are not sensitive, the editor
                    // is a local-only tool.
                    let err_body = serde_json::json!({"error": e}).to_string();
                    let _ = request.respond(
                        tiny_http::Response::from_string(err_body)
                            .with_status_code(400)
                            .with_header(json_header),
                    );
                }
            }
        }
        _ => {
            let err_body = serde_json::json!({"error": "Not found"}).to_string();
            let _ = request.respond(
                tiny_http::Response::from_string(err_body)
                    .with_status_code(404)
                    .with_header(
                        "Content-Type: application/json"
                            .parse::<tiny_http::Header>()
                            .unwrap(),
                    ),
            );
        }
    }
}

fn handle_render(req: &RenderRequest) -> Result<String, String> {
    let doc = di_ag_dsl::parse(&req.dsl).map_err(|e| format!("Parse error: {}", e))?;

    let doc = di_ag_layout::layout(doc).map_err(|e| format!("Layout error: {}", e))?;

    let theme = req.theme.as_deref();
    let svg = di_ag_render::render_svg_with_theme(&doc, theme)
        .map_err(|e| format!("Render error: {}", e))?;

    if req.inspect {
        let report = di_ag_inspect::inspect(&doc);
        let result = serde_json::json!({
            "svg": svg,
            "inspection": report,
        });
        serde_json::to_string(&result).map_err(|e| format!("JSON error: {}", e))
    } else {
        let result = serde_json::json!({ "svg": svg });
        serde_json::to_string(&result).map_err(|e| format!("JSON error: {}", e))
    }
}
