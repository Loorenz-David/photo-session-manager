//! `psm-research`: the dev-Mac research benchmark tool (R1). It makes real provider calls only
//! when run by a person with keys in the environment; every exchange is recorded under the out
//! folder (MC-A11 layout), scanned for key values before it is written, and counted against the
//! spending ceiling and the call caps.

pub mod answer_key;
pub mod bench_search;
pub mod budget;
pub mod cloud_vision;
pub mod fixtures;
pub mod http;
pub mod openai;
pub mod pages;
pub mod probes;
pub mod recording;
pub mod report;
pub mod serpapi;
pub mod shop;

use budget::{Ledger, Limits, Prices, parse_micro_usd};
use http::HttpSeam;
use recording::{Caller, Secrets};
use std::path::PathBuf;

pub const USAGE: &str = "usage: psm-research <command> --out <dir> [--ceiling-usd <n>] [options]
commands:
  probe <id>      v1 … v10, ep4 (needs --ceiling-usd)
  bench-search    both visual-search routes on every test piece, then the pages (needs --ceiling-usd)
  fixtures <dest> copy sanitized captures to <dest> (with --bench1 <a02>: one reply per stage)
  recost          recompute recorded OpenAI costs from prices.json
  score-search    score the recorded visual-search benchmark and write <out>/report.md
  bench-models    the per-stage model matrix (needs --ceiling-usd; --bench1 <a01 out folder>)
  score-models    score the recorded model matrix and write <out>/report_models.md
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
    match run_inner(args, seam) {
        Ok(()) => 0,
        Err(e) => {
            eprintln!("psm-research: {e}");
            1
        }
    }
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
