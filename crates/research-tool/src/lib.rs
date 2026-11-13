//! `psm-research`: the dev-Mac research tool. R1: the benchmark (probes, visual-search and model
//! benchmarks); it makes real provider calls only when run by a person with keys in the
//! environment; every exchange is recorded under the out folder (MC-A11 layout), scanned for key
//! values before it is written, and counted against the spending ceiling and the call caps.
//! R9: `report` and `replay` over an export of the app's research record (08A §10A).

pub mod answer_key;
pub mod bench_models;
pub mod bench_search;
pub mod budget;
pub mod cloud_vision;
pub mod export_reader;
pub mod fixtures;
pub mod http;
pub mod openai;
pub mod pages;
pub mod probes;
pub mod recording;
pub mod replay;
pub mod report;
pub mod score;
pub mod serpapi;
pub mod shop;

use budget::{Ledger, Limits, Prices, parse_micro_usd};
use http::HttpSeam;
use recording::{Caller, Secrets};
use std::{io::Write, path::PathBuf};

pub const USAGE: &str = "usage: psm-research <command> --out <dir> [--ceiling-usd <n>] [options]
commands:
  probe <id>      v1 … v10, ep4 (needs --ceiling-usd)
  bench-search    both visual-search routes on every test piece, then the pages (needs --ceiling-usd)
  fixtures <dest> copy sanitized captures to <dest> (with --bench1 <a02>: one reply per stage)
  recost          recompute recorded OpenAI costs from prices.json
  score-search    score the recorded visual-search benchmark and write <out>/report.md
  bench-models    the per-stage model matrix (needs --ceiling-usd; --bench1 <a01 out folder>)
  score-models    score the recorded model matrix and write <out>/report_models.md
  report <export> what was accepted, edited, cleared per stage, cost and time (an export folder
                  or zip, or a benchmark recording folder) [--json]
  replay <export> --run <id> --stage <stage> [--model <m>] [--effort <e>] [--instructions <file>]
                  [--out <dir>]; the key is OPENAI_API_KEY from the environment
options: --pieces <a-b>  --test-file <path>  --corrections <path>  --routes <r,…>  --models <a,b,c>  --stages <s,…>  --efforts <e,…>";

/// Parsed command line.
#[derive(Debug, Clone, Default)]
pub struct Args {
    pub command: String,
    pub positional: Vec<String>,
    pub out: Option<PathBuf>,
    pub ceiling_usd: Option<String>,
    pub pieces: Option<(u32, u32)>,
    pub test_file: PathBuf,
    pub corrections: PathBuf,
    pub bench1: Option<PathBuf>,
    pub models: Vec<String>,
    pub stages: Vec<String>,
    pub efforts: Vec<String>,
    /// `bench-search` routes to run (`serpapi_lens`, `cloud_vision`); empty = both.
    pub routes: Vec<String>,
    /// `replay --run`.
    pub run: Option<i64>,
    /// `replay --stage`.
    pub stage: Option<String>,
    /// `replay --model`, `--effort`, `--instructions <file>`.
    pub model: Option<String>,
    pub effort: Option<String>,
    pub instructions: Option<PathBuf>,
    /// `report --json`.
    pub json: bool,
}

pub fn parse_args(args: &[String]) -> Result<Args, String> {
    let mut a = Args {
        test_file: PathBuf::from(answer_key::TEST_FILE),
        corrections: PathBuf::from(answer_key::CORRECTIONS_FILE),
        ..Args::default()
    };
    let mut it = args.iter();
    a.command = it.next().cloned().ok_or("no command")?;
    let list = |v: &str| v.split(',').map(str::to_owned).collect::<Vec<_>>();
    while let Some(arg) = it.next() {
        let mut value = || it.next().cloned().ok_or(format!("{arg} needs a value"));
        match arg.as_str() {
            "--out" => a.out = Some(PathBuf::from(value()?)),
            "--ceiling-usd" => a.ceiling_usd = Some(value()?),
            "--test-file" => a.test_file = PathBuf::from(value()?),
            "--corrections" => a.corrections = PathBuf::from(value()?),
            "--bench1" => a.bench1 = Some(PathBuf::from(value()?)),
            "--models" => a.models = list(&value()?),
            "--stages" => a.stages = list(&value()?),
            "--efforts" => a.efforts = list(&value()?),
            "--routes" => a.routes = list(&value()?),
            "--run" => a.run = Some(value()?.parse().map_err(|_| "bad --run")?),
            "--stage" => a.stage = Some(value()?),
            "--model" => a.model = Some(value()?),
            "--effort" => a.effort = Some(value()?),
            "--instructions" => a.instructions = Some(PathBuf::from(value()?)),
            "--json" => a.json = true,
            "--pieces" => {
                let v = value()?;
                let (from, to) = v.split_once('-').unwrap_or((&v, &v));
                a.pieces = Some((
                    from.parse().map_err(|_| "bad --pieces")?,
                    to.parse().map_err(|_| "bad --pieces")?,
                ));
            }
            other if other.starts_with("--") => return Err(format!("unknown option {other}")),
            other => a.positional.push(other.to_owned()),
        }
    }
    Ok(a)
}

/// Everything a command needs.
pub struct Ctx<'a> {
    pub args: Args,
    pub out: PathBuf,
    pub caller: Caller<'a>,
    pub seam: &'a dyn HttpSeam,
}

impl Ctx<'_> {
    pub fn prices(&self) -> Result<Prices, String> {
        Prices::load(&self.out.join("prices.json"))
    }
}

/// Runs one command; the exit code. Nothing leaves before the arguments are complete.
pub fn run(args: &[String], seam: &dyn HttpSeam) -> i32 {
    let env = |name: &str| std::env::var(name).ok();
    run_with(args, seam, &env, &mut std::io::stdout())
}

