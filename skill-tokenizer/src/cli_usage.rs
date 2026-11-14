use clap::{Command, ValueHint};
use clap_usage::usage::{Spec, SpecCommand, SpecComplete, SpecFlag};

pub fn spec(command: &mut Command) -> Spec {
    let mut spec = clap_usage::spec(command, "skill-tokenizer");
    enrich(command, &mut spec.cmd);
    spec
}

// clap_usage 5 omits path hints, environment names, and flag-triggered subcommands.
// Read them from Clap so the completion spec follows the CLI declarations.
fn enrich(command: &Command, spec: &mut SpecCommand) {
    for argument in command.get_arguments() {
        if let Some(flag) = spec.flags.iter_mut().find(|flag| {
            argument
                .get_long()
                .is_some_and(|long| flag.long.iter().any(|name| name == long))
        }) {
            flag.env = argument
                .get_env()
                .map(|name| name.to_string_lossy().into_owned());
        }
        let kind = match argument.get_value_hint() {
            ValueHint::FilePath => "file",
            ValueHint::DirPath => "dir",
            ValueHint::AnyPath => "path",
            _ => continue,
        };
        if let Some(name) = argument.get_value_names().and_then(|names| names.first()) {
            let name = name.to_ascii_lowercase();
            let mut complete = SpecComplete::new(&name);
            complete.type_ = Some(kind.to_owned());
            spec.complete.insert(name, complete);
        }
    }
    for subcommand in command.get_subcommands() {
        if let Some(long) = subcommand.get_long_flag() {
            let mut flag = SpecFlag::default();
            flag.name = long.to_owned();
            flag.long.push(long.to_owned());
            flag.help = subcommand.get_about().map(ToString::to_string);
            spec.flags.push(flag);
        }
        if let Some(child) = spec.subcommands.get_mut(subcommand.get_name()) {
            enrich(subcommand, child);
        }
    }
}
