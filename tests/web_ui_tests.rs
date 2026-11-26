use std::fs;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpStream;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::mpsc;
use std::time::Duration;

struct TestServer {
    child: Child,
    dir: PathBuf,
    address: String,
    token: String,
}

impl Drop for TestServer {
    fn drop(&mut self) {
        self.child.kill().expect("stop UI");
        self.child.wait().expect("reap UI");
        fs::remove_dir_all(&self.dir).expect("remove test directory");
    }
}

impl TestServer {
    fn start() -> Self {
        let dir = std::env::temp_dir().join(format!("klm-web-test-{}", std::process::id()));
        fs::create_dir(&dir).unwrap();
        let bin = dir.join("bin");
        fs::create_dir(&bin).unwrap();
        fs::write(
            bin.join("systemctl"),
            "#!/bin/sh\nprintf '%s\\n' '[{\"unit\":\"klm-research.timer\",\"next\":1795718400000000},{\"unit\":\"klm-backup.timer\",\"next\":1795731378234069}]'\n",
        )
        .unwrap();
        fs::set_permissions(bin.join("systemctl"), fs::Permissions::from_mode(0o700)).unwrap();
        let mut child = Command::new(env!("CARGO_BIN_EXE_klm"))
            .arg("ui")
            .env("KOGNITARA_PATH", &dir)
            .env("USER", "test-user")
            .env("PATH", &bin)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        let stdout = child.stdout.take().unwrap();
        let (sender, receiver) = mpsc::channel();
        std::thread::spawn(move || {
            for line in BufReader::new(stdout).lines() {
                match line {
                    Ok(line) if line.starts_with("Interface locale : ") => {
                        let _ = sender.send(line);
                        return;
                    }
                    Ok(_) => {}
                    Err(error) => panic!("Cannot read UI startup: {error}"),
                }
            }
        });
        let line = receiver.recv_timeout(Duration::from_secs(30)).unwrap();
        let url = line.strip_prefix("Interface locale : http://").unwrap();
        let (address, token) = url.split_once("/#").unwrap();
        Self {
            child,
            dir,
            address: address.to_string(),
            token: token.to_string(),
        }
    }

    fn request(
        &self,
        method: &str,
        path: &str,
        host: &str,
        headers: &str,
        body: &str,
    ) -> (u16, String) {
        let mut stream = TcpStream::connect(&self.address).unwrap();
        stream
            .set_read_timeout(Some(Duration::from_secs(10)))
            .unwrap();
        write!(stream, "{method} {path} HTTP/1.1\r\nHost: {host}\r\nConnection: close\r\nContent-Length: {}\r\n{headers}\r\n{body}", body.len()).unwrap();
        let mut response = String::new();
        stream.read_to_string(&mut response).unwrap();
        let (head, body) = response.split_once("\r\n\r\n").unwrap();
        let status = head.split_whitespace().nth(1).unwrap().parse().unwrap();
        (status, body.to_owned())
    }
}

#[test]
fn local_ui_authenticates_and_persists_conversation() {
    let server = TestServer::start();
    assert!(server.address.starts_with("127.0.0.1:"));
    let (status, page) = server.request("GET", "/", &server.address, "", "");
    assert_eq!(status, 200);
    assert!(page.contains("Conversation"));
    assert_eq!(
        server
            .request("GET", "/api/state", &server.address, "", "")
            .0,
        401
    );
    assert_eq!(
        server
            .request("GET", "/api/timers", &server.address, "", "")
            .0,
        401
    );
    let auth = format!("Authorization: Bearer {}\r\n", server.token);
    let (status, body) = server.request("GET", "/api/timers", &server.address, &auth, "");
    assert_eq!(status, 200);
    let timers: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(timers["research"]["next_ms"], 1_795_718_400_000u64);
    assert_eq!(timers["backup"]["next_ms"], 1_795_731_378_234u64);
    fs::write(
        server.dir.join("bin/systemctl"),
        "#!/bin/sh\necho unavailable >&2\nexit 1\n",
    )
    .unwrap();
    let (status, body) = server.request("GET", "/api/timers", &server.address, &auth, "");
    assert_eq!(status, 503);
    assert!(body.contains("unavailable"));
    assert_eq!(
        server
            .request("GET", "/api/state", "example.com", &auth, "")
            .0,
        403
    );
    let (status, body) = server.request("GET", "/api/state", &server.address, &auth, "");
    assert_eq!(status, 200);
    let state: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(state["brains"].as_array().unwrap().len(), 8);

    let payload = r#"{"message":"Bonjour depuis le navigateur"}"#;
    let post = format!("{auth}Content-Type: application/json\r\n");
    assert_eq!(
        server
            .request("POST", "/api/chat", &server.address, &post, payload)
            .0,
        403
    );
    let wrong_origin = format!("{post}Origin: https://example.com\r\n");
    assert_eq!(
        server
            .request("POST", "/api/chat", &server.address, &wrong_origin, payload)
            .0,
        403
    );
    let headers = format!("{post}Origin: http://{}\r\n", server.address);
    assert_eq!(
        server
            .request(
                "POST",
                "/api/chat",
                &server.address,
                &headers,
                &"x".repeat(17_000),
            )
            .0,
        413
    );
    let (status, body) = server.request("POST", "/api/chat", &server.address, &headers, payload);
    assert_eq!(status, 200);
    assert!(serde_json::from_str::<serde_json::Value>(&body).unwrap()["response"].is_string());
    let lines = fs::read_to_string(server.dir.join("data")).unwrap();
    let recipe: klm::KpackRecipe = serde_json::from_str(lines.lines().last().unwrap()).unwrap();
    let mind: klm::Kognitara = serde_json::from_slice(&recipe.decode().unwrap()).unwrap();
    assert!(mind.swarm.brains.values().any(|brain| {
        brain
            .learner
            .episodic_stream
            .iter()
            .any(|episode| episode.stimulus == "Bonjour depuis le navigateur")
    }));

    fs::rename(server.dir.join("data"), server.dir.join("data-test-backup")).unwrap();
    let (status, error) = server.request("POST", "/api/chat", &server.address, &headers, payload);
    assert_eq!(status, 500);
    assert!(error.contains("mémoire a changé"));
    fs::rename(server.dir.join("data-test-backup"), server.dir.join("data")).unwrap();

    let locked = Command::new(env!("CARGO_BIN_EXE_klm"))
        .arg("ui")
        .env("KOGNITARA_PATH", &server.dir)
        .env("USER", "test-user")
        .output()
        .unwrap();
    assert!(!locked.status.success());
    assert!(String::from_utf8_lossy(&locked.stderr).contains("already in use"));
}
