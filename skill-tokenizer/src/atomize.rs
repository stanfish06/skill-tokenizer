use clap::Args;
use reqwest::Url;
use reqwest::blocking::Client;
use reqwest::header::{AUTHORIZATION, HeaderValue};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};
use tempfile::NamedTempFile;

const PROMPT: &str = include_str!("../prompts/atomize-v1.txt");
const PROMPT_VERSION: &str = "atomize-v1";
const MAX_RESPONSE_BYTES: u64 = 8 * 1024 * 1024;

#[derive(Args)]
pub struct AtomizeArgs {
    /// Markdown file, or - to read stdin
    pub path: PathBuf,
    /// Write JSON atomically to a file instead of stdout
    #[arg(short, long)]
    pub output: Option<PathBuf>,
    /// Model served by the endpoint (required; there is no paid default)
    #[arg(long, env = "OPENAI_MODEL")]
    pub model: String,
    /// API base URL, including /v1 when required by the server
    #[arg(
        long,
        env = "OPENAI_BASE_URL",
        default_value = "https://api.openai.com/v1"
    )]
    pub base_url: String,
    /// Read a raw API key from this file instead of OPENAI_API_KEY
    #[arg(long)]
    pub api_key_file: Option<PathBuf>,
    /// Maximum original input bytes; oversized documents fail before a request
    #[arg(long, default_value_t = 131_072, value_parser = clap::value_parser!(u64).range(1..=4_194_304))]
    pub max_input_bytes: u64,
    /// Completion token budget, including reasoning tokens on reasoning models
    #[arg(long, default_value_t = 16_384, value_parser = clap::value_parser!(u32).range(1..))]
    pub max_output_tokens: u32,
    /// Total request timeout; requests are never automatically retried
    #[arg(long, default_value_t = 120, value_parser = clap::value_parser!(u64).range(1..=3600))]
    pub timeout_secs: u64,
}

