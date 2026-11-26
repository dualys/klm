use crate::{persist_import, reflection_log};
use klm::{Kognitara, current_timestamp, family::MemberId};
use serde::Deserialize;
use serde_json::json;
use std::io::Read;
use std::path::Path;
use std::process::Command;
use tiny_http::{Header, Method, Request, Response, Server, StatusCode};

const MAX_BODY: usize = 16 * 1024;
const CSP: &str = "default-src 'none'; script-src 'self'; style-src 'self'; connect-src 'self'; base-uri 'none'; frame-ancestors 'none'; form-action 'self'";

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ChatMessage {
    message: String,
}

#[derive(Deserialize)]
struct SystemTimer {
    unit: String,
    next: Option<u64>,
}

fn parse_timers(output: &[u8]) -> Result<serde_json::Value, String> {
    let timers: Vec<SystemTimer> =
        serde_json::from_slice(output).map_err(|error| format!("Invalid timer data: {error}"))?;
    let entry = |unit: &str| match timers.iter().find(|timer| timer.unit == unit) {
        Some(timer) => match timer.next {
            Some(next) if next > 0 => json!({ "status": "scheduled", "next_ms": next / 1000 }),
            _ => json!({ "status": "unscheduled", "next_ms": null }),
        },
        None => json!({ "status": "missing", "next_ms": null }),
    };
    Ok(json!({
        "research": entry("klm-research.timer"),
        "backup": entry("klm-backup.timer"),
    }))
}

