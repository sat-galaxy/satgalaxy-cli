use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};

use crate::{
    core::{Stat, Writer,parse_path, SmartPath, SmartReader}, utils::{self}
};
use clap::Args;
use satgalaxy::{
    parser::read_dimacs_from_reader,
    solver::{self, GlucoseSolver},
};
use std::io::Write;
use validator::Validate;

#[derive(Args, Validate)]
pub struct Arg {
    /// Input source: local file (.cnf, .xz, .tar.gz), URL, default for stdin
    #[arg(value_name = "INPUT",value_parser = parse_path)]
    input: Option<SmartPath>,
    #[arg(value_name = "OUTPUT")]
    output: Option<PathBuf>,
    #[arg(long = "K", default_value_t = 0.8, group = "core")]
    #[validate(range(
        exclusive_min = 0.0,
        exclusive_max = 1.0,
        message = "K must be in (0, 1)"
    ))]
    /// The constant used to force restart
    k: f64,

    #[arg(long = "R", default_value_t = 1.4, group = "core")]
    #[validate(range(
        exclusive_min = 0.0,
        exclusive_max = 5.0,
        message = "R must be in (0, 5)"
    ))]
    /// The constant used to block restart
    r: f64,

    #[arg(long = "szLBDQueue", default_value_t = 50, group = "core")]
    #[validate(range(min = 10, message = "Size of LBD queue must be at least 10"))]
    /// The size of moving average for LBD (restarts)
    size_lbd_queue: i32,

    #[arg(long = "szTrailQueue", default_value_t = 5000, group = "core")]
    #[validate(range(min = 10, message = "Size of trail queue must be at least 10"))]
    /// The size of moving average for trail (block restarts)
    size_trail_queue: i32,

    #[arg(long = "firstReduceDB", default_value_t = 2000, group = "core")]
    #[validate(range(min = 0, message = "First reduce DB must be a non-negative integer"))]
    /// The number of conflicts before the first reduce DB (or the size of learnts if chanseok is used)
    first_reduce_db: i32,

    #[arg(long = "incReduceDB", default_value_t = 300, group = "core")]
    #[validate(range(
        min = 0,
        message = "Increment for reduce DB must be a non-negative integer"
    ))]
    /// Increment for reduce DB
    inc_reduce_db: i32,

    #[arg(long = "specialIncReduceDB", default_value_t = 1000, group = "core")]
    #[validate(range(
        min = 0,
        message = "Special increment for reduce DB must be a non-negative integer"
    ))]
    /// Special increment for reduce DB
    spec_inc_reduce_db: i32,

    #[arg(long = "minLBDFrozenClause", default_value_t = 30, group = "core")]
    #[validate(range(
        min = 0,
        message = "Minimum LBD for frozen clause must be a non-negative integer"
    ))]
    /// Protect clauses if their LBD decrease and is lower than (for one turn)
    lb_lbd_frozen_clause: i32,

    #[arg(long = "chanseok", num_args(0..=1),default_value_t = false, group = "core")]
    /// Use Chanseok Oh strategy for LBD (keep all LBD<=co and remove half of firstreduceDB other learnt clauses)
    chanseok_hack: bool,

    #[arg(long = "co", default_value_t = 5, group = "core")]
    #[validate(range(
        min = 2,
        message = "Chanseok limit must be a positive integer greater than 1"
    ))]
    /// Chanseok Oh: all learnt clauses with LBD<=co are permanent
    chanseok_limit: i32,

    #[arg(long = "minSizeMinimizingClause", default_value_t = 30, group = "core")]
    #[validate(range(
        min = 3,
        message = "Minimum size for minimizing clause must be at least 3"
    ))]
    /// The min size required to minimize clause
    lb_size_minimzing_clause: i32,

    #[arg(long = "minLBDMinimizingClause", default_value_t = 6, group = "core")]
    #[validate(range(
        min = 3,
        message = "Minimum LBD for minimizing clause must be at least 3"
    ))]
    /// The min LBD required to minimize clause
    lb_lbd_minimzing_clause: i32,

    #[arg(long = "lcm", num_args(0..=1),default_value_t = true, group = "core")]
    /// Use inprocessing vivif (ijcai17 paper)
    lcm: bool,

    #[arg(long = "lcm-update",num_args(0..=1), default_value_t = false, group = "core")]
    /// Updates LBD when doing LCM
    lcm_update_lbd: bool,

    #[arg(long = "var-decay", default_value_t = 0.8, group = "core")]
    #[validate(range(
        exclusive_min = 0.0,
        exclusive_max = 1.0,
        message = "Variable activity decay factor must be in (0, 1)"
    ))]
    /// The variable activity decay factor (starting point)
    var_decay: f64,

    #[arg(long = "max-var-decay", default_value_t = 0.95, group = "core")]
    #[validate(range(
        exclusive_min = 0.0,
        exclusive_max = 1.0,
        message = "Maximum variable activity decay factor must be in (0, 1)"
    ))]
    /// The maximum variable activity decay factor
    max_var_decay: f64,

    #[arg(long = "cla-decay", default_value_t = 0.999, group = "core")]
    #[validate(range(
        exclusive_min = 0.0,
        exclusive_max = 1.0,
        message = "Clause activity decay factor must be in (0, 1)"
    ))]
    /// The clause activity decay factor
    clause_decay: f64,

    #[arg(long = "rnd-freq", default_value_t = 0.0, group = "core")]
    #[validate(range(
        min = 0.0,
        max = 1.0,
        message = "Random variable frequency must be in [0, 1]"
    ))]
    /// The frequency with which the decision heuristic tries to choose a random variable
    random_var_freq: f64,

    #[arg(long = "rnd-seed", default_value_t = 91648253.0, group = "core")]
    #[validate(range(exclusive_min = 0.0, message = "Random seed must be positive"))]
    /// Used by the random variable selection
    random_seed: f64,

    #[arg(long = "ccmin-mode", default_value_t = 2, group = "core")]
    #[validate(range(
        min = 0,
        max = 2,
        message = "Conflict clause minimization mode must be 0, 1, or 2"
    ))]
    /// Controls conflict clause minimization (0=none, 1=basic, 2=deep)
    ccmin_mode: i32,

    #[arg(long = "phase-saving", default_value_t = 2, group = "core")]
    #[validate(range(min = 0, max = 2, message = "Phase saving mode must be 0, 1, or 2"))]
    /// Controls phase saving (0=none, 1=basic, 2=deep)
    phase_saving: i32,

    #[arg(long = "rnd-init",num_args(0..=1), default_value_t = false, group = "core")]
    /// Randomize the initial activity
    rnd_init_act: bool,

    #[arg(long = "gc-frac", default_value_t = 0.2, group = "core")]
    #[validate(range(
        exclusive_min = 0.0,
        message = "Garbage collection fraction must be positive"
    ))]
    /// The fraction of wasted memory allowed before a garbage collection is triggered
    garbage_frac: f64,

    #[arg(long = "gr", num_args(0..=1),default_value_t = true, group = "core")]
    /// glucose strategy to fire clause database reduction (must be false to fire Chanseok strategy)
    glu_reduction: bool,

    #[arg(long = "luby",num_args(0..=1), default_value_t = false, group = "core")]
    /// Use the Luby restart sequence
    luby_restart: bool,

    #[arg(long = "rinc", default_value_t = 2.0, group = "core")]
    #[validate(range(
        min = 1.0,
        message = "Restart interval increase factor must be at least 1.0"
    ))]
    /// Restart interval increase factor
    restart_inc: f64,

    #[arg(long = "luby-factor", default_value_t = 100, group = "core")]
    #[validate(range(min = 1, message = "Luby restart factor must be a positive integer"))]
    /// Luby restart factor
    luby_restart_factor: i32,

    #[arg(long = "phase-restart", default_value_t = 0, group = "core")]
    #[validate(range(
        min = 0,
        max = 2,
        message = "Phase restart factor must be 0, 1, 2, or 3"
    ))]
    /// The amount of randomization for the phase at each restart (0=none, 1=first branch, 2=first branch (no bad clauses), 3=first branch (only initial clauses))
    randomize_phase_on_restarts: i32,

    #[arg(long = "fix-phas-rest",num_args(0..=1), default_value_t = false, group = "core")]
    /// Fixes the first 7 levels at random phase
    fixed_randomize_phase_on_restarts: bool,

    #[arg(long = "adapt",num_args(0..=1), default_value_t = true, group = "core")]
    /// Adapt dynamically stategies after 100000 conflicts
    adapt: bool,

    #[arg(long = "forceunsat",num_args(0..=1), default_value_t = false, group = "core")]
    /// Force the phase for UNSAT
    forceunsat: bool,

    #[arg(long = "asymm",num_args(0..=1), default_value_t = false, group = "core")]
    /// Shrink clauses by asymmetric branching
    use_asymm: bool,

    #[arg(long = "rcheck",num_args(0..=1), default_value_t = false, group = "core")]
    /// Check if a clause is already implied. (costly)
    use_rcheck: bool,

    #[arg(long = "elim",num_args(0..=1), default_value_t = true, group = "core")]
    /// Perform variable elimination.
    use_elim: bool,

    #[arg(long = "grow", default_value_t = 0, group = "core")]
    /// Allow a variable elimination step to grow by a number of clauses.
    grow: i32,

    #[arg(long = "cl-lim", default_value_t = 20, group = "core")]
    #[validate(range(min = -1))]
    /// Variables are not eliminated if it produces a resolvent with a length above this limit. -1 means no limit
    clause_lim: i32,

    #[arg(long = "sub-lim", default_value_t = 1000, group = "core")]
    #[validate(range(min = -1, message = "Subsumption limit must be -1 or a positive integer"))]
    /// Do not check if subsumption against a clause larger than this. -1 means no limit.
    subsumption_lim: i32,

    #[arg(long = "simp-gc-frac", default_value_t = 0.5, group = "core")]
    #[validate(range(
        exclusive_min = 0.0,
        message = "Simplification garbage collection fraction must be positive"
    ))]
    /// The fraction of wasted memory allowed before a garbage collection is triggered during simplification.
    simp_garbage_frac: f64,

    // MAIN
    #[arg(long = "verb", default_value_t = 0, group = "main")]
    #[validate(range(min = 0, max = 2, message = "Verbosity level must be 0, 1, or 2"))]
    /// Verbosity level (0=silent, 1=some, 2=more).
    verb: i32,

    #[arg(long = "pre",num_args(0..=1), default_value_t = true, group = "main")]
    /// Completely turn on/off any preprocessing.
    pre: bool,
    #[arg(long = "solve",num_args(0..=1), default_value_t = true, group = "main")]
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

    #[arg(long = "strictp", num_args(0..=1),default_value_t = false, group = "main")]
    /// Validate DIMACS header during parsing.
    strictp: bool,
}

