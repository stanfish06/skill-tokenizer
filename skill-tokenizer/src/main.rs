use pulldown_cmark::{Event, Parser, html};
use tokenizers::normalizer::NormalizedString;
use tokenizers::Normalizer;
use tokenizers::PreTokenizer;
use std::env;
use std::fs;
use tokenizers::pre_tokenizer::PreTokenizedString;
use tokenizers::pre_tokenizers::whitespace::WhitespaceSplit;
use tokenizers::normalizers::{Lowercase, Sequence, Strip};

fn preprocess(s: &str) -> PreTokenizedString {
    let mut normalized : NormalizedString = s.to_string().into();
    let normalizer = Sequence::new(vec![Strip::new(true, true).into(), Lowercase.into()]);
    normalizer.normalize(&mut normalized);
    let mut pre_tokenized = PreTokenizedString::from(normalized);
    WhitespaceSplit.pre_tokenize(&mut pre_tokenized);
    return pre_tokenized;
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
                        preprocess(s);
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