fn timers() -> Result<serde_json::Value, String> {
    let output = Command::new("systemctl")
        .args([
            "--user",
            "list-timers",
            "--all",
            "--output=json",
            "--no-pager",
            "klm-research.timer",
            "klm-backup.timer",
        ])
        .output()
        .map_err(|error| format!("Cannot run systemctl: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "Cannot read systemd timers: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    parse_timers(&output.stdout)
}

fn header<'a>(request: &'a Request, name: &'static str) -> Option<&'a str> {
    request
        .headers()
        .iter()
        .find(|header| header.field.equiv(name))
        .map(|header| header.value.as_str())
}

fn reply(request: Request, status: u16, content_type: &str, body: impl Into<Vec<u8>>) {
    let response = Response::from_data(body)
        .with_status_code(StatusCode(status))
        .with_header(Header::from_bytes("Content-Type", content_type).expect("valid header"))
        .with_header(Header::from_bytes("Cache-Control", "no-store").expect("valid header"))
        .with_header(Header::from_bytes("X-Content-Type-Options", "nosniff").expect("valid header"))
        .with_header(Header::from_bytes("Content-Security-Policy", CSP).expect("valid header"))
        .with_header(Header::from_bytes("Referrer-Policy", "no-referrer").expect("valid header"));
    if let Err(error) = request.respond(response) {
        eprintln!("Web response failed: {error}");
    }
}

fn error(request: Request, status: u16, message: &str) {
    reply(
        request,
        status,
        "application/json; charset=utf-8",
        json!({ "error": message }).to_string(),
    );
}

fn handle(
    mut request: Request,
    mind: &mut Kognitara,
    data: &Path,
    user: &MemberId,
    address: &str,
    token: &str,
) {
    if header(&request, "Host") != Some(address) {
        error(request, 403, "Invalid host.");
        return;
    }
    let origin = format!("http://{address}");
    if header(&request, "Origin").is_some_and(|value| value != origin) {
        error(request, 403, "Invalid origin.");
        return;
    }

    match (request.method(), request.url()) {
        (&Method::Get, "/") => reply(
            request,
            200,
            "text/html; charset=utf-8",
            include_str!("web_ui.html"),
        ),
        (&Method::Get, "/app.css") => reply(
            request,
            200,
            "text/css; charset=utf-8",
            include_str!("web_ui.css"),
        ),
        (&Method::Get, "/app.js") => reply(
            request,
            200,
            "text/javascript; charset=utf-8",
            include_str!("web_ui.js"),
        ),
        (&Method::Get, "/api/state" | "/api/timers") | (&Method::Post, "/api/chat") => {
            if header(&request, "Authorization") != Some(format!("Bearer {token}").as_str()) {
                error(request, 401, "Session token missing or invalid.");
                return;
            }
            if request.method() == &Method::Get {
                if request.url() == "/api/timers" {
                    match timers() {
                        Ok(timers) => reply(
                            request,
                            200,
                            "application/json; charset=utf-8",
                            timers.to_string(),
                        ),
                        Err(timer_error) => {
                            eprintln!("Cannot read timers: {timer_error}");
                            error(request, 503, &timer_error);
                        }
                    }
                    return;
                }
                let brains: Vec<_> = mind
                    .swarm
                    .brains
                    .iter()
                    .map(|(coord, brain)| {
                        json!({
                            "id": coord.0,
                            "phase": format!("{:?}", brain.phase),
                            "concepts": brain.tree.concepts.len(),
                            "tension": brain.state.cognitive_tension,
                        })
                    })
                    .collect();
                let state = json!({
                    "sanctuary": mind.sanctuary.sanctuary_name,
                    "epoch": mind.swarm.global_epoch,
                    "brains": brains,
                    "hypotheses": mind.hypotheses.hypotheses,
                    "reflections": reflection_log::recent(mind, current_timestamp()),
                    "questions": mind.pending_questions(),
                });
                reply(
                    request,
                    200,
                    "application/json; charset=utf-8",
                    state.to_string(),
                );
                return;
            }

            if header(&request, "Origin") != Some(origin.as_str())
                || !header(&request, "Content-Type")
                    .is_some_and(|value| value.eq_ignore_ascii_case("application/json"))
            {
                error(request, 403, "Invalid request origin or content type.");
                return;
            }
            if request.body_length().is_none_or(|length| length > MAX_BODY) {
                error(request, 413, "Request body is missing or too large.");
                return;
            }
            let mut body = Vec::new();
            if let Err(read_error) = request
                .as_reader()
                .take((MAX_BODY + 1) as u64)
                .read_to_end(&mut body)
            {
                eprintln!("Cannot read web request: {read_error}");
                error(request, 400, "Cannot read request body.");
                return;
            }
            let message: ChatMessage = match serde_json::from_slice(&body) {
                Ok(message) => message,
                Err(_) => {
                    error(request, 400, "Invalid message.");
                    return;
                }
            };
            let text = message.message.trim();
            if text.is_empty() || text.chars().count() > 4000 {
                error(request, 400, "Message must contain 1 to 4000 characters.");
                return;
            }
            let (response, _) = mind.converse(user, text);
            if let Err(save_error) = persist_import(mind, data) {
                eprintln!("Web conversation could not be fully saved: {save_error}");
                error(
                    request,
                    500,
                    &format!(
                        "La mémoire a changé. Vérifiez la sauvegarde avant de réessayer : {save_error}"
                    ),
                );
                return;
            }
            reply(
                request,
                200,
                "application/json; charset=utf-8",
                json!({ "response": response }).to_string(),
            );
        }
        _ => error(request, 404, "Not found."),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn timer_output_handles_scheduled_unscheduled_and_missing() {
        let output = br#"[{"unit":"klm-research.timer","next":1795718400000000},{"unit":"klm-backup.timer","next":0}]"#;
        let result = parse_timers(output).unwrap();
        assert_eq!(result["research"]["next_ms"], 1_795_718_400_000u64);
        assert_eq!(result["research"]["status"], "scheduled");
        assert_eq!(result["backup"]["status"], "unscheduled");
        let missing = parse_timers(br#"[]"#).unwrap();
        assert_eq!(missing["research"]["status"], "missing");
        assert!(parse_timers(b"not json").is_err());
    }
}

pub fn serve(mind: &mut Kognitara, data: &Path, user: &MemberId) -> Result<(), String> {
    let mut random = [0u8; 32];
    getrandom::fill(&mut random)
        .map_err(|error| format!("Cannot create session token: {error}"))?;
    let token: String = random.iter().map(|byte| format!("{byte:02x}")).collect();
    let server =
        Server::http("127.0.0.1:0").map_err(|error| format!("Cannot listen locally: {error}"))?;
    let address = server.server_addr().to_string();
    println!(
        "\nInterface locale : http://{address}/#{token}\nGardez ce lien privé. Arrêtez le serveur avec Ctrl+C."
    );
    loop {
        let request = server
            .recv()
            .map_err(|error| format!("Cannot receive HTTP request: {error}"))?;
        handle(request, mind, data, user, &address, &token);
    }
}
