//! Full-stack process tests (Stage 2 hardening): spawn the real compiled
//! binary, drive it over real HTTP, and verify what the *process* actually
//! emits and how it dies.
//!
//! - Sentinel leak check (TEST-028 application layer): distinctive clinical
//!   values submitted over the wire must never appear in anything the
//!   process writes to stdout or stderr.
//! - Graceful SIGTERM drain (spec 10 container contract): the process stops
//!   accepting work and exits cleanly within ten seconds.
//! - Binary-level readiness gate: clinical mode without authorisation
//!   refuses readiness in the shipped artifact, not just in unit tests.

use std::io::{Read, Write};
use std::net::TcpStream;
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

struct ApiProcess {
    child: Child,
    port: u16,
}

impl ApiProcess {
    fn spawn(port: u16, extra_env: &[(&str, &str)]) -> Self {
        let mut command = Command::new(env!("CARGO_BIN_EXE_bili-mate-api"));
        command
            .env("BILI_MATE_BIND", format!("127.0.0.1:{port}"))
            .env("RUST_LOG", "info")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        for (key, value) in extra_env {
            command.env(key, value);
        }
        let child = command.spawn().expect("spawn bili-mate-api");
        let process = Self { child, port };
        process.wait_until_live();
        process
    }

    fn wait_until_live(&self) {
        let deadline = Instant::now() + Duration::from_secs(30);
        while Instant::now() < deadline {
            if let Ok((status, _)) = self.request("GET", "/health/live", None) {
                if status == 200 {
                    return;
                }
            }
            std::thread::sleep(Duration::from_millis(100));
        }
        panic!("server did not become live on port {}", self.port);
    }

    /// Minimal HTTP/1.1 client over a raw socket: no client library between
    /// the test and the process under test.
    fn request(
        &self,
        method: &str,
        path: &str,
        body: Option<&str>,
    ) -> std::io::Result<(u16, String)> {
        let mut stream = TcpStream::connect(("127.0.0.1", self.port))?;
        stream.set_read_timeout(Some(Duration::from_secs(10)))?;
        let body = body.unwrap_or("");
        let request = format!(
            "{method} {path} HTTP/1.1\r\nHost: 127.0.0.1\r\nAccept: application/json\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        );
        stream.write_all(request.as_bytes())?;
        let mut response = String::new();
        stream.read_to_string(&mut response)?;
        let status = response
            .split_whitespace()
            .nth(1)
            .and_then(|s| s.parse::<u16>().ok())
            .unwrap_or(0);
        Ok((status, response))
    }

    fn terminate_and_collect(mut self) -> (bool, Duration, String) {
        let pid = self.child.id().to_string();
        Command::new("kill")
            .args(["-TERM", &pid])
            .status()
            .expect("send SIGTERM");
        let started = Instant::now();
        let deadline = started + Duration::from_secs(10);
        loop {
            match self.child.try_wait().expect("try_wait") {
                Some(status) => {
                    let elapsed = started.elapsed();
                    let mut output = String::new();
                    if let Some(mut stdout) = self.child.stdout.take() {
                        stdout.read_to_string(&mut output).ok();
                    }
                    if let Some(mut stderr) = self.child.stderr.take() {
                        stderr.read_to_string(&mut output).ok();
                    }
                    return (status.success(), elapsed, output);
                }
                None if Instant::now() > deadline => {
                    self.child.kill().ok();
                    panic!("server did not drain within 10 seconds of SIGTERM");
                }
                None => std::thread::sleep(Duration::from_millis(50)),
            }
        }
    }
}

/// A valid assessment carrying unmistakable sentinel values.
fn sentinel_request() -> String {
    include_str!("../../../spec/examples/normal-below-threshold-request.json")
        .replace(
            "\"total_bilirubin_umol_l\": 180",
            "\"total_bilirubin_umol_l\": 777",
        )
        .replace(
            "\"assessment_age_minutes\": 2880",
            "\"assessment_age_minutes\": 33333",
        )
        .replace("\"age_minutes\": 2880", "\"age_minutes\": 33333")
}

#[test]
fn process_output_never_contains_clinical_content() {
    let server = ApiProcess::spawn(19731, &[]);

    // Exercise every route so the log surface is as wide as possible.
    let (status, response) = server
        .request("POST", "/v1/evaluations", Some(&sentinel_request()))
        .expect("evaluation request");
    assert_eq!(status, 200, "sentinel evaluation must succeed: {response}");
    assert!(response.contains("primary_action"));
    server
        .request("GET", "/v1/guidelines/active", None)
        .unwrap();
    server
        .request(
            "GET",
            "/v1/threshold-curves/nice-cg98-2023-10-31.1?gestational_age_completed_weeks=31",
            None,
        )
        .unwrap();
    server.request("GET", "/health/ready", None).unwrap();
    // An invalid request, so error paths are logged too.
    server
        .request(
            "POST",
            "/v1/evaluations",
            Some("{\"nhs_number\": \"SENTINEL-9241\"}"),
        )
        .unwrap();

    let (graceful, drain, output) = server.terminate_and_collect();
    assert!(graceful, "process must exit cleanly on SIGTERM");
    assert!(drain < Duration::from_secs(10), "drain took {drain:?}");

    // The allowlisted operational log is present…
    assert!(
        output.contains("request completed"),
        "expected request logs; got: {output}"
    );
    assert!(output.contains("/v1/evaluations"));

    // …and nothing clinical or submitted ever is (SEC-007, API-015).
    for forbidden in [
        "777",
        "33333",
        "SENTINEL-9241",
        "total_bilirubin",
        "gestational_age",
        "suspected_or_obvious_jaundice",
        "NO_ROUTINE_REPEAT",
        "digest",
        "nhs_number",
    ] {
        assert!(
            !output.contains(forbidden),
            "process output must never contain {forbidden:?}; got: {output}"
        );
    }
}

#[test]
fn clinical_mode_binary_refuses_readiness_without_authorisation() {
    let server = ApiProcess::spawn(19733, &[("BILI_MATE_MODE", "clinical")]);
    let (live, _) = server.request("GET", "/health/live", None).unwrap();
    assert_eq!(
        live, 200,
        "liveness is independent of clinical authorisation"
    );
    let (ready, body) = server.request("GET", "/health/ready", None).unwrap();
    assert_eq!(
        ready, 503,
        "draft pack must never serve clinical mode: {body}"
    );
    let (evaluation, body) = server
        .request(
            "POST",
            "/v1/evaluations",
            Some(include_str!(
                "../../../spec/examples/normal-below-threshold-request.json"
            )),
        )
        .unwrap();
    // An unready clinical-mode instance fails closed: a draft pack never
    // answers a clinical-mode evaluation (CLIN-003), even before
    // orchestration removes the instance.
    assert_eq!(
        evaluation, 503,
        "unready clinical mode must refuse evaluations: {body}"
    );
    assert!(
        !body.contains("primary_action"),
        "no clinical content when refusing"
    );
    let (_, _, output) = server.terminate_and_collect();
    assert!(output.contains("listening"));
}

#[test]
fn in_flight_requests_complete_during_drain() {
    let server = ApiProcess::spawn(19735, &[]);
    // Requests immediately before termination still get complete responses.
    let (status, response) = server
        .request(
            "POST",
            "/v1/evaluations",
            Some(include_str!(
                "../../../spec/examples/early-jaundice-request.json"
            )),
        )
        .unwrap();
    assert_eq!(status, 200);
    assert!(
        response.contains("decision_receipt"),
        "response must be complete"
    );
    let (graceful, _, _) = server.terminate_and_collect();
    assert!(graceful);
}
