use std::{
    io::{ Write},
    path::PathBuf,
    sync::{Arc, Mutex},
};

use clap::{Args};
use satgalaxy::{parser::read_dimacs_from_file, solver::{self, MinisatSolver}};
use validator::Validate;

use crate::{core::{Stat, Writer}, utils::{self}};

#[derive(Args, Validate)]
pub struct Arg {
    /// Input file
    #[arg(value_name = "INPUT")]
    input: Option<PathBuf>,
    #[arg(value_name = "OUTPUT")]
    output: Option<PathBuf>,
    /// The variable activity decay factor
    #[arg(long, value_name = "VAR_DECAY", default_value_t = 0.95, group = "core")]
    #[validate(range(exclusive_min = 0.0, exclusive_max = 1.0, message = "Variable decay must be in (0, 1)"))]
    var_decay: f64,
    /// The clause activity decay factor
    #[arg(long = "cla-decay", default_value_t = 0.999, group = "core")]
    #[validate(range(exclusive_min = 0.0, exclusive_max = 1.0, message = "Clause decay must be in (0, 1)"))]
    clause_decay: f64,
    #[arg(long = "rnd-freq", default_value_t = 0.0, group = "core")]
    #[validate(range(min = 0.0, max = 1.0))]
    /// The frequency with which the decision heuristic tries to choose a random variable
    random_var_freq: f64,

    #[arg(long = "rnd-seed", default_value_t = 91648253.0, group = "core")]
    #[validate(range(exclusive_min = 0.0, message = "Random seed must be positive"))]
    /// Used by the random variable selection
    random_seed: f64,

    #[arg(long, default_value_t = 2, group = "core")]
    #[validate(range(min = 0, max = 2, message = "Conflict clause minimization mode must be 0, 1, or 2"))]
    /// Controls conflict clause minimization (0=none, 1=basic, 2=deep)
    ccmin_mode: i32,
    #[arg(long, default_value_t = 2, group = "core")]
    #[validate(range(min = 0, max = 2, message = "Phase saving level must be 0, 1, or 2"))]
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
    #[validate(range(min = 1.0, message = "Restart interval increase factor must be at least 1.0"))]
    /// Restart interval increase factor
    restart_inc: f64,
    #[arg(long = "gc-frac", default_value_t = 0.2, group = "core")]
    #[validate(range(exclusive_min = 0.0, message = "Garbage collection fraction must be positive"))]
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

    #[arg(long = "sub-lim", default_value_t = 1000, group = "simp")]
    #[validate(range(min = -1, message = "Subsumption limit must be -1 or a positive integer"))]
    /// Do not check if subsumption against a clause larger than this. -1 means no limit.
    subsumption_lim: i32,

    #[arg(long = "simp-gc-frac", default_value_t = 0.5, group = "simp")]
    #[validate(range(exclusive_min = 0.0, message = "Simplification garbage collection fraction must be positive"))]
    /// The fraction of wasted memory allowed before a garbage collection is triggered during simplification.
    simp_garbage_frac: f64,

    // MAIN
    #[arg(long = "verb", default_value_t = 0, group = "main")]
    #[validate(range(min = 0, max = 2, message = "Verbosity level must be 0, 1, or 2"))]
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
    #[validate(range(min = 0, message = "CPU time limit must be a non-negative integer"))]
    /// Limit on CPU time allowed in seconds.
    cpu_lim: u32,

    #[arg(long = "mem-lim", default_value_t = 0, group = "main")]
    #[validate(range(min = 0, message = "Memory limit must be a non-negative integer"))]
    /// Limit on memory usage in megabytes.
    mem_lim: u32,

    #[arg(long = "strictp", default_value_t = false, group = "main")]
    /// Validate DIMACS header during parsing.
    strictp: bool,
}


impl Arg {
    fn set_opt(&self) {
       MinisatSolver::set_opt_var_decay(self.var_decay);
       MinisatSolver::set_opt_clause_decay(self.clause_decay);
       MinisatSolver::set_opt_random_var_freq(self.random_var_freq);
       MinisatSolver::set_opt_random_seed(self.random_seed);
       MinisatSolver::set_opt_ccmin_mode(self.ccmin_mode);
       MinisatSolver::set_opt_phase_saving(self.phase_saving);
       MinisatSolver::set_opt_rnd_init_act(self.rnd_init_act);
       MinisatSolver::set_opt_luby_restart(self.luby_restart);
       MinisatSolver::set_opt_restart_first(self.restart_first);
       MinisatSolver::set_opt_restart_inc(self.restart_inc);
       MinisatSolver::set_opt_garbage_frac(self.garbage_frac);
       MinisatSolver::set_opt_min_learnts_lim(self.min_learnts_lim);
       MinisatSolver::set_opt_use_asymm(self.use_asymm);
       MinisatSolver::set_opt_use_rcheck(self.use_rcheck);
       MinisatSolver::set_opt_use_elim(self.use_elim);
       MinisatSolver::set_opt_grow(self.grow);
       MinisatSolver::set_opt_clause_lim(self.clause_lim);
       MinisatSolver::set_opt_subsumption_lim(self.subsumption_lim);
       MinisatSolver::set_opt_simp_garbage_frac(self.simp_garbage_frac);
       MinisatSolver::set_opt_verbosity(self.verb);
    }

    pub fn run(&self) -> anyhow::Result<i32> {
        self.validate()?;
        let stat = Arc::new(Mutex::new(Stat::new()));
        let mut output: Writer = self.output.as_ref().into();

        self.set_opt();
        let cloned_stat = stat.clone();
        ctrlc::set_handler(move || {
            if let Ok(mut stat) = cloned_stat.lock() {
                if stat.print() {
                    println!("c Interrupted");
                }
                std::process::exit(30);
            }
        })?;
        let mut solver =MinisatSolver::new();
        if let Err(e) = utils::limit_time(self.cpu_lim as u64) {
            println!("c WARNING: {}", e);
        }
        if let Err(e) = utils::limit_memory(self.mem_lim as u64) {
            println!("c WARNING: {}", e);
        }
        if !self.pre {
            solver.eliminate(true);
        }
        stat.lock().unwrap().start_log();
        read_dimacs_from_file(self.input.as_ref(), self.strictp, &mut solver)?;
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