impl Arg {
    fn set_opt(&self) {
        GlucoseSolver::set_opt_k(self.k);

        GlucoseSolver::set_opt_r(self.r);

        GlucoseSolver::set_opt_size_lbd_queue(self.size_lbd_queue);

        GlucoseSolver::set_opt_size_trail_queue(self.size_trail_queue);

        GlucoseSolver::set_opt_first_reduce_db(self.first_reduce_db);

        GlucoseSolver::set_opt_inc_reduce_db(self.inc_reduce_db);

        GlucoseSolver::set_opt_spec_inc_reduce_db(self.spec_inc_reduce_db);

        GlucoseSolver::set_opt_lb_lbd_frozen_clause(self.lb_lbd_frozen_clause);

        GlucoseSolver::set_opt_chanseok_hack(self.chanseok_hack);

        GlucoseSolver::set_opt_chanseok_limit(self.chanseok_limit);

        GlucoseSolver::set_opt_lb_size_minimzing_clause(self.lb_size_minimzing_clause);

        GlucoseSolver::set_opt_lb_lbd_minimzing_clause(self.lb_lbd_minimzing_clause);

        GlucoseSolver::set_opt_lcm(self.lcm);

        GlucoseSolver::set_opt_lcm_update_lbd(self.lcm_update_lbd);

        GlucoseSolver::set_opt_var_decay(self.var_decay);

        GlucoseSolver::set_opt_max_var_decay(self.max_var_decay);

        GlucoseSolver::set_opt_clause_decay(self.clause_decay);

        GlucoseSolver::set_opt_random_var_freq(self.random_var_freq);

        GlucoseSolver::set_opt_random_seed(self.random_seed);

        GlucoseSolver::set_opt_ccmin_mode(self.ccmin_mode);

        GlucoseSolver::set_opt_phase_saving(self.phase_saving);

        GlucoseSolver::set_opt_rnd_init_act(self.rnd_init_act);

        GlucoseSolver::set_opt_garbage_frac(self.garbage_frac);

        GlucoseSolver::set_opt_glu_reduction(self.glu_reduction);

        GlucoseSolver::set_opt_luby_restart(self.luby_restart);

        GlucoseSolver::set_opt_restart_inc(self.restart_inc);

        GlucoseSolver::set_opt_luby_restart_factor(self.luby_restart_factor);

        GlucoseSolver::set_opt_randomize_phase_on_restarts(self.randomize_phase_on_restarts);

        GlucoseSolver::set_opt_fixed_randomize_phase_on_restarts(
            self.fixed_randomize_phase_on_restarts,
        );

        GlucoseSolver::set_opt_adapt(self.adapt);

        GlucoseSolver::set_opt_forceunsat(self.forceunsat);

        GlucoseSolver::set_opt_use_asymm(self.use_asymm);

        GlucoseSolver::set_opt_use_rcheck(self.use_rcheck);

        GlucoseSolver::set_opt_use_elim(self.use_elim);

        GlucoseSolver::set_opt_grow(self.grow);

        GlucoseSolver::set_opt_clause_lim(self.clause_lim);

        GlucoseSolver::set_opt_subsumption_lim(self.subsumption_lim);

        GlucoseSolver::set_opt_simp_garbage_frac(self.simp_garbage_frac);

        GlucoseSolver::set_opt_verbosity(self.verb);
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
        let mut solver = GlucoseSolver::new();
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
        let reader:SmartReader= self.input.as_ref().try_into()?;
        read_dimacs_from_reader(reader, self.strictp, &mut solver)?;
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