#[derive(Clone, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
enum AtomKind {
    Fact,
    Rule,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Candidates {
    atoms: Vec<Candidate>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Candidate {
    statement: String,
    kind: AtomKind,
    evidence: Vec<Citation>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Citation {
    quote: String,
    start_line: usize,
    end_line: usize,
}

#[derive(Eq, PartialEq, Serialize)]
struct Evidence {
    quote: String,
    start_line: usize,
    end_line: usize,
    start_byte: usize,
    end_byte: usize,
}

#[derive(Serialize)]
struct Atom {
    id: String,
    statement: String,
    kind: AtomKind,
    evidence: Vec<Evidence>,
}

#[derive(Serialize)]
struct Document {
    schema_version: u32,
    source: Source,
    prompt: Prompt,
    model: Model,
    usage: Option<Usage>,
    atoms: Vec<Atom>,
}

#[derive(Serialize)]
struct Source {
    path: String,
    sha256: String,
    bytes: usize,
}

#[derive(Serialize)]
struct Prompt {
    version: &'static str,
    sha256: String,
}

#[derive(Serialize)]
struct Model {
    requested: String,
    returned: Option<String>,
}

#[derive(Deserialize, Serialize)]
struct Usage {
    prompt_tokens: u64,
    completion_tokens: u64,
    total_tokens: u64,
}

#[derive(Deserialize)]
struct Completion {
    model: Option<String>,
    choices: Vec<Choice>,
    usage: Option<Usage>,
}

#[derive(Deserialize)]
struct Choice {
    finish_reason: Option<String>,
    message: Message,
}

#[derive(Deserialize)]
struct Message {
    content: Option<String>,
    refusal: Option<String>,
}

fn invalid(message: impl Into<String>) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message.into())
}

fn read_bounded(reader: impl Read, limit: u64, label: &str) -> io::Result<String> {
    let mut bytes = Vec::new();
    reader.take(limit + 1).read_to_end(&mut bytes)?;
    if bytes.len() as u64 > limit {
        return Err(invalid(format!("{label} exceeds the {limit}-byte limit")));
    }
    String::from_utf8(bytes).map_err(|_| invalid(format!("{label} must be UTF-8")))
}

fn endpoint(base_url: &str) -> io::Result<Url> {
    let url = Url::parse(&format!(
        "{}/chat/completions",
        base_url.trim_end_matches('/')
    ))
    .map_err(|_| invalid("invalid API base URL"))?;
    if !matches!(url.scheme(), "http" | "https")
        || url.host_str().is_none()
        || !url.username().is_empty()
        || url.password().is_some()
        || url.query().is_some()
        || url.fragment().is_some()
    {
        return Err(invalid(
            "API base URL must be HTTP(S) without credentials, query, or fragment",
        ));
    }
    Ok(url)
}

fn request(args: &AtomizeArgs, url: Url, source: &str) -> tokenizers::Result<Completion> {
    let key = if let Some(path) = &args.api_key_file {
        Some(read_bounded(File::open(path)?, 16_384, "API key file")?)
    } else {
        std::env::var("OPENAI_API_KEY").ok()
    };
    let key = key.as_deref().map(str::trim).filter(|key| !key.is_empty());
    let is_loopback = url.host_str().is_some_and(|host| {
        host == "localhost"
            || host
                .trim_matches(['[', ']'])
                .parse::<std::net::IpAddr>()
                .is_ok_and(|address| address.is_loopback())
    });
    if key.is_none() && !is_loopback {
        return Err(invalid("set OPENAI_API_KEY or --api-key-file for a hosted endpoint").into());
    }
    let client = Client::builder()
        .timeout(Duration::from_secs(args.timeout_secs))
        .connect_timeout(Duration::from_secs(args.timeout_secs.min(15)))
        .redirect(reqwest::redirect::Policy::none())
        .retry(reqwest::retry::never())
        .build()?;
    let numbered: String = source
        .split_inclusive('\n')
        .enumerate()
        .map(|(index, line)| format!("{} | {line}", index + 1))
        .collect();
    // The request deadline also bounds a body that keeps sending small chunks.
    let mut request = client.post(url).timeout(Duration::from_secs(args.timeout_secs)).json(&json!({
        "model": args.model,
        "messages": [
            {"role": "system", "content": PROMPT},
            {"role": "user", "content": json!({"source_markdown_with_line_numbers": numbered}).to_string()}
        ],
        "response_format": {"type": "json_object"},
        "max_completion_tokens": args.max_output_tokens,
        "stream": false
    }));
    if let Some(key) = key {
        let mut authorization = HeaderValue::from_str(&format!("Bearer {key}"))
            .map_err(|_| invalid("API key contains invalid header characters"))?;
        authorization.set_sensitive(true);
        request = request.header(AUTHORIZATION, authorization);
    }
    let response = request.send().map_err(reqwest::Error::without_url)?;
    if !response.status().is_success() {
        // Providers can echo credentials or source text in error bodies.
        return Err(invalid(format!(
            "atomization endpoint returned HTTP {}",
            response.status()
        ))
        .into());
    }
    let body = read_bounded(response, MAX_RESPONSE_BYTES, "API response")?;
    serde_json::from_str(&body)
        .map_err(|_| invalid("endpoint returned an invalid chat completion JSON object").into())
}

fn line_starts(source: &str) -> Vec<usize> {
    std::iter::once(0)
        .chain(
            source
                .match_indices('\n')
                .map(|(index, _)| index + 1)
                .filter(|index| *index < source.len()),
        )
        .collect()
}

fn body_start(source: &str, starts: &[usize]) -> io::Result<usize> {
    if source.trim_start_matches('\u{feff}').lines().next() != Some("---") {
        return Ok(0);
    }
    for (index, line) in source.lines().enumerate().skip(1) {
        if line == "---" || line == "..." {
            return Ok(starts.get(index + 1).copied().unwrap_or(source.len()));
        }
    }
    Err(invalid("unterminated YAML frontmatter"))
}

fn evidence(
    citation: Citation,
    source: &str,
    starts: &[usize],
    body: usize,
) -> io::Result<Evidence> {
    if citation.quote.trim().is_empty()
        || citation.start_line == 0
        || citation.start_line > citation.end_line
        || citation.end_line > starts.len()
    {
        return Err(invalid("empty quotation or invalid evidence line range"));
    }
    let window_start = starts[citation.start_line - 1];
    let window_end = starts
        .get(citation.end_line)
        .copied()
        .unwrap_or(source.len());
    let window = &source[window_start..window_end];
    let offset = window
        .find(&citation.quote)
        .ok_or_else(|| invalid("evidence quotation does not match the cited source lines"))?;
    // Search after one character so overlapping occurrences also count.
    let following = offset
        + citation
            .quote
            .chars()
            .next()
            .expect("quotation is nonempty")
            .len_utf8();
    if window[following..].contains(&citation.quote) {
        return Err(invalid(
            "evidence quotation is ambiguous within the cited source lines",
        ));
    }
    let start_byte = window_start + offset;
    let end_byte = start_byte + citation.quote.len();
    let actual_start = starts.partition_point(|offset| *offset <= start_byte);
    let actual_end = starts.partition_point(|offset| *offset < end_byte);
    if actual_start != citation.start_line || actual_end != citation.end_line {
        return Err(invalid(
            "evidence line range must match the quotation's first and last lines",
        ));
    }
    if start_byte < body {
        return Err(invalid(
            "evidence must cite skill content, not YAML frontmatter",
        ));
    }
    Ok(Evidence {
        quote: citation.quote,
        start_line: actual_start,
        end_line: actual_end,
        start_byte,
        end_byte,
    })
}

fn atoms(
    completion: &mut Completion,
    source: &str,
    starts: &[usize],
    body: usize,
) -> tokenizers::Result<Vec<Atom>> {
    if completion.choices.len() != 1 {
        return Err(invalid("endpoint must return exactly one completion choice").into());
    }
    let choice = completion.choices.remove(0);
    if choice
        .message
        .refusal
        .as_ref()
        .is_some_and(|refusal| !refusal.is_empty())
    {
        return Err(invalid("model refused atomization").into());
    }
    match choice.finish_reason.as_deref() {
        Some("stop") => (),
        Some("length") => {
            return Err(invalid("model output was truncated; increase --max-output-tokens").into());
        }
        _ => return Err(invalid("model did not finish with a complete text response").into()),
    }
    let content = choice
        .message
        .content
        .ok_or_else(|| invalid("model returned no text content"))?;
    let candidates: Candidates = serde_json::from_str(&content).map_err(|error| {
        invalid(format!(
            "invalid atom JSON at line {}, column {}",
            error.line(),
            error.column()
        ))
    })?;
    let mut result: Vec<Atom> = Vec::new();
    for (index, candidate) in candidates.atoms.into_iter().enumerate() {
        let statement = candidate.statement.trim().to_owned();
        if statement.is_empty() || candidate.evidence.is_empty() {
            return Err(
                invalid(format!("atom {} needs a statement and evidence", index + 1)).into(),
            );
        }
        let citations = candidate
            .evidence
            .into_iter()
            .map(|citation| evidence(citation, source, starts, body))
            .collect::<io::Result<Vec<_>>>()
            .map_err(|error| invalid(format!("atom {}: {error}", index + 1)))?;
        if let Some(existing) = result
            .iter_mut()
            .find(|atom| atom.statement == statement && atom.kind == candidate.kind)
        {
            existing.evidence.extend(citations);
        } else {
            result.push(Atom {
                id: String::new(),
                statement,
                kind: candidate.kind,
                evidence: citations,
            });
        }
    }
    for atom in &mut result {
        atom.evidence.sort_by(|left, right| {
            (left.start_byte, left.end_byte).cmp(&(right.start_byte, right.end_byte))
        });
        atom.evidence.dedup();
    }
    result.sort_by(|left, right| {
        (left.evidence[0].start_byte, &left.statement)
            .cmp(&(right.evidence[0].start_byte, &right.statement))
    });
    for (index, atom) in result.iter_mut().enumerate() {
        atom.id = format!("atom-{:04}", index + 1);
    }
    Ok(result)
}

pub fn run(args: AtomizeArgs) -> tokenizers::Result<()> {
    if args.model.trim().is_empty() {
        return Err(invalid("--model cannot be empty").into());
    }
    let url = endpoint(&args.base_url)?;
    let source = if args.path == Path::new("-") {
        read_bounded(io::stdin().lock(), args.max_input_bytes, "input")?
    } else {
        read_bounded(File::open(&args.path)?, args.max_input_bytes, "input")?
    };
    if source.trim().is_empty() {
        return Err(invalid("input Markdown is empty").into());
    }
    let starts = line_starts(&source);
    let body = body_start(&source, &starts)?;
    // Create the temporary output before spending a request on an unwritable path.
    let mut output = if let Some(path) = &args.output {
        if path.is_dir() {
            return Err(invalid("output path is a directory; choose a JSON file path").into());
        }
        if args.path != Path::new("-")
            && path.exists()
            && fs::canonicalize(path)? == fs::canonicalize(&args.path)?
        {
            return Err(invalid("output must not overwrite the input skill").into());
        }
        let parent = path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
            .unwrap_or(Path::new("."));
        Some(NamedTempFile::new_in(parent)?)
    } else {
        None
    };
    eprintln!(
        "atomize: requesting atoms for {} bytes (timeout {}s)",
        source.len(),
        args.timeout_secs
    );
    let (stop, receiver) = mpsc::channel::<()>();
    let heartbeat = thread::spawn(move || {
        let started = Instant::now();
        while matches!(
            receiver.recv_timeout(Duration::from_secs(15)),
            Err(mpsc::RecvTimeoutError::Timeout)
        ) {
            eprintln!(
                "atomize: waiting for model response ({}s)",
                started.elapsed().as_secs()
            );
        }
    });
    let response = request(&args, url, &source);
    drop(stop);
    let _ = heartbeat.join();
    let mut completion = response?;
    let atoms = atoms(&mut completion, &source, &starts, body)?;
    let document = Document {
        schema_version: 1,
        source: Source {
            path: args.path.display().to_string(),
            sha256: format!("{:x}", Sha256::digest(source.as_bytes())),
            bytes: source.len(),
        },
        prompt: Prompt {
            version: PROMPT_VERSION,
            sha256: format!("{:x}", Sha256::digest(PROMPT.as_bytes())),
        },
        model: Model {
            requested: args.model,
            returned: completion.model,
        },
        usage: completion.usage,
        atoms,
    };
    let mut serialized = serde_json::to_vec_pretty(&document)?;
    serialized.push(b'\n');
    if let Some(file) = &mut output {
        file.write_all(&serialized)?;
        file.as_file().sync_all()?;
    }
    if let Some(file) = output {
        file.persist(args.output.as_ref().expect("output path was checked"))?;
    } else {
        io::stdout().lock().write_all(&serialized)?;
    }
    eprintln!(
        "atomize: validated {} atoms; semantic review is still required",
        document.atoms.len()
    );
    Ok(())
}
