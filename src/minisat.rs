use std::{
    alloc::Layout,
    error::Error,
    fs::File,
    io::{self, BufWriter, Write},
    ops::ControlFlow,
    os,
    path::PathBuf,
    process::exit,
    sync::{Arc, Mutex},
    thread::sleep,
    time::{Duration, Instant},
};

use clap::{Args, Parser, Subcommand};
use cpu_time::ProcessTime;
use rssat::solver::{self, SatSolver, Status};
use validator::Validate;

use crate::utils::{self, get_memory};

#[derive(Args, Validate)]
pub struct Arg {
    /// Input file
    #[arg(value_name = "INPUT")]
    input: Option<PathBuf>,
    #[arg(value_name = "OUTPUT")]
    output: Option<PathBuf>,
    /// The variable activity decay factor
    #[arg(long, value_name = "VAR_DECAY", default_value_t = 0.95, group = "core")]
    #[validate(range(exclusive_min = 0.0, exclusive_max = 1.0))]
    var_decay: f64,
    /// The clause activity decay factor
    #[arg(long = "cla-decay", default_value_t = 0.999, group = "core")]
    #[validate(range(exclusive_min = 0.0, exclusive_max = 1.0))]
    clause_decay: f64,
    #[arg(long = "rnd-freq", default_value_t = 0.0, group = "core")]
    #[validate(range(min = 0.0, max = 1.0))]
    /// The frequency with which the decision heuristic tries to choose a random variable
    random_var_freq: f64,

    #[arg(long = "rnd-seed", default_value_t = 91648253.0, group = "core")]
    #[validate(range(exclusive_min = 0.0))]
    /// Used by the random variable selection
    random_seed: f64,

    #[arg(long, default_value_t = 2, group = "core")]
    #[validate(range(min = 0, max = 2))]
    /// Controls conflict clause minimization (0=none, 1=basic, 2=deep)
    ccmin_mode: i32,
    #[arg(long, default_value_t = 2, group = "core")]
    #[validate(range(min = 0, max = 2))]
    /// Controls the level of phase saving (0=none, 1=limited, 2=full)
    phase_saving: i32,
    #[arg(long = "rnd-init", default_value_t = false, group = "core")]
    /// Randomize the initial activity
    rnd_init_act: bool,
    #[arg(long = "luby", default_value_t = true, group = "core")]
    /// Use the Luby restart sequence
    luby_restart: bool,
    #[arg(long = "rfirst", default_value_t = 100, group = "core")]
    /// The base restart interval
    restart_first: i32,
    #[arg(long = "rinc", default_value_t = 2.0, group = "core")]
    #[validate(range(min = 1.0))]
    /// Restart interval increase factor
    restart_inc: f64,
    #[arg(long = "gc-frac", default_value_t = 0.2, group = "core")]
    #[validate(range(exclusive_min = 0.0))]
    /// The fraction of wasted memory allowed before a garbage collection is triggered
    garbage_frac: f64,
    #[arg(long = "min-learnts", default_value_t = 0, group = "core")]
    #[validate(range(min = 0))]
    /// Minimum learnt clause limit
    min_learnts_lim: i32,

    // simp
    #[arg(long = "asymm", default_value_t = false, group = "simp")]
    /// Shrink clauses by asymmetric branching.
    use_asymm: bool,

    #[arg(long = "rcheck", default_value_t = false, group = "simp")]
    /// Check if a clause is already implied. (costly)
    use_rcheck: bool,
    #[arg(long = "elim", default_value_t = true, group = "simp")]
    /// Perform variable elimination.
    use_elim: bool,

    #[arg(long = "grow", default_value_t = 0, group = "simp")]
    #[validate(range(min = 0))]
    /// Allow a variable elimination step to grow by a number of clauses.
    grow: i32,

    #[arg(long = "cl-lim", default_value_t = 20, group = "simp")]
    #[validate(range(min = -1))]
    /// Variables are not eliminated if it produces a resolvent with a length above this limit. -1 means no limit
    clause_lim: i32,

