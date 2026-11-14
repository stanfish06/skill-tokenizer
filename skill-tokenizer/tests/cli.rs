use serde_json::{Value, json};
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::process::{Command, Output, Stdio};
use std::thread;
use std::time::{Duration, Instant};
use tempfile::TempDir;

fn cli() -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_skill-tokenizer"));
    command.env_remove("OPENAI_API_KEY");
    command.env_remove("OPENAI_BASE_URL");
    command.env_remove("OPENAI_MODEL");
    command.env("NO_PROXY", "127.0.0.1,localhost");
    command
}

fn read_request(stream: &mut TcpStream) -> (String, Value) {
    stream
        .set_read_timeout(Some(Duration::from_secs(10)))
        .unwrap();
    let mut bytes = Vec::new();
    let header_end = loop {
        let mut buffer = [0; 4096];
        let count = stream.read(&mut buffer).unwrap();
        assert!(count > 0, "connection closed before request headers");
        bytes.extend_from_slice(&buffer[..count]);
        if let Some(index) = bytes.windows(4).position(|part| part == b"\r\n\r\n") {
            break index + 4;
        }
    };
    let headers = String::from_utf8(bytes[..header_end].to_vec()).unwrap();
    let length: usize = headers
        .lines()
        .find_map(|line| {
            let (name, value) = line.split_once(':')?;
            name.eq_ignore_ascii_case("content-length")
                .then(|| value.trim().parse().unwrap())
        })
        .unwrap();
    while bytes.len() - header_end < length {
        let mut buffer = [0; 4096];
        let count = stream.read(&mut buffer).unwrap();
        assert!(count > 0, "connection closed before request body");
        bytes.extend_from_slice(&buffer[..count]);
    }
    let body = serde_json::from_slice(&bytes[header_end..header_end + length]).unwrap();
    (headers, body)
}

fn server(status: u16, body: Value) -> (String, thread::JoinHandle<(String, Value)>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    listener.set_nonblocking(true).unwrap();
    let address = format!("http://{}/v1", listener.local_addr().unwrap());
    let handle = thread::spawn(move || {
        let deadline = Instant::now() + Duration::from_secs(10);
        let mut stream = loop {
            match listener.accept() {
                Ok((stream, _)) => break stream,
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                    assert!(Instant::now() < deadline, "CLI never contacted endpoint");
                    thread::sleep(Duration::from_millis(10));
                }
                Err(error) => panic!("{error}"),
            }
        };
        let request = read_request(&mut stream);
        let body = body.to_string();
        write!(
            stream,
            "HTTP/1.1 {status} Test\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        )
        .unwrap();
        request
    });
    (address, handle)
}

fn completion(atoms: Value) -> Value {
    json!({
        "model": "test-model-snapshot",
        "choices": [{"finish_reason": "stop", "message": {
            "role": "assistant", "content": json!({"atoms": atoms}).to_string()
        }}],
        "usage": {"prompt_tokens": 100, "completion_tokens": 50, "total_tokens": 150}
    })
}

fn atom(statement: &str, quote: &str, line: usize) -> Value {
    json!({"statement": statement, "kind": "rule", "evidence": [{
        "quote": quote, "start_line": line, "end_line": line
    }]})
}

