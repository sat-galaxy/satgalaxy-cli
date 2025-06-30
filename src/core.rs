use std::{
    io::{Read, Stdin, stdin,self, Write},
    path::{PathBuf,Path},
    time::{Duration, Instant},
    fs::File
};

use cpu_time::ProcessTime;

use crate::utils::get_memory;

pub enum Writer {
    File(File),
    Stdout(io::Stdout),
}

impl<P: AsRef<Path>> From<Option<P>> for Writer {
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

#[derive(Debug, Clone, PartialEq)]
pub enum SmartPath {
    FilePath(PathBuf),
    Url(url::Url),
}

pub fn parse_path(s: &str) -> Result<SmartPath, String> {
    url::Url::parse(s).map(SmartPath::Url).or_else(|_| {
        let path = PathBuf::from(s);
        if path.exists() {
            Ok(SmartPath::FilePath(path))
        } else {
            Err(format!("`{s}` is not a valid URL or file path"))
        }
    })
}

pub(crate) enum SmartReader {
    Stdin(Stdin),
    File(File),
    Url(reqwest::blocking::Response),
}

impl Read for SmartReader {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        match self {
            SmartReader::Stdin(reader) => reader.read(buf),
            SmartReader::File(reader) => reader.read(buf),
            SmartReader::Url(reader) => reader.read(buf),
        }
    }
}

impl TryFrom<Option<&SmartPath>> for SmartReader {
    fn try_from(value: Option<&SmartPath>) -> Result<Self, Self::Error> {
        match value {
            Some(SmartPath::FilePath(path)) => File::open(path).map(SmartReader::File),
            Some(SmartPath::Url(url)) => reqwest::blocking::get(url.clone())
                .map(|resp| SmartReader::Url(resp))
                .map_err(|e| io::Error::new(io::ErrorKind::Other, e)),
            None => Ok(SmartReader::Stdin(stdin())),
        }
    }

    type Error = io::Error;
}
