use clap::{App, Arg};
use std::fs::OpenOptions;
use std::io::{BufWriter, Write};
use std::sync::atomic::{AtomicBool, Ordering, Ordering::SeqCst};
use std::sync::Arc;

/// great-appender
fn main() {
    let args = App::new("great-appender")
        .version("1.0")
        .author("Tim Marinin <mt@marinintim.com>")
        .arg(
            Arg::with_name("file")
                .short("f")
                .long("file")
                .help("file to append message to")
                .takes_value(true)
                .required(true),
        )
        .arg(
            Arg::with_name("message")
                .short("m")
                .long("message")
                .help("message to append")
                .takes_value(true)
                .default_value("message"),
        )
        .arg(
            Arg::with_name("verbose")
                .short("v")
                .long("verbose")
                .help("print")
                .takes_value(false),
        )
        .arg(
            Arg::with_name("per_write")
                .short("p")
                .long("per_write")
                .takes_value(true)
                .default_value("1024"),
        )
        .about("append some message to file repeatedly")
        .get_matches();

    let file = OpenOptions::new()
        .append(true)
        .create(true)
        .open(args.value_of("file").expect("File should be specified"))
        .expect("Could not open nor create file");
    let repeated_str = args
        .value_of("message")
        .expect("Message should be specified")
        .to_string();
    let per_write: usize = args
        .value_of("per_write")
        .expect("per_write should be specified")
        .parse()
        .expect("Per_write should be numeric");
    let should_stop = Arc::new(AtomicBool::new(false));
    let verbose = args.is_present("verbose");
    let should_stop_printing = should_stop.clone();
    let should_stop_writing = should_stop.clone();

    signal_hook::flag::register(signal_hook::SIGINT, Arc::clone(&should_stop))
        .expect("Could not register SIGINT handler");

    let (rx, tx) = std::sync::mpsc::channel();

    let writing_thread = std::thread::spawn(move || {
        let mut buf = Vec::new();
        fill_big_buffer(&mut buf, &repeated_str, per_write);

        let mut bytes = 0;
        let mut writer = file;

        while !should_stop_writing.load(Ordering::SeqCst) {
            bytes += writer.write(&buf).expect("Could not write");
            rx.send(bytes)
                .expect("Could not send bytes count to channel");
        }

    });

    let printing_thread = std::thread::spawn(move || {
        let mut measurer = ThroughputMeasurer::new();

        while !should_stop_printing.load(SeqCst) {
            let bytes = tx.recv().expect("Could not get bytes written");
            measurer.measure(bytes as u64);
            if verbose {
                measurer.print_throughput();
            }

        }
        measurer.print_throughput();
        print!("\n");
        std::io::stdout().flush().expect("Could not flush stdout");
    });

    writing_thread
        .join()
        .expect("Could not join the writing thread");
    printing_thread
        .join()
        .expect("Could not join the printing thread");
    
}

struct ThroughputMeasurer {
    started: std::time::Instant,
    bytes: u64,
    last_measure_throughput: u64,
    average_throughput: u64,
    prev_moment: std::time::Instant
}

impl ThroughputMeasurer {
    fn new() -> ThroughputMeasurer {
        ThroughputMeasurer {
            started: std::time::Instant::now(),
            bytes: 0,
            prev_moment: std::time::Instant::now(),
            average_throughput: 0,
            last_measure_throughput: 0
        }
    }

    fn measure(&mut self, bytes: u64) {
        let now = std::time::Instant::now();
        let elapsed = now.duration_since(self.prev_moment).as_secs();
        let elapsed_from_start = now.duration_since(self.started).as_secs();

        let written_bytes = bytes - self.bytes;
        self.last_measure_throughput = (written_bytes as f64 / elapsed as f64) as u64;
        self.average_throughput = (bytes as f64 / elapsed_from_start as f64) as u64;

        self.prev_moment = now;
        self.bytes = bytes;
    }

    fn print_throughput(&self) {
        print!("\u{001b}[2K\u{001b}[1000D");
        print!("Written {}", bytefmt::format(self.bytes));
        print!("\t");
        print!("{}/s", bytefmt::format(self.last_measure_throughput));
        print!("\t");
        print!("{}/s", bytefmt::format(self.average_throughput));
        std::io::stdout()
            .flush()
            .expect("Could not flush to stdout");
    }
}

fn fill_big_buffer(buf: &mut Vec<u8>, s: &str, per_write: usize) {
    let mut writer = BufWriter::new(buf);
    let mut bytes_in_buf = 0;
    while bytes_in_buf < per_write {
        bytes_in_buf += writer.write(s.as_bytes()).unwrap();
        bytes_in_buf += writer.write(&[0x0A]).unwrap();
    }
}
