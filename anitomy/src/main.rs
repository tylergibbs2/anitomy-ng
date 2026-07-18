// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Command-line front-end for [`anitomy_ng`].
//!
//! Parses one or more anime video filenames (given as arguments, or read from
//! stdin one per line) and prints their elements as an aligned text table or as
//! JSON. Thin by design: all parsing lives in the `anitomy-ng` library.

use std::io::{self, BufRead, Write};
use std::process::ExitCode;

use anitomy_ng::{Element, Options};
use clap::Parser;

#[derive(Parser)]
#[command(
    name = "anitomy",
    version,
    about = "Parse anime video filenames into their elements.",
    long_about = "Parse anime video filenames into their elements.\n\n\
        Provide filenames as arguments, or pipe them to stdin (one per line):\n\
        \n    \
        anitomy '[Group] Title - 01 [1080p].mkv'\n    \
        ls *.mkv | anitomy --json"
)]
struct Cli {
    /// Filenames to parse. If none are given, they are read from stdin, one per
    /// line.
    #[arg(value_name = "FILENAME")]
    filenames: Vec<String>,

    /// Emit JSON (an array of { filename, elements }) instead of the text table.
    #[arg(short, long)]
    json: bool,

    /// Parse related filenames together, grouped by parent directory
    #[arg(short = 't', long)]
    together: bool,

    // clap derives each long flag from the field name (`no_episode` ->
    // `--no-episode`) and uses the doc comment as its help text.
    /// Disable episode parsing
    #[arg(long, help_heading = "Parsing toggles")]
    no_episode: bool,
    /// Disable episode-title parsing
    #[arg(long, help_heading = "Parsing toggles")]
    no_episode_title: bool,
    /// Disable file-checksum parsing
    #[arg(long, help_heading = "Parsing toggles")]
    no_file_checksum: bool,
    /// Disable file-extension parsing
    #[arg(long, help_heading = "Parsing toggles")]
    no_file_extension: bool,
    /// Disable part parsing
    #[arg(long, help_heading = "Parsing toggles")]
    no_part: bool,
    /// Disable release-group parsing
    #[arg(long, help_heading = "Parsing toggles")]
    no_release_group: bool,
    /// Disable season parsing
    #[arg(long, help_heading = "Parsing toggles")]
    no_season: bool,
    /// Disable title parsing
    #[arg(long, help_heading = "Parsing toggles")]
    no_title: bool,
    /// Disable video-resolution parsing
    #[arg(long, help_heading = "Parsing toggles")]
    no_video_resolution: bool,
    /// Disable year parsing
    #[arg(long, help_heading = "Parsing toggles")]
    no_year: bool,
}

impl Cli {
    /// Builds the parser [`Options`] from the `--no-*` toggles (each flag
    /// negates the corresponding, default-on category).
    fn options(&self) -> Options {
        Options {
            parse_episode: !self.no_episode,
            parse_episode_title: !self.no_episode_title,
            parse_file_checksum: !self.no_file_checksum,
            parse_file_extension: !self.no_file_extension,
            parse_part: !self.no_part,
            parse_release_group: !self.no_release_group,
            parse_season: !self.no_season,
            parse_title: !self.no_title,
            parse_video_resolution: !self.no_video_resolution,
            parse_year: !self.no_year,
        }
    }
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    let options = cli.options();

    // Collect inputs from args, or fall back to stdin (one filename per line).
    let inputs: Vec<String> = if cli.filenames.is_empty() {
        match read_stdin_lines() {
            Ok(lines) => lines,
            Err(err) => {
                eprintln!("anitomy: error reading stdin: {err}");
                return ExitCode::FAILURE;
            }
        }
    } else {
        cli.filenames.clone()
    };

    let results: Vec<(String, Vec<Element>)> = if cli.together {
        let parsed = parse_together_grouped(&inputs, options);
        inputs.into_iter().zip(parsed).collect()
    } else {
        inputs
            .into_iter()
            .map(|name| {
                let elements = anitomy_ng::parse(&name, options);
                (name, elements)
            })
            .collect()
    };

    // Buffer all output behind a single locked handle; ignore downstream
    // `BrokenPipe` (e.g. `| head`) rather than panicking on it.
    let stdout = io::stdout();
    let mut out = stdout.lock();
    let render = if cli.json {
        write_json(&mut out, &results)
    } else {
        write_table(&mut out, &results)
    };
    match render {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) if err.kind() == io::ErrorKind::BrokenPipe => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("anitomy: error writing output: {err}");
            ExitCode::FAILURE
        }
    }
}

