use std::fs::File;
use std::io::{self, Write};
use std::path::Path;
use std::time::{Duration, Instant};

use cpu_time::ProcessTime;

use crate::utils::get_memory;

pub enum Writer {
    File(File),
    Stdout(io::Stdout),
}

impl<P:AsRef<Path>> From<Option<P>> for Writer {
    fn from(path: Option<P>) -> Self {
        match path {
            Some(p) => Writer::File(File::create(p).unwrap()),
            None => Writer::Stdout(io::stdout()),
        }
    } 
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

pub struct Stat {
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
            println!("c Interrupted");
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
            println!("c Parse time:           {:?}", v);
        });
        self.simplified_time.map(|v| {
            println!("c Simplification time:  {:?}", v);
        });
        self.solve_time.map(|v| {
            println!("c Solve time:           {:?}", v);
        });
        println!("c Total time:           {:?}", self.total_time.elapsed());
        println!("c Run time:             {:?}", self.run_time.elapsed());
        get_memory().map(|v| {
            println!(
                "c Memory:               {}",
                human_bytes::human_bytes(v as f64)
            );
        });
        std::io::stdout().flush().unwrap();
        self.printed = true;
        return true;
    }
}
