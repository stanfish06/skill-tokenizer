use clap::Parser as ClapParser;
use pulldown_cmark::{Event, Options, Parser};
use std::fs;
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use tokenizers::decoders::byte_fallback::ByteFallback;
use tokenizers::models::bpe::{BPE, BpeTrainer, BpeTrainerBuilder};
use tokenizers::normalizers::{Lowercase, Sequence, Strip, StripAccents, unicode::NFC};
use tokenizers::pre_tokenizers::whitespace::WhitespaceSplit;
use tokenizers::processors::template::TemplateProcessing;
use tokenizers::{AddedToken, Tokenizer, TokenizerBuilder, TokenizerImpl};

#[derive(ClapParser)]
#[command(
    name = "skill-tokenizer",
    version,
    about = "tokenizer for agent skills"
)]
enum Cli {
    /// train a BPE tokenizer on every .md file under PATH
    Train {
        path: PathBuf,
        #[arg(short, long, default_value = "tokenizer.json")]
        output: PathBuf,
        #[arg(long, default_value_t = 1000)]
        vocab_size: usize,
    },
    /// tokenize stdin with a saved tokenizer json
    Encode {
        #[arg(default_value = "tokenizer.json")]
        model: PathBuf,
    },
}

type Model = TokenizerImpl<BPE, Sequence, WhitespaceSplit, TemplateProcessing, ByteFallback>;

fn initialize_model() -> tokenizers::Result<Model> {
    let normalizer = Sequence::new(vec![
        Strip::new(true, true).into(),
        StripAccents.into(),
        Lowercase.into(),
        NFC.into(),
    ]);
    let post_processor = TemplateProcessing::builder()
        .try_single("[CLS] $0 [SEP]")?
        .try_pair("[CLS]:0 $A:0 [SEP]:0 $B:1 [SEP]:1")?
        .special_tokens(vec![("[CLS]", 1), ("[SEP]", 2)])
        .build()?;
    TokenizerBuilder::new()
        .with_model(BPE::builder().unk_token("[UNK]".into()).build()?)
        .with_normalizer(Some(normalizer))
        .with_pre_tokenizer(Some(WhitespaceSplit))
        .with_post_processor(Some(post_processor))
        .with_decoder(Some(ByteFallback::default()))
        .build()
}

fn initialize_trainer(vocab_size: usize) -> BpeTrainer {
    // special token order fixes ids: [UNK]=0 [CLS]=1 [SEP]=2, matching the template above
    let specials = ["[UNK]", "[CLS]", "[SEP]"].map(|t| AddedToken::from(t, true));
    BpeTrainerBuilder::new()
        .show_progress(true)
        .vocab_size(vocab_size)
        .min_frequency(0)
        .special_tokens(specials.to_vec())
        .build()
}

fn md_files(dir: &Path) -> Vec<PathBuf> {
    let mut out = vec![];
    for entry in fs::read_dir(dir).unwrap().flatten() {
        let p = entry.path();
        if p.is_dir() {
            out.extend(md_files(&p));
        } else if p.extension().is_some_and(|e| e == "md") {
            out.push(p);
        }
    }
    out
}

fn markdown_text(path: &Path) -> Vec<String> {
    let src = fs::read_to_string(path).unwrap();
    Parser::new_ext(&src, Options::all())
        .filter_map(|e| match e {
            Event::Text(s)
            | Event::Code(s)
            | Event::Html(s)
            | Event::InlineHtml(s)
            | Event::DisplayMath(s)
            | Event::InlineMath(s) => Some(s.into_string()),
            _ => None,
        })
        .collect()
}

fn main() -> tokenizers::Result<()> {
    match Cli::parse() {
        Cli::Train {
            path,
            output,
            vocab_size,
        } => {
            let files = md_files(&path);
            let mut trainer = initialize_trainer(vocab_size);
            initialize_model()?
                .train(&mut trainer, files.iter().flat_map(|f| markdown_text(f)))?
                .save(&output, false)?;
        }
        Cli::Encode { model } => {
            let tokenizer = Tokenizer::from_file(model)?;
            let mut input = String::new();
            io::stdin().read_to_string(&mut input)?;
            let encoding = tokenizer.encode(input, true)?;
            println!("{}", encoding.get_tokens().join(" "));
            println!("{:?}", encoding.get_ids());
        }
    }
    Ok(())
}
