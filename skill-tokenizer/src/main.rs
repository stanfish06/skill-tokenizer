use clap::{Parser as ClapParser, Subcommand};
use pulldown_cmark::{Event, Parser, html};
use std::env;
use std::fs;
use std::path::PathBuf;
use tokenizers::TokenizerBuilder;
use tokenizers::TokenizerImpl;
use tokenizers::decoders::byte_fallback::ByteFallback;
use tokenizers::models::bpe::BPE;
use tokenizers::models::bpe::BpeTrainer;
use tokenizers::models::bpe::BpeTrainerBuilder;
use tokenizers::normalizers::{Lowercase, Sequence, Strip, StripAccents, unicode::NFC};
use tokenizers::pre_tokenizers::whitespace::WhitespaceSplit;
use tokenizers::processors::template::TemplateProcessing;

#[derive(ClapParser)]
#[command(
    name = "skill-tokenizer",
    version,
    about = "tokenizer for agent skills"
)]
struct Cli {
    input: PathBuf,

    #[arg(short, long)]
    output: Option<PathBuf>,

    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,

    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    Run {
        #[arg(long, default_value_t = 4)]
        threads: usize,
    },
    Clean,
}

fn initialize_model() -> Result<
    TokenizerImpl<
        tokenizers::models::bpe::BPE,
        tokenizers::normalizers::Sequence,
        tokenizers::pre_tokenizers::whitespace::WhitespaceSplit,
        TemplateProcessing,
        tokenizers::decoders::byte_fallback::ByteFallback,
    >,
    Box<dyn std::error::Error + Send + Sync>,
> {
    let normalizer = Some(Sequence::new(vec![
        Strip::new(true, true).into(),
        StripAccents.into(),
        Lowercase.into(),
        NFC.into(),
    ]));
    let post_processor = TemplateProcessing::builder()
        .try_single("[CLS] $0 [SEP]")
        .unwrap()
        .try_pair("[CLS]:0 $A:0 [SEP]:0 $B:1 [SEP]:1")
        .unwrap()
        .special_tokens(vec![("[CLS]", 1), ("[SEP]", 0)])
        .build()
        .unwrap();
    let tokenizer = TokenizerBuilder::new()
        .with_model(BPE::default())
        .with_normalizer(normalizer)
        .with_pre_tokenizer(Some(WhitespaceSplit))
        .with_post_processor(Some(post_processor))
        .with_decoder(Some(ByteFallback::default()))
        .build();
    return tokenizer;
}

const DEFAULT_VOCAB_SIZE: usize = 1000;
fn initialize_trainer(vocab_size: usize) -> BpeTrainer {
    let trainer = BpeTrainerBuilder::new()
        .show_progress(true)
        .vocab_size(vocab_size)
        .min_frequency(0)
        .build();
    return trainer;
}

fn encode() {}

fn text_parser() {}
fn html_parser() {}
fn code_parser() {}
fn math_parser() {}

fn markdown_parser(f: &str) {
    let content = fs::read_to_string(f);
    match &content {
        Ok(v) => {
            let parser = Parser::new(v).map(|event| {
                match &event {
                    Event::Html(s) => html_parser(s),
                    Event::InlineHtml(s) => html_parser(s),
                    Event::Text(s) => text_parser(s),
                    Event::Code(s) => code_parser(s),
                    Event::DisplayMath(s) => math_parser(s),
                    Event::InlineMath(s) => math_parser(s),
                    // patterns that likely dont matter
                    // Event::Rule => println!("Rule"),
                    // Event::Start(tag) => println!("Start: {:?}", tag),
                    // Event::End(tag) => println!("End: {:?}", tag),
                    // Event::FootnoteReference(s) => println!("FootnoteReference: {:?}", s),
                    // Event::SoftBreak => println!("SoftBreak"),
                    // Event::HardBreak => println!("HardBreak"),
                    // Event::TaskListMarker(b) => println!("TaskListMarker: {:?}", b),
                    _ => (),
                };
                event
            });
        }
        Err(e) => println!("Failed to read file: {:?}", e),
    }
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    let paths = &args;
    let train_data = paths.into_iter().flat_map(|f| markdown_parser(f));

    let trainer = initialize_trainer(DEFAULT_VOCAB_SIZE);
    let mut model = initialize_model().unwrap();
    let pretty = false;
    model
        .train(trainer, train_data)
        .save("tokenizer.json", pretty)?;
}
