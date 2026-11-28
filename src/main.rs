use absys::machine::{Negation, Notation, Opts, Search, Select};
use absys::report::{derivation, run, state, Show};
use absys::session::{answers, notation_for, Session, DEFAULT_BUDGET};
use std::io::{BufRead, IsTerminal, Write};
use std::process::exit;

const USAGE: &str = "\
absys FILE                accept the text an assertion at a time; show every derivation left
absys repl [FILE]         the incremental system: type assertions, and see what they do
absys verify [DIR]        run every example (examples/*.abs, *.ab1) against its .out
absys decisions           what the reconstruction does, and what each rule rests on
absys permute FILE        accept the assertions in every order; which orders give which answers
absys graph FILE          the derivation tree and the suspended processes, as Graphviz dot
absys json [FILE|-]       the run as JSON, for the playground
absys lsp                 a language server on standard input and output

options:
  --trace                 every step, under the assertion that caused it
  --each                  the state after each assertion
  --env                   each derivation's whole binding environment (X₁/1, L₁/[2,U], …)
  --search fair|depth-first       which derivation next (default fair, as Absys)
  --select in-place|fifo          which literal of a derivation next (default in-place)
  --negation immediate|delayed    when not is elaborated (default immediate, as Absys)
  --steps N               the budget of steps for each assertion (default 100000)
  --wake front|back       where a returned relation goes in the goal (default front)
  --measure               after each assertion, what the computation holds and the peak of each
                          quantity while it ran, on standard error (measurement only)
  --peak-every N          with --measure, take the peak every N steps (default 1, every step:
                          exact; 0: only at the start and end of each assertion; otherwise sampled)
  --full                  print lists and derivations in full, not cut short (measurement only)
  --notation 1990|1969|1982  (default from the file name: .ab1 is 1969, .ab2 1982, anything else 1990)";

const DECISIONS: &str = include_str!("../docs/decisions.md");

struct Args {
    opts: Opts,
    show: Show,
    budget: usize,
    notation: Option<Notation>,
    measure: bool,
    peak_every: usize,
    rest: Vec<String>,
}

fn fail(m: &str) -> ! {
    eprintln!("{m}");
    exit(2)
}

fn args() -> Args {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut a = Args {
        opts: Opts::default(),
        show: Show::default(),
        budget: DEFAULT_BUDGET,
        notation: None,
        measure: false,
        peak_every: 1,
        rest: vec![],
    };
    let mut i = 0;
    let value = |i: &mut usize| -> String {
        *i += 1;
        args.get(*i).cloned().unwrap_or_else(|| fail(USAGE))
    };
    while i < args.len() {
        match args[i].as_str() {
            "--trace" => a.show.trace = true,
            "--each" => a.show.each = true,
            "--env" => a.show.env = true,
            "--measure" => a.measure = true,
            "--peak-every" => {
                a.peak_every = value(&mut i).parse().unwrap_or_else(|_| fail("--peak-every N"))
            }
            "--full" => absys::machine::FULL.store(true, std::sync::atomic::Ordering::Relaxed),
            "--search" => {
                a.opts.search = match value(&mut i).as_str() {
                    "fair" => Search::Fair,
                    "depth-first" => Search::DepthFirst,
                    _ => fail("--search fair|depth-first"),
                }
            }
            "--select" => {
                a.opts.select = match value(&mut i).as_str() {
                    "in-place" => Select::InPlace,
                    "fifo" => Select::Fifo,
                    _ => fail("--select in-place|fifo"),
                }
            }
            "--negation" => {
                a.opts.negation = match value(&mut i).as_str() {
                    "immediate" => Negation::Immediate,
                    "delayed" => Negation::Delayed,
                    _ => fail("--negation immediate|delayed"),
                }
            }
            "--wake" => {
                a.opts.wake = match value(&mut i).as_str() {
                    "front" => absys::machine::Wake::Front,
                    "back" => absys::machine::Wake::Back,
                    _ => fail("--wake front|back"),
                }
            }
            "--steps" => a.budget = value(&mut i).parse().unwrap_or_else(|_| fail("--steps N")),
            "--notation" => {
                a.notation = Some(match value(&mut i).as_str() {
                    "1969" => Notation::Absys1,
                    "1982" => Notation::Elcock1982,
                    "1990" => Notation::Retrospective,
                    _ => fail("--notation 1990|1969|1982"),
                })
            }
            "-h" | "--help" => {
                println!("{USAGE}");
                exit(0)
            }
            x => a.rest.push(x.to_string()),
        }
        i += 1;
    }
    a
}

fn read(path: &str) -> String {
    if path == "-" {
        let mut t = String::new();
        std::io::Read::read_to_string(&mut std::io::stdin(), &mut t).unwrap();
        return t;
    }
    std::fs::read_to_string(path).unwrap_or_else(|e| fail(&format!("{path}: {e}")))
}

fn report_error(path: &str, text: &str, e: &absys::syntax::Error) -> ! {
    let (l, c) = absys::syntax::line_col(text, e.span.start);
    fail(&format!("{path}:{}:{}: {}", l + 1, c + 1, e.message))
}

fn main() {
    let mut a = args();
    let rest: Vec<&str> = a.rest.iter().map(String::as_str).collect();
    match rest.as_slice() {
        [] | ["help"] => println!("{USAGE}"),
        ["decisions"] => print!("{DECISIONS}"),
        ["lsp"] => {
            let stdin = std::io::stdin();
            if let Err(e) = absys::lsp::serve(&mut stdin.lock(), &mut std::io::stdout()) {
                fail(&e)
            }
        }
        ["json"] => {
            a.opts.notation = a.notation.unwrap_or(Notation::Retrospective);
            println!("{}", absys::play::run_json(&read("-"), a.opts, a.budget));
        }
        ["json", f] => {
            a.opts.notation = a.notation.unwrap_or_else(|| notation_for(f));
            println!("{}", absys::play::run_json(&read(f), a.opts, a.budget));
        }
        ["permute", f] => {
            let text = read(f);
            a.opts.notation = a.notation.unwrap_or_else(|| notation_for(f));
            let budget = if a.budget == DEFAULT_BUDGET { 20_000 } else { a.budget };
            match absys::research::permute_text(&text, a.opts, budget) {
                Ok(out) => print!("{out}"),
                Err(e) => report_error(f, &text, &e),
            }
        }
        ["graph", f] => {
            let text = read(f);
            a.opts.notation = a.notation.unwrap_or_else(|| notation_for(f));
            let mut s = Session::new(a.opts);
            s.budget = a.budget;
            if let Err(e) = s.load(&text) {
                report_error(f, &text, &e)
            }
            print!("{}", absys::research::graph(&s));
        }
        ["verify"] => verify("examples", a.opts),
        ["verify", dir] => verify(dir, a.opts),
        ["repl"] => repl(None, a),
        ["repl", f] => {
            let f = f.to_string();
            repl(Some(f), a)
        }
        [f] => {
            let text = read(f);
            a.opts.notation = a.notation.unwrap_or_else(|| notation_for(f));
            let mut s = Session::new(a.opts);
            s.budget = a.budget;
            s.measuring = a.measure;
            s.peak_every = a.peak_every;
            match run(&mut s, &text, a.show) {
                Ok(out) => print!("{out}"),
                Err(e) => report_error(f, &text, &e),
            }
            if a.measure {
                eprint!("{}", absys::report::measures(&s));
            }
        }
        _ => fail(USAGE),
    }
}

/// Every example, run as `absys FILE --env` and compared with FILE's `.out`.
fn verify(dir: &str, opts: Opts) {
    let mut names: Vec<String> = std::fs::read_dir(dir)
        .unwrap_or_else(|e| fail(&format!("{dir}: {e}")))
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .filter(|n| n.ends_with(".abs") || n.ends_with(".ab1") || n.ends_with(".ab2"))
        .collect();
    names.sort();
    let (mut good, mut bad, mut missing) = (0, 0, 0);
    for n in &names {
        let path = format!("{dir}/{n}");
        let text = read(&path);
        let mut s = Session::new(Opts {
            notation: notation_for(n),
            ..opts
        });
        let got = match run(
            &mut s,
            &text,
            Show {
                env: true,
                ..Show::default()
            },
        ) {
            Ok(o) => o,
            Err(e) => format!("error: {}\n", e.message),
        };
        let out = path.rsplit_once('.').unwrap().0.to_string() + ".out";
        match std::fs::read_to_string(&out) {
            Ok(want) if want == got => {
                good += 1;
                println!("✓ {n}");
            }
            Ok(_) => {
                bad += 1;
                println!("✗ {n}: differs from {out}");
            }
            Err(_) => {
                missing += 1;
                println!("? {n}: no {out}");
            }
        }
    }
    println!(
        "\n{} examples: {good} agree, {bad} differ, {missing} without an .out",
        names.len()
    );
    if bad > 0 {
        exit(1)
    }
}

const REPL_HELP: &str = "\
Type assertions; each is added to every derivation, and whatever can be done is done.
  ?          the state: every derivation, what it knows and what is suspended
  ? X        the value of X in each derivation
  .env       the state, with each derivation's whole binding environment
  .trace     show every step from now on (.trace off to stop)
  .help      this
  .quit      end (or end of input)";

fn repl(file: Option<String>, a: Args) {
    let mut opts = a.opts;
    opts.notation = a
        .notation
        .unwrap_or_else(|| file.as_deref().map(notation_for).unwrap_or(Notation::Retrospective));
    let mut s = Session::new(opts);
    s.budget = a.budget;
    let mut show = a.show;
    let tty = std::io::stdin().is_terminal();
    if let Some(f) = &file {
        let text = read(f);
        match run(&mut s, &text, Show { each: false, ..show }) {
            Ok(_) => {
                let live = s.comp.derivs.len();
                println!(
                    "{f}: {} assertions accepted; {live} derivation{}",
                    s.reports.len(),
                    if live == 1 { "" } else { "s" }
                );
            }
            Err(e) => report_error(f, &text, &e),
        }
    }
    if tty {
        println!(
            "Absys ({} notation). .help for help.",
            match opts.notation {
                Notation::Absys1 => "1969",
                Notation::Elcock1982 => "1982",
                Notation::Retrospective => "1990",
            }
        );
    }
    let stdin = std::io::stdin();
    let mut lines = stdin.lock().lines();
    let mut buffer = String::new();
    // all the text accepted so far, so that spans and errors refer to it
    let mut text = String::new();
    loop {
        if tty {
            print!("{}", if buffer.is_empty() { "> " } else { "… " });
            let _ = std::io::stdout().flush();
        }
        let Some(Ok(line)) = lines.next() else { break };
        let t = line.trim();
        if buffer.is_empty() {
            match t {
                "" => continue,
                ".quit" | ".exit" => break,
                ".help" => {
                    println!("{REPL_HELP}");
                    continue;
                }
                ".trace" | ".trace on" => {
                    show.trace = true;
                    continue;
                }
                ".trace off" => {
                    show.trace = false;
                    continue;
                }
                "?" | ".env" => {
                    let mut out = String::new();
                    state(
                        &s,
                        Show {
                            env: t == ".env",
                            ..show
                        },
                        &mut out,
                    );
                    print!("{out}");
                    continue;
                }
                _ if t.starts_with('?') => {
                    let name = t[1..].trim();
                    if s.comp.derivs.is_empty() {
                        println!("no derivation survives");
                    }
                    for d in &s.comp.derivs {
                        match answers(d, opts.notation).into_iter().find(|x| x.name == name) {
                            Some(x) if x.via.is_empty() => println!("#{}: {} = {}", d.id, x.name, x.value),
                            Some(x) => println!(
                                "#{}: {} = {}   (suspended: {})",
                                d.id,
                                x.name,
                                x.value,
                                x.via.join(", ")
                            ),
                            None => println!("#{}: {name} does not occur", d.id),
                        }
                    }
                    continue;
                }
                _ => {}
            }
        }
        buffer.push_str(&line);
        buffer.push('\n');
        let items = match absys::session::parse(opts.notation, &buffer) {
            Ok(items) => items,
            Err(e) if e.span.start >= buffer.trim_end().len() => continue, // unfinished: read on
            Err(e) => {
                let (l, c) = absys::syntax::line_col(&buffer, e.span.start);
                println!("error at {}:{}: {}", l + 1, c + 1, e.message);
                buffer.clear();
                continue;
            }
        };
        let base = text.len();
        text.push_str(&buffer);
        buffer.clear();
        for mut item in items {
            item.span.start += base;
            item.span.end += base;
            let before = s.comp.derivs.len();
            let r = match s.accept(&text, &item) {
                Ok(r) => r.clone(),
                Err(e) => {
                    println!("error: {}", e.message);
                    continue;
                }
            };
            if show.trace {
                for e in &s.events()[r.events.0..r.events.1] {
                    println!("  {}#{:<4} {:<9} {}", "  ".repeat(e.depth), e.deriv, e.kind, e.text);
                }
            }
            let now = s.comp.derivs.len();
            if now == 0 {
                println!("no derivation survives: the assertions are unsatisfiable");
                if let Some((_, why)) = s.cx.failed.last() {
                    println!("  (the last failure: {why})");
                }
            } else {
                let mut out = String::new();
                if now <= 3 {
                    for d in &s.comp.derivs {
                        derivation(d, opts.notation, show, &mut out);
                    }
                } else {
                    out.push_str(&format!("  {now} derivations (? to see them)\n"));
                }
                if before > 0 && now != before {
                    out = format!("  {before} → {now} derivations\n{out}");
                }
                print!("{out}");
            }
            if r.exhausted {
                println!(
                    "  (the budget of {} steps ran out; derivations are still elaborating)",
                    s.budget
                );
            }
        }
    }
}