    #[arg(long = "sub-lim", default_value_t = 1000, group = "simp",value_parser = clap::value_parser!(i32).range(-1..))]
    #[validate(range(min = -1))]
    /// Do not check if subsumption against a clause larger than this. -1 means no limit.
    subsumption_lim: i32,

    #[arg(long = "simp-gc-frac", default_value_t = 0.5, group = "simp")]
    #[validate(range(exclusive_min = 0.0))]
    /// The fraction of wasted memory allowed before a garbage collection is triggered during simplification.
    simp_garbage_frac: f64,

    // MAIN
    #[arg(long = "verb", default_value_t = 0, group = "main")]
    #[validate(range(min = 0, max = 2))]
    /// Verbosity level (0=silent, 1=some, 2=more).
    verb: i32,

    #[arg(long = "pre", default_value_t = true, group = "main")]
    /// Completely turn on/off any preprocessing.
    pre: bool,
    #[arg(long = "solve", default_value_t = true, group = "main")]
    /// Completely turn on/off solving after preprocessing.
    solve: bool,

    // #[arg(long = "dimacs")]
    // /// If given, stop after preprocessing and write the result to this file.
    // dimacs: Option<String>,
    #[arg(long = "cpu-lim", default_value_t = 0, group = "main")]
    #[validate(range(min = 0))]
    /// Limit on CPU time allowed in seconds.
    cpu_lim: 32,

    #[arg(long = "mem-lim", default_value_t = 0, group = "main",value_parser = clap::value_parser!(u32).range(0..))]
    /// Limit on memory usage in megabytes.
    mem_lim: u32,

    #[arg(long = "strictp", default_value_t = false, group = "main")]
    /// Validate DIMACS header during parsing.
    strictp: bool,
}

enum Writer {
    File(File),
    Stdout(io::Stdout),
}

impl Write for Writer {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match self {
            Writer::File(file) => file.write(buf),
            Writer::Stdout(stdout) => stdout.write(buf),
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        match self {
            Writer::File(file) => file.flush(),
            Writer::Stdout(stdout) => stdout.flush(),
        }
    }
}

struct Stat {
    pub parsed_time: Option<Duration>,
    pub simplified_time: Option<Duration>,
    pub solve_time: Option<Duration>,
    pub run_time: Instant,
    pub total_time: ProcessTime,
    least_time: ProcessTime,
    pub printed: bool,
}

impl Drop for Stat {
    fn drop(&mut self) {
        if self.print() {
            eprintln!("c Interrupted");
        }
    }
}

impl Stat {
    pub fn new() -> Self {
        return Self {
            run_time: Instant::now(),
            total_time: ProcessTime::now(),
            least_time: ProcessTime::now(),
            printed: false,
            parsed_time: Default::default(),
            simplified_time: Default::default(),
            solve_time: Default::default(),
        };
    }
    pub fn start_log(&mut self) {
        self.total_time = ProcessTime::now();
        self.least_time = ProcessTime::now();
    }
    pub fn parsed(&mut self) {
        self.parsed_time = Some(self.least_time.elapsed());
        self.least_time = ProcessTime::now();
    }
    pub fn simplified(&mut self) {
        self.simplified_time = Some(self.least_time.elapsed());
        self.least_time = ProcessTime::now();
    }
    pub fn solved(&mut self) {
        self.solve_time = Some(self.least_time.elapsed());
        self.least_time = ProcessTime::now();
    }

    pub fn print(&mut self) -> bool {
        if self.printed {
            return false;
        }
        self.parsed_time.map(|v| {
            eprintln!("c Parse time:           {:?}", v);
        });
        self.simplified_time.map(|v| {
            eprintln!("c Simplification time:  {:?}", v);
        });
        self.solve_time.map(|v| {
            eprintln!("c Solve time:           {:?}", v);
        });
        eprintln!("c Total time:           {:?}", self.total_time.elapsed());
        eprintln!("c Run time:             {:?}", self.run_time.elapsed());
        get_memory().map(|v| {
            eprintln!(
                "c Memory:               {}",
                human_bytes::human_bytes(v as f64)
            );
        });
        std::io::stdout().flush().unwrap();
        self.printed = true;
        return true;
    }
}