/// `run` with the environment lookup and the printed output given: `report`, `replay` read their
/// key only through `env` and print only to `out`; the other commands are R1's, unchanged.
pub fn run_with(
    args: &[String],
    seam: &dyn HttpSeam,
    env: &dyn Fn(&str) -> Option<String>,
    out: &mut dyn Write,
) -> i32 {
    let result = match args.first().map(String::as_str) {
        Some("report" | "replay") => run_export_command(args, seam, env, out),
        _ => run_inner(args, seam),
    };
    match result {
        Ok(()) => 0,
        Err(e) => {
            eprintln!("psm-research: {e}");
            1
        }
    }
}

/// `report <export>` and `replay <export> …` (R9·U1).
fn run_export_command(
    raw: &[String],
    seam: &dyn HttpSeam,
    env: &dyn Fn(&str) -> Option<String>,
    out: &mut dyn Write,
) -> Result<(), String> {
    let args = parse_args(raw).map_err(|e| format!("{e}\n{USAGE}"))?;
    let path = args
        .positional
        .first()
        .ok_or(format!("{} needs an export path\n{USAGE}", args.command))?;
    let print = |out: &mut dyn Write, text: &str| {
        out.write_all(text.as_bytes())
            .map_err(|e| format!("output: {e}"))
    };
    if args.command == "replay" {
        // The key first: without it nothing is read, planned or sent.
        let key = env(replay::KEY_VARIABLE).filter(|k| !k.trim().is_empty());
        let Some(key) = key else {
            return Err(format!(
                "{} is not set; nothing was sent",
                replay::KEY_VARIABLE
            ));
        };
        let run_id = args.run.ok_or("replay needs --run <id>")?;
        let stage = args.stage.clone().ok_or("replay needs --stage <stage>")?;
        let instructions = match &args.instructions {
            Some(file) => Some(
                std::fs::read_to_string(file)
                    .map_err(|e| format!("cannot read {}: {e}", file.display()))?,
            ),
            None => None,
        };
        let opened = export_reader::open(std::path::Path::new(path))?;
        let run = opened
            .run(run_id)
            .ok_or(format!("run {run_id} is not in {path}"))?;
        let with = replay::Substitutions {
            model: args.model.clone(),
            effort: args.effort.clone(),
            instructions,
        };
        let (text, summary) = replay::run(run, &stage, &with, Some(&key), seam)?;
        let secrets = Secrets::new([key.clone()]);
        if secrets.found_in(text.as_bytes()) {
            return Err("the replay output contains the key; nothing was printed".into());
        }
        if let Some(dir) = &args.out {
            let file = dir.join(format!("replay-run{run_id}-{stage}.json"));
            let bytes = serde_json::to_vec_pretty(&summary).unwrap_or_default();
            recording::write_scanned(&secrets, &file, &bytes).map_err(|e| e.to_string())?;
        }
        return print(out, &text);
    }
    let opened = export_reader::open(std::path::Path::new(path))?;
    let text = match &opened.contents {
        export_reader::Contents::Export(runs) => {
            let report = report::export_report(runs);
            if args.json {
                serde_json::to_string_pretty(&report).unwrap_or_default() + "\n"
            } else {
                report::render_export(&report)
            }
        }
        export_reader::Contents::Bench(calls) => {
            let groups = report::bench_report(calls);
            if args.json {
                serde_json::to_string_pretty(&groups).unwrap_or_default() + "\n"
            } else {
                report::render_bench(&groups)
            }
        }
    };
    print(out, &text)
}

fn run_inner(raw: &[String], seam: &dyn HttpSeam) -> Result<(), String> {
    let args = parse_args(raw).map_err(|e| format!("{e}\n{USAGE}"))?;
    let out = args
        .out
        .clone()
        .ok_or(format!("--out is required\n{USAGE}"))?;
    let spends = matches!(
        args.command.as_str(),
        "probe" | "bench-search" | "bench-models"
    );
    let limits = if spends {
        let text = args
            .ceiling_usd
            .as_deref()
            .ok_or(format!("--ceiling-usd is required for {}", args.command))?;
        let ceiling = parse_micro_usd(text)?;
        if args.command == "bench-models" {
            Limits::u2(ceiling)
        } else {
            Limits::u1(ceiling)
        }
    } else {
        Limits::u1(0)
    };
    std::fs::create_dir_all(&out).map_err(|e| format!("out folder: {e}"))?;
    let ledger = Ledger::from_recordings(&out);
    let caller = Caller::new(seam, Secrets::from_env(), ledger, limits);
    let ctx = Ctx {
        args,
        out,
        caller,
        seam,
    };
    let result = match ctx.args.command.as_str() {
        "probe" => probes::run(&ctx),
        "bench-search" => bench_search::run(&ctx).and_then(|()| report::write_search(&ctx)),
        "score-search" => report::write_search(&ctx),
        "bench-models" => bench_models::run(&ctx).and_then(|()| bench_models::write_report(&ctx)),
        "score-models" => bench_models::write_report(&ctx),
        "fixtures" => fixtures::run(&ctx),
        "recost" => ctx.prices().and_then(|p| {
            let total = openai::recost(&ctx.out, &p, &ctx.caller.secrets.borrow())?;
            eprintln!(
                "recomputed: out folder total USD {}",
                budget::format_usd(total)
            );
            Ok(())
        }),
        other => Err(format!("unknown command {other}\n{USAGE}")),
    };
    let ledger = ctx.caller.ledger.borrow();
    eprintln!(
        "psm-research: {} request(s) sent this run; out folder total USD {}; calls per cap {:?}",
        ctx.caller.sent.borrow(),
        budget::format_usd(ledger.spent_micro),
        ledger.counts
    );
    result
}
