//! `bili-load` — open-loop load generator for the PRD-028 objective:
//! p95 under 100 ms and p99 under 250 ms at a sustained 100 requests per
//! second against a warmed instance, using the spec 09 payload mix of
//! 1, 2, 10 and 64 measurements.
//!
//! Usage:
//!   bili-load [--url http://127.0.0.1:8080] [--rps 100] [--duration 60]
//!             [--concurrency 64] [--warmup 5]
//!
//! Exit code 0 when the acceptance criteria hold, 2 otherwise. The target
//! service must run with test-environment rate limits (spec 09: controlled
//! bypass of per-IP throttling).

use std::sync::Arc;
use std::time::{Duration, Instant};

use bytes::Bytes;
use http_body_util::{BodyExt, Full};
use hyper_util::client::legacy::Client;
use hyper_util::rt::TokioExecutor;
use tokio::sync::{mpsc, Semaphore};

#[derive(Clone, Copy)]
struct Options {
    rps: u64,
    duration: Duration,
    warmup: Duration,
    concurrency: usize,
}

fn parse_args() -> (String, Options) {
    let mut url = "http://127.0.0.1:8080".to_string();
    let mut options = Options {
        rps: 100,
        duration: Duration::from_secs(60),
        warmup: Duration::from_secs(5),
        concurrency: 64,
    };
    let mut args = std::env::args().skip(1);
    while let Some(flag) = args.next() {
        let mut value = || args.next().expect("flag value");
        match flag.as_str() {
            "--url" => url = value(),
            "--rps" => options.rps = value().parse().expect("--rps"),
            "--duration" => {
                options.duration = Duration::from_secs(value().parse().expect("--duration"))
            }
            "--warmup" => options.warmup = Duration::from_secs(value().parse().expect("--warmup")),
            "--concurrency" => options.concurrency = value().parse().expect("--concurrency"),
            other => panic!("unknown flag {other}"),
        }
    }
    (url, options)
}

/// A realistic valid request with `count` serum measurements at distinct
/// ages (spec 09 load profile: 1, 2, 10 and 64 measurements).
fn payload(count: usize) -> Bytes {
    let measurements: Vec<serde_json::Value> = (0..count)
        .map(|i| {
            serde_json::json!({
                "id": format!("m{i}"),
                "age_minutes": 1000 + (i as u32) * 60,
                "total_bilirubin_umol_l": 120 + ((i * 7) % 100) as u32,
                "method": if i % 4 == 0 { "transcutaneous" } else { "serum" },
            })
        })
        .collect();
    let request = serde_json::json!({
        "rule_pack_id": "nice-cg98-2023-10-31.1",
        "gestational_age_completed_weeks": 38,
        "assessment_age_minutes": 5000,
        "clinical_features": {
            "suspected_or_obvious_jaundice": "present",
            "visible_jaundice": "present",
            "clinically_well": "present",
            "acute_bilirubin_encephalopathy": "absent",
            "pale_chalky_stools": "absent",
            "dark_urine_stains_nappy": "absent",
            "rhesus_haemolytic_disease": "absent",
            "abo_haemolytic_disease": "absent",
            "infection_suspected": "absent",
            "urinary_tract_infection_suspected": "absent",
            "routine_metabolic_screen_completed": "present"
        },
        "risk_factors": {
            "previous_sibling_required_phototherapy": "absent",
            "exclusive_breastfeeding_intended": "present"
        },
        "measurements": measurements,
        "treatment_state": { "mode": "none" }
    });
    Bytes::from(serde_json::to_vec(&request).expect("payload serialises"))
}

struct Sample {
    latency: Duration,
    status: u16,
}

#[tokio::main]
async fn main() {
    let (url, options) = parse_args();
    let endpoint: http::Uri = format!("{url}/v1/evaluations").parse().expect("valid url");
    let client: Client<_, Full<Bytes>> = Client::builder(TokioExecutor::new()).build_http();
    let payloads: Arc<Vec<Bytes>> =
        Arc::new(vec![payload(1), payload(2), payload(10), payload(64)]);

    // Warm the service and the connection pool before measuring.
    for i in 0..options.warmup.as_secs().max(1) * 20 {
        let request = http::Request::post(endpoint.clone())
            .header("content-type", "application/json")
            .body(Full::new(payloads[(i % 4) as usize].clone()))
            .unwrap();
        let _ = client
            .request(request)
            .await
            .map(|response| async { response.into_body().collect().await });
        tokio::time::sleep(Duration::from_millis(10)).await;
    }

    let (sender, mut receiver) = mpsc::unbounded_channel::<Sample>();
    let limiter = Arc::new(Semaphore::new(options.concurrency));
    let started = Instant::now();
    let total_requests = options.rps * options.duration.as_secs();
    let mut ticker = tokio::time::interval(Duration::from_nanos(1_000_000_000 / options.rps));
    ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Burst);

    for i in 0..total_requests {
        ticker.tick().await;
        let permit = limiter.clone().acquire_owned().await.expect("semaphore");
        let client = client.clone();
        let endpoint = endpoint.clone();
        let body = payloads[(i % 4) as usize].clone();
        let sender = sender.clone();
        tokio::spawn(async move {
            let _permit = permit;
            let request = http::Request::post(endpoint)
                .header("content-type", "application/json")
                .body(Full::new(body))
                .unwrap();
            let begin = Instant::now();
            match client.request(request).await {
                Ok(response) => {
                    let status = response.status().as_u16();
                    let _ = response.into_body().collect().await;
                    let _ = sender.send(Sample {
                        latency: begin.elapsed(),
                        status,
                    });
                }
                Err(_) => {
                    let _ = sender.send(Sample {
                        latency: begin.elapsed(),
                        status: 0,
                    });
                }
            }
        });
    }
    drop(sender);

    let mut latencies: Vec<Duration> = Vec::with_capacity(total_requests as usize);
    let mut failures = 0u64;
    while let Some(sample) = receiver.recv().await {
        if sample.status == 200 {
            latencies.push(sample.latency);
        } else {
            failures += 1;
        }
    }
    let elapsed = started.elapsed();
    latencies.sort_unstable();

    let percentile = |p: f64| -> Duration {
        if latencies.is_empty() {
            return Duration::ZERO;
        }
        let index = ((latencies.len() as f64 - 1.0) * p).round() as usize;
        latencies[index]
    };
    let completed = latencies.len() as u64 + failures;
    let error_rate = if completed > 0 {
        failures as f64 / completed as f64
    } else {
        1.0
    };
    let p50 = percentile(0.50);
    let p95 = percentile(0.95);
    let p99 = percentile(0.99);
    let max = latencies.last().copied().unwrap_or_default();

    println!("bili-load report");
    println!("  target             {endpoint}");
    println!(
        "  offered load       {} rps for {:?} (payload mix 1/2/10/64 measurements)",
        options.rps, options.duration
    );
    println!(
        "  completed          {completed} requests in {elapsed:?} ({:.1} rps achieved)",
        completed as f64 / elapsed.as_secs_f64()
    );
    println!(
        "  non-200/transport  {failures} ({:.3}%)",
        error_rate * 100.0
    );
    println!("  p50                {p50:?}");
    println!("  p95                {p95:?}  (objective < 100 ms)");
    println!("  p99                {p99:?}  (objective < 250 ms)");
    println!("  max                {max:?}");

    let pass =
        p95 < Duration::from_millis(100) && p99 < Duration::from_millis(250) && error_rate < 0.001;
    println!(
        "  verdict            {}",
        if pass { "PASS" } else { "FAIL" }
    );
    std::process::exit(if pass { 0 } else { 2 });
}