impl Arg {
    fn set_opt(&self) {
        rssat::solver::MinisatSolver::set_opt_var_decay(self.var_decay);
        rssat::solver::MinisatSolver::set_opt_clause_decay(self.clause_decay);
        rssat::solver::MinisatSolver::set_opt_random_var_freq(self.random_var_freq);
        rssat::solver::MinisatSolver::set_opt_random_seed(self.random_seed);
        rssat::solver::MinisatSolver::set_opt_ccmin_mode(self.ccmin_mode);
        rssat::solver::MinisatSolver::set_opt_phase_saving(self.phase_saving);
        rssat::solver::MinisatSolver::set_opt_rnd_init_act(self.rnd_init_act);
        rssat::solver::MinisatSolver::set_opt_luby_restart(self.luby_restart);
        rssat::solver::MinisatSolver::set_opt_restart_first(self.restart_first);
        rssat::solver::MinisatSolver::set_opt_restart_inc(self.restart_inc);
        rssat::solver::MinisatSolver::set_opt_garbage_frac(self.garbage_frac);
        rssat::solver::MinisatSolver::set_opt_min_learnts_lim(self.min_learnts_lim);
        rssat::solver::MinisatSolver::set_opt_use_asymm(self.use_asymm);
        rssat::solver::MinisatSolver::set_opt_use_rcheck(self.use_rcheck);
        rssat::solver::MinisatSolver::set_opt_use_elim(self.use_elim);
        rssat::solver::MinisatSolver::set_opt_grow(self.grow);
        rssat::solver::MinisatSolver::set_opt_clause_lim(self.clause_lim);
        rssat::solver::MinisatSolver::set_opt_subsumption_lim(self.subsumption_lim);
        rssat::solver::MinisatSolver::set_opt_simp_garbage_frac(self.simp_garbage_frac);
        rssat::solver::MinisatSolver::set_opt_verbosity(self.verb);
    }

    pub fn run(&self) -> anyhow::Result<i32> {
        let stat = Arc::new(Mutex::new(Stat::new()));
        let mut output = if let Some(path) = &self.output {
            Writer::File(File::create(path)?)
        } else {
            Writer::Stdout(io::stdout())
        };
        self.set_opt();
        let cloned_stat = stat.clone();
        ctrlc::set_handler(move || {
            if let Ok(mut stat) = cloned_stat.lock() {
                if stat.print() {
                    println!("c Interrupted");
                }
            }
        })?;
        let mut solver = rssat::solver::MinisatSolver::new();
        utils::limit_time(self.cpu_lim);
        utils::limit_memory(self.mem_lim);
        if !self.pre {
            solver.eliminate(true);
        }
        stat.lock().unwrap().start_log();
        rssat::parser::read_dimacs_from_file(self.input.as_ref(), self.strictp, &mut solver)
            .unwrap();
        stat.lock().unwrap().parsed();
        solver.eliminate(true);
        stat.lock().unwrap().simplified();
        if !solver.okay() {
            stat.lock().unwrap().print();
            println!("UNSATISFIABLE");
            writeln!(output, "UNSAT")?;

            return Ok(20);
        }
        let mut ret = Default::default();
        if self.solve {
            ret = solver.solve_limited(&[], true, false);
        }
        stat.lock().unwrap().solved();
        stat.lock().unwrap().print();
        match ret {
            solver::RawStatus::Satisfiable => {
                println!("c SATISFIABLE");
                writeln!(output, "SAT")?;
                (0..solver.vars()).map(|v| v + 1).try_for_each(|v| {
                    if solver.model_value(v) {
                        write!(output, "{} ", v)
                    } else {
                        write!(output, "-{} ", v)
                    }
                })?;
                writeln!(output, "0")?;
                return Ok(0);
            }
            solver::RawStatus::Unsatisfiable => {
                println!("c UNSATISFIABLE");
                writeln!(output, "UNSAT")?;
                return Ok(20);
            }
            solver::RawStatus::Unknown => {
                println!("c UNKNOWN");
                writeln!(output, "UNKNOWN")?;
                return Ok(30);
            }
        }
    }
}