fn assert_success(output: &Output) {
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn atomize_file_returns_source_grounded_json() {
    let directory = TempDir::new().unwrap();
    let input = directory.path().join("SKILL.md");
    let source = "# Conversion\n\nIf validation fails, do not write output.\n";
    std::fs::write(&input, source).unwrap();
    let response = completion(json!([atom(
        "If validation fails, do not write output.",
        "If validation fails, do not write output.",
        3,
    )]));
    let (endpoint, server) = server(200, response);
    let output = cli()
        .args([
            "atomize",
            input.to_str().unwrap(),
            "--model",
            "test-model",
            "--base-url",
            &endpoint,
        ])
        .output()
        .unwrap();
    assert_success(&output);
    let document: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(document["schema_version"], 1);
    assert_eq!(document["model"]["requested"], "test-model");
    assert_eq!(document["model"]["returned"], "test-model-snapshot");
    assert_eq!(
        document["atoms"][0]["statement"],
        "If validation fails, do not write output."
    );
    let evidence = &document["atoms"][0]["evidence"][0];
    let start = evidence["start_byte"].as_u64().unwrap() as usize;
    let end = evidence["end_byte"].as_u64().unwrap() as usize;
    assert_eq!(&source[start..end], evidence["quote"].as_str().unwrap());
    let (headers, request) = server.join().unwrap();
    assert!(headers.starts_with("POST /v1/chat/completions HTTP/1.1"));
    assert!(!headers.to_lowercase().contains("authorization:"));
    assert_eq!(request["model"], "test-model");
    assert_eq!(request["response_format"]["type"], "json_object");
    assert!(
        request["messages"][1]["content"]
            .as_str()
            .unwrap()
            .contains("3 | If validation fails")
    );
}

#[test]
fn train_and_encode_keep_the_existing_interface() {
    let directory = TempDir::new().unwrap();
    std::fs::write(
        directory.path().join("SKILL.md"),
        "# Validation\n\nvalidation fails\n",
    )
    .unwrap();
    let tokenizer = directory.path().join("tokenizer.json");
    let trained = cli()
        .args([
            "train",
            directory.path().to_str().unwrap(),
            "--output",
            tokenizer.to_str().unwrap(),
            "--vocab-size",
            "100",
        ])
        .output()
        .unwrap();
    assert_success(&trained);
    let mut encoded = cli()
        .args(["encode", tokenizer.to_str().unwrap()])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    encoded
        .stdin
        .take()
        .unwrap()
        .write_all(b"validation fails")
        .unwrap();
    let encoded = encoded.wait_with_output().unwrap();
    assert_success(&encoded);
    let encoded = String::from_utf8(encoded.stdout).unwrap();
    let lines: Vec<_> = encoded.lines().collect();
    assert_eq!(lines.len(), 2);
    assert!(lines[0].starts_with("[CLS] "));
    assert!(lines[0].ends_with(" [SEP]"));
    assert!(serde_json::from_str::<Vec<u32>>(lines[1]).unwrap().len() > 2);
}

#[test]
fn stdin_preserves_unicode_crlf_and_multiline_code_and_merges_duplicates() {
    let source = "# Café\r\n\r\nIf checking fails:\r\n```sh\r\nexit 1\r\n```\r\n";
    let citation = json!({"quote": "If checking fails:\r\n```sh\r\nexit 1\r\n```", "start_line": 3, "end_line": 6});
    let candidate = json!({"statement":"If checking fails, use `exit 1` to stop.", "kind":"rule", "evidence":[citation]});
    let (endpoint, server) = server(200, completion(json!([candidate.clone(), candidate])));
    let mut child = cli()
        .args([
            "atomize",
            "-",
            "--model",
            "test-model",
            "--base-url",
            &endpoint,
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(source.as_bytes())
        .unwrap();
    let output = child.wait_with_output().unwrap();
    assert_success(&output);
    let document: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(document["source"]["path"], "-");
    assert_eq!(document["source"]["bytes"], source.len());
    assert_eq!(document["atoms"].as_array().unwrap().len(), 1);
    assert_eq!(
        document["atoms"][0]["evidence"].as_array().unwrap().len(),
        1
    );
    let evidence = &document["atoms"][0]["evidence"][0];
    assert_eq!(evidence["start_byte"], 11);
    let end = evidence["end_byte"].as_u64().unwrap() as usize;
    assert_eq!(&source[11..end], evidence["quote"].as_str().unwrap());
    let (_, request) = server.join().unwrap();
    assert_eq!(request["max_completion_tokens"], 16_384);
}

#[test]
fn api_key_file_authenticates_without_leaking_and_success_replaces_output() {
    let directory = TempDir::new().unwrap();
    let input = directory.path().join("SKILL.md");
    let output_path = directory.path().join("atoms.json");
    let key_path = directory.path().join("credential.txt");
    std::fs::write(&input, "Always validate first.\n").unwrap();
    std::fs::write(&output_path, "previous result").unwrap();
    std::fs::write(&key_path, "fake-test-key\n").unwrap();
    let (endpoint, server) = server(
        200,
        completion(json!([atom(
            "Always validate first.",
            "Always validate first.",
            1
        )])),
    );
    let output = cli()
        .args([
            "atomize",
            input.to_str().unwrap(),
            "--model",
            "test-model",
            "--base-url",
            &endpoint,
            "--api-key-file",
            key_path.to_str().unwrap(),
            "--output",
            output_path.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert_success(&output);
    assert!(output.stdout.is_empty());
    assert!(!String::from_utf8_lossy(&output.stderr).contains("fake-test-key"));
    let written = std::fs::read_to_string(&output_path).unwrap();
    assert!(!written.contains("fake-test-key"));
    let document: Value = serde_json::from_str(&written).unwrap();
    assert_eq!(document["atoms"][0]["id"], "atom-0001");
    let (headers, request) = server.join().unwrap();
    assert!(
        headers
            .to_lowercase()
            .contains("authorization: bearer fake-test-key")
    );
    assert!(!request.to_string().contains("fake-test-key"));
}

fn assert_failed_extraction_preserves_output(
    source: &str,
    status: u16,
    response: Value,
    expected_error: &str,
) {
    let directory = TempDir::new().unwrap();
    let input = directory.path().join("SKILL.md");
    let output_path = directory.path().join("atoms.json");
    std::fs::write(&input, source).unwrap();
    std::fs::write(&output_path, "previous result").unwrap();
    let (endpoint, server) = server(status, response);
    let output = cli()
        .args([
            "atomize",
            input.to_str().unwrap(),
            "--model",
            "test-model",
            "--base-url",
            &endpoint,
            "--output",
            output_path.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    let error = String::from_utf8_lossy(&output.stderr);
    assert!(
        error.contains(expected_error),
        "expected {expected_error:?}, got {error}"
    );
    assert!(!error.contains("secret-echo"));
    assert_eq!(
        std::fs::read_to_string(output_path).unwrap(),
        "previous result"
    );
    server.join().unwrap();
}

#[test]
fn invalid_evidence_and_schema_do_not_publish_results() {
    let source = "---\nname: sample\n---\n# Rules\n\nValidate first.\n";
    let cases = [
        (
            json!([atom("Use a secret command.", "invented quotation", 6)]),
            "does not match",
        ),
        (
            json!([atom("Validate first.", "Validate first.", 99)]),
            "invalid evidence line range",
        ),
        (
            json!([atom("The skill is named sample.", "name: sample", 2)]),
            "not YAML frontmatter",
        ),
        (
            json!([{"statement":"Validate first.", "kind":"rule", "evidence":[]}]),
            "needs a statement and evidence",
        ),
        (
            json!([{"statement":"Validate first.", "kind":"unknown", "evidence":[]}]),
            "invalid atom JSON",
        ),
        (
            json!([{"statement":"", "kind":"rule", "evidence":[]}]),
            "needs a statement and evidence",
        ),
        (
            json!([{"statement":"Validate first.", "kind":"rule", "evidence":[{"quote":"Validate first.","start_line":4,"end_line":6}]}]),
            "must match the quotation",
        ),
    ];
    for (candidates, error) in cases {
        assert_failed_extraction_preserves_output(source, 200, completion(candidates), error);
    }
    assert_failed_extraction_preserves_output(
        "Repeat Repeat\n",
        200,
        completion(json!([atom("Repeat.", "Repeat", 1)])),
        "ambiguous",
    );
}

#[test]
fn malformed_truncated_refused_and_failed_completions_are_errors() {
    let mut malformed = completion(json!([]));
    malformed["choices"][0]["message"]["content"] = json!("{broken secret-echo}");
    let mut truncated = completion(json!([]));
    truncated["choices"][0]["finish_reason"] = json!("length");
    let mut refused = completion(json!([]));
    refused["choices"][0]["message"]["refusal"] = json!("secret-echo");
    let cases = [
        (200, malformed, "invalid atom JSON"),
        (200, truncated, "truncated"),
        (200, refused, "refused"),
        (200, json!({"secret-echo": true}), "invalid chat completion"),
        (429, json!({"error":"secret-echo"}), "HTTP 429"),
        (500, json!({"error":"secret-echo"}), "HTTP 500"),
    ];
    for (status, response, error) in cases {
        assert_failed_extraction_preserves_output("Validate first.\n", status, response, error);
    }
}

#[test]
fn invalid_inputs_fail_before_contacting_the_endpoint() {
    let directory = TempDir::new().unwrap();
    let input = directory.path().join("SKILL.md");
    let cases = [
        ("", vec![], "empty"),
        ("some text", vec!["--max-input-bytes", "3"], "exceeds"),
        ("---\nname: broken\n", vec![], "unterminated"),
    ];
    for (source, flags, expected_error) in cases {
        std::fs::write(&input, source).unwrap();
        let output = cli()
            .args([
                "atomize",
                input.to_str().unwrap(),
                "--model",
                "test-model",
                "--base-url",
                "http://127.0.0.1:1/v1",
            ])
            .args(flags)
            .output()
            .unwrap();
        assert!(!output.status.success());
        assert!(String::from_utf8_lossy(&output.stderr).contains(expected_error));
    }
    std::fs::write(&input, "Validate first.\n").unwrap();
    let output = cli()
        .args([
            "atomize",
            input.to_str().unwrap(),
            "--model",
            "test-model",
            "--output",
            input.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("must not overwrite"));
    assert_eq!(
        std::fs::read_to_string(&input).unwrap(),
        "Validate first.\n"
    );
}
