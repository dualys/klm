use std::fs;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpStream;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
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
        Self::start_with_setup(|_| {})
    }

    fn start_with_setup(setup: impl FnOnce(&std::path::Path)) -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let dir = std::env::temp_dir().join(format!(
            "klm-web-test-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&dir).unwrap();
        setup(&dir);
        let bin = dir.join("bin");
        fs::create_dir(&bin).unwrap();
        fs::write(
            bin.join("systemctl"),
            "#!/bin/sh\nif [ \"$2\" = show ]; then\nprintf 'Id=klm-research.service\\nActiveState=active\\nSubState=running\\nResult=success\\n\\nId=klm-backup.service\\nActiveState=failed\\nSubState=failed\\nResult=exit-code\\n'\nelse\nprintf '%s\\n' '[{\"unit\":\"klm-research.timer\",\"next\":1795718400000000},{\"unit\":\"klm-backup.timer\",\"next\":1795731378234069}]'\nfi\n",
        )
        .unwrap();
        fs::set_permissions(bin.join("systemctl"), fs::Permissions::from_mode(0o700)).unwrap();
        fs::write(bin.join("journalctl"), "#!/bin/sh\nprintf '%s\\n' '{\"USER_UNIT\":\"klm-backup.service\",\"MESSAGE\":\"Backup failed\",\"PRIORITY\":\"3\",\"__REALTIME_TIMESTAMP\":\"1234000\"}'\n").unwrap();
        fs::set_permissions(bin.join("journalctl"), fs::Permissions::from_mode(0o700)).unwrap();
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
}

#[test]
fn dashboard_lists_real_associations_and_recorded_night() {
    let server = TestServer::start_with_setup(|dir| {
        let mut mind = klm::Kognitara::new("Test");
        let id = klm::ConceptId("fiche-cible".into());
        let brain = mind.swarm.brains.get_mut(&klm::NodeCoord(6)).unwrap();
        for i in 0..101 {
            let concept_id = if i == 0 {
                id.clone()
            } else {
                klm::ConceptId(format!("fiche-{i}"))
            };
            brain.tree.add_concept(klm::Concept::new(
                concept_id,
                format!("fiche_cible {i}"),
                "Description",
                klm::ConceptOrigin::HarvestedSeed {
                    source_domain: "example.org".into(),
                },
            ));
        }
        brain
            .learner
            .curiosity_engine
            .questions
            .push(klm::OpenQuestion {
                id: "q1".into(),
                related_concepts: vec![id.clone()],
                question_text: "Question liée ?".into(),
                urgency: 0.5,
                is_resolved: false,
                resolution_insight: None,
                created_at: 1,
            });
        let linked = mind.hypotheses.add("Liée", vec![id]).unwrap();
        mind.hypotheses
            .get_mut(linked)
            .unwrap()
            .add_evidence(
                klm::EvidenceRelation::Reports,
                "https://example.org/source",
                "note",
                1,
            )
            .unwrap();
        let unlinked = mind.hypotheses.add("Non liée", vec![]).unwrap();
        mind.hypotheses
            .get_mut(unlinked)
            .unwrap()
            .add_evidence(
                klm::EvidenceRelation::Reports,
                "https://example.org/unrelated",
                "note",
                1,
            )
            .unwrap();
        let mut snapshot = serde_json::to_vec(&klm::KpackRecipe::encode(
            &serde_json::to_vec(&mind).unwrap(),
        ))
        .unwrap();
        snapshot.push(b'\n');
        fs::write(dir.join("data"), snapshot).unwrap();
        let research = dir.join("research");
        fs::create_dir(&research).unwrap();
        let subjects: serde_json::Map<String, serde_json::Value> = (0..70).map(|i|
                (format!("subject-{i}"), serde_json::json!({"label":format!("Sujet {i}"), "kind":"Concept", "contradictions": if i == 0 { vec!["Lien signalé"] } else { vec![] }}))
            ).collect();
        fs::write(research.join("night-review.state"), serde_json::to_vec(&serde_json::json!({
                "concept_cursor":null, "hypothesis_cursor":null, "concept_examinations":70,
                "hypothesis_examinations":2, "last_run":100, "last_request":null,
                "concepts":[], "hypotheses":{},
                "summaries":{"2026-11-27":{"cycles":4,"concept_examinations":70,"hypothesis_examinations":2,"last_run":100,"subjects":subjects}}
            })).unwrap()).unwrap();
    });
    let auth = format!("Authorization: Bearer {}\r\n", server.token);
    for path in ["/api/knowledge", "/api/morning", "/api/activity"] {
        assert_eq!(server.request("GET", path, &server.address, "", "").0, 401);
    }
    let (status, body) = server.request(
        "GET",
        "/api/knowledge?q=FICHE_CIBLE",
        &server.address,
        &auth,
        "",
    );
    assert_eq!(status, 200);
    let data: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(data["total"], 101);
    assert_eq!(data["items"].as_array().unwrap().len(), 50);
    let (_, body) = server.request(
        "GET",
        "/api/knowledge?q=Question+li%C3%A9e",
        &server.address,
        &auth,
        "",
    );
    let data: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(data["total"], 1);
    let item = &data["items"][0];
    assert_eq!(item["source_domain"], "example.org");
    assert_eq!(item["questions"][0], "Question liée ?");
    assert_eq!(item["sources"].as_array().unwrap().len(), 1);
    assert_eq!(
        item["sources"][0]["evidence"]["source"],
        "https://example.org/source"
    );
    let (_, body) = server.request("GET", "/api/morning?page=2", &server.address, &auth, "");
    let data: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(data["summary"]["cycles"], 4);
    assert_eq!(data["summary"]["flagged"], 1);
    assert_eq!(data["items"].as_array().unwrap().len(), 20);
    let (_, body) = server.request(
        "GET",
        "/api/morning?q=lien+signal%C3%A9",
        &server.address,
        &auth,
        "",
    );
    let data: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(data["total"], 1);
    let (status, body) = server.request("GET", "/api/activity", &server.address, &auth, "");
    assert_eq!(status, 200);
    let data: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(data["services"][0]["active"], "active");
    assert_eq!(data["services"][1]["result"], "exit-code");
    assert_eq!(data["events"][0]["status"], "error");
    fs::write(
        server.dir.join("bin/journalctl"),
        "#!/bin/sh\necho 'journal indisponible' >&2\nexit 1\n",
    )
    .unwrap();
    let (status, body) = server.request("GET", "/api/activity", &server.address, &auth, "");
    assert_eq!(status, 503);
    assert!(body.contains("journal indisponible"));
    assert_eq!(
        server
            .request("GET", "/api/knowledge?q=x", &server.address, &auth, "")
            .0,
        200
    );
}

impl TestServer {
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
fn search_and_pagination_are_bounded_for_all_lists() {
    let server = TestServer::start_with_setup(|dir| {
        let mut mind = klm::Kognitara::new("Test");
        for i in 0..121 {
            mind.hypotheses
                .add(
                    &format!("{} {i}", if i < 101 { "Forêt" } else { "Océan" }),
                    vec![],
                )
                .unwrap();
        }
        let brain = mind.swarm.brains.get_mut(&klm::NodeCoord(6)).unwrap();
        for i in 0..101 {
            brain
                .learner
                .socratic_engine
                .reflections
                .push(klm::SocraticReflection {
                    id: format!("test-{i}"),
                    stage: klm::SocraticStage::Aporia,
                    interior_monologue: "Examen local".into(),
                    examined_hypothesis: "Une hypothèse".into(),
                    maieutic_question: format!("Pourquoi la forêt {i} ?"),
                    concepts: vec![],
                    humility_index: 0.5,
                    created_at: klm::current_timestamp(),
                });
        }
        let data = serde_json::to_vec(&mind).unwrap();
        let mut snapshot = serde_json::to_vec(&klm::KpackRecipe::encode(&data)).unwrap();
        snapshot.push(b'\n');
        fs::write(dir.join("data"), snapshot).unwrap();
        let research = dir.join("research");
        fs::create_dir(&research).unwrap();
        let reviews: serde_json::Map<String, serde_json::Value> = (0..70).map(|i| {
            (format!("{i}:0"), serde_json::json!({
                "id":i, "revision":0, "statement":format!("Arbre {i}"),
                "examinations":1, "examined_at":100, "reports":1, "supports":0,
                "contradicts":0, "contradictions":[], "questions":["Pourquoi ?"], "wikipedia":null
            }))
        }).collect();
        fs::write(research.join("night-review.state"), serde_json::to_vec(&serde_json::json!({
            "concept_cursor":null, "hypothesis_cursor":null, "concept_examinations":3,
            "hypothesis_examinations":70, "last_run":100, "last_request":null,
            "concepts":[{"brain":6,"label":"Rivière","contradictions":[],"question":"Pourquoi ?"}],
            "hypotheses":reviews
        })).unwrap()).unwrap();
    });
    let auth = format!("Authorization: Bearer {}\r\n", server.token);
    for path in ["/api/hypotheses", "/api/reflections", "/api/night"] {
        assert_eq!(server.request("GET", path, &server.address, "", "").0, 401);
        let (status, body) = server.request("GET", path, &server.address, &auth, "");
        assert_eq!(status, 200);
        let result: serde_json::Value = serde_json::from_str(&body).unwrap();
        assert_eq!(result["items"].as_array().unwrap().len(), 50, "{path}");
    }
    for (path, total, last) in [
        ("/api/hypotheses?q=FOR%C3%8AT&page=3", 101, 1),
        ("/api/reflections?q=for%C3%AAt&page=3", 101, 1),
        ("/api/night?q=ARBRE&page=2", 70, 20),
    ] {
        let (status, body) = server.request("GET", path, &server.address, &auth, "");
        assert_eq!(status, 200);
        let result: serde_json::Value = serde_json::from_str(&body).unwrap();
        assert_eq!(result["total"], total);
        assert_eq!(result["items"].as_array().unwrap().len(), last);
    }
    let (status, body) = server.request(
        "GET",
        "/api/night?q=Rivi%C3%A8re&page=99",
        &server.address,
        &auth,
        "",
    );
    let result: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(status, 200);
    assert_eq!(result["page"], 1);
    assert_eq!(result["total"], 1);
    for path in [
        "/api/hypotheses?page=0",
        "/api/reflections?q=%FF",
        "/api/night?page=1&page=2",
    ] {
        assert_eq!(
            server.request("GET", path, &server.address, &auth, "").0,
            400
        );
    }
    let (_, body) = server.request("GET", "/api/state", &server.address, &auth, "");
    let state: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert!(state.get("hypotheses").is_none());
    assert!(state.get("reflections").is_none());
    assert!(state["night_review"].get("hypotheses").is_none());
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
    assert!(state["night_review"].is_null());

    fs::create_dir_all(server.dir.join("research")).unwrap();
    let review = server.dir.join("research/night-review.state");
    fs::write(&review, r#"{"concept_cursor":null,"hypothesis_cursor":null,"concept_examinations":4,"hypothesis_examinations":2,"last_run":100,"last_request":null,"concepts":[],"hypotheses":{}}"#).unwrap();
    let (status, body) = server.request("GET", "/api/state", &server.address, &auth, "");
    assert_eq!(status, 200);
    let state: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(state["night_review"]["concept_examinations"], 4);
    fs::write(&review, "invalid").unwrap();
    assert_eq!(
        server
            .request("GET", "/api/state", &server.address, &auth, "")
            .0,
        503
    );
    fs::remove_file(review).unwrap();

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