fn read_stdin_lines() -> io::Result<Vec<String>> {
    let mut lines = Vec::new();
    for line in io::stdin().lock().lines() {
        let line = line?;
        // Skip blank lines so trailing newlines in a pipe don't parse as "".
        if !line.trim().is_empty() {
            lines.push(line);
        }
    }
    Ok(lines)
}

/// Parse `inputs` in per-directory groups via [`anitomy_ng::parse_together`],
/// returning one result per input in the original order.
fn parse_together_grouped(inputs: &[String], options: Options) -> Vec<Vec<Element>> {
    let mut groups: Vec<(&str, Vec<usize>)> = Vec::new();
    for (i, name) in inputs.iter().enumerate() {
        let key = dir_prefix(name);
        match groups.iter_mut().find(|(k, _)| *k == key) {
            Some((_, indices)) => indices.push(i),
            None => groups.push((key, vec![i])),
        }
    }

    let mut results: Vec<Vec<Element>> = vec![Vec::new(); inputs.len()];
    for (_, indices) in &groups {
        let group: Vec<&str> = indices
            .iter()
            .filter_map(|&i| inputs.get(i))
            .map(String::as_str)
            .collect();
        for (&i, elements) in indices
            .iter()
            .zip(anitomy_ng::parse_together(&group, options))
        {
            if let Some(slot) = results.get_mut(i) {
                *slot = elements;
            }
        }
    }
    results
}

/// The directory portion of `name` (up to the last `/` or `\`), or `""`.
fn dir_prefix(name: &str) -> &str {
    match name.rfind(['/', '\\']) {
        Some(pos) => name.get(..pos).unwrap_or(""),
        None => "",
    }
}

/// Human-readable, aligned `kind  value` rows under each filename.
fn write_table(out: &mut impl Write, results: &[(String, Vec<Element>)]) -> io::Result<()> {
    for (i, (filename, elements)) in results.iter().enumerate() {
        if i > 0 {
            writeln!(out)?;
        }
        writeln!(out, "{filename}")?;
        if elements.is_empty() {
            writeln!(out, "  (no elements)")?;
            continue;
        }
        let width = elements
            .iter()
            .map(|e| e.kind.as_str().len())
            .max()
            .unwrap_or(0);
        for element in elements {
            writeln!(
                out,
                "  {:<width$}  {}",
                element.kind.as_str(),
                element.value,
                width = width
            )?;
        }
    }
    Ok(())
}

/// A JSON array of `{ "filename", "elements": [{ "kind", "value", "position" }] }`.
fn write_json(out: &mut impl Write, results: &[(String, Vec<Element>)]) -> io::Result<()> {
    let json: Vec<serde_json::Value> = results
        .iter()
        .map(|(filename, elements)| {
            let elements: Vec<serde_json::Value> = elements
                .iter()
                .map(|e| {
                    serde_json::json!({
                        "kind": e.kind.as_str(),
                        "value": e.value,
                        "position": e.position,
                    })
                })
                .collect();
            serde_json::json!({ "filename": filename, "elements": elements })
        })
        .collect();
    writeln!(out, "{}", serde_json::Value::Array(json))
}

#[cfg(test)]
mod tests {
    use super::*;
    use anitomy_ng::ElementKind;

    fn episodes(elements: &[Element]) -> Vec<&str> {
        elements
            .iter()
            .filter(|e| e.kind == ElementKind::Episode)
            .map(|e| e.value.as_str())
            .collect()
    }

    #[test]
    fn dir_prefix_covers_both_separators_and_bare_names() {
        assert_eq!(dir_prefix("a/b/c.mkv"), "a/b");
        assert_eq!(dir_prefix("a\\b\\c.mkv"), "a\\b");
        assert_eq!(dir_prefix("c.mkv"), "");
    }

    #[test]
    fn grouped_folder_recovers_episode_from_range() {
        let inputs = vec![
            "Series (01-12)[batch]/Series - 07.mkv".to_string(),
            "Series (01-12)[batch]/Series - 08.mkv".to_string(),
        ];
        let results = parse_together_grouped(&inputs, Options::default());
        assert_eq!(episodes(&results[0]), ["07"]);
        assert_eq!(episodes(&results[1]), ["08"]);
    }

    #[test]
    fn separate_directories_group_apart_in_original_order() {
        let inputs = vec![
            "A/Show - 01.mkv".to_string(),
            "B/Other - 05.mkv".to_string(),
            "A/Show - 02.mkv".to_string(),
        ];
        let results = parse_together_grouped(&inputs, Options::default());
        assert_eq!(episodes(&results[0]), ["01"]);
        assert_eq!(episodes(&results[1]), ["05"]);
        assert_eq!(episodes(&results[2]), ["02"]);
    }
}
