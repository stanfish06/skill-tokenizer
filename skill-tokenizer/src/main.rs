use clap::{Parser as ClapParser, Subcommand};
use pulldown_cmark::{Event, Parser, html};
use std::env;
use std::fs;
use std::path::PathBuf;
use tokenizers::TokenizerBuilder;
use tokenizers::TokenizerImpl;
use tokenizers::decoders::byte_fallback::ByteFallback;
use tokenizers::models::bpe::BPE;
use tokenizers::normalizers::{Lowercase, Sequence, Strip, StripAccents, unicode::NFC};
use tokenizers::pre_tokenizers::whitespace::WhitespaceSplit;
use tokenizers::processors::template::TemplateProcessing;

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

fn train() {}

fn encode() {}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    let file_path = &args[0];
    let content = fs::read_to_string(file_path);
    match &content {
        Ok(v) => {
            let parser = Parser::new(v).map(|event| {
                match &event {
                    Event::Start(tag) => println!("Start: {:?}", tag),
                    Event::End(tag) => println!("End: {:?}", tag),
                    Event::Html(s) => println!("Html: {:?}", s),
                    Event::InlineHtml(s) => println!("InlineHtml: {:?}", s),
                    Event::Text(s) => {
                        println!("Text: {:?}", s);
                    }
                    Event::Code(s) => println!("Code: {:?}", s),
                    Event::DisplayMath(s) => println!("DisplayMath: {:?}", s),
                    Event::InlineMath(s) => println!("Math: {:?}", s),
                    Event::FootnoteReference(s) => println!("FootnoteReference: {:?}", s),
                    Event::TaskListMarker(b) => println!("TaskListMarker: {:?}", b),
                    Event::SoftBreak => println!("SoftBreak"),
                    Event::HardBreak => println!("HardBreak"),
                    Event::Rule => println!("Rule"),
                };
                event
            });
            let mut html_output = String::new();
            html::push_html(&mut html_output, parser);
        }
        Err(e) => println!("Failed to read file: {:?}", e),
    }
}
