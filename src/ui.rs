use lowcharts::plot::{self, MatchBarRow};
use std::{collections::HashMap, time::Instant};
use surf::StatusCode;
use terminal_emoji::Emoji;

const REFRESH_TIME: u128 = 300;

pub struct UI {
    ttfb_points: Vec<f64>,
    total_time_points: Vec<f64>,
    errors: HashMap<String, usize>,
    status_map: HashMap<StatusCode, usize>,
    is_done: bool,
    last_updated: Instant,
    total_duration: u64,
}

pub fn new() -> UI {
    return UI {
        ttfb_points: Vec::new(),
        total_time_points: Vec::new(),
        errors: HashMap::new(),
        status_map: HashMap::new(),
        is_done: false,
        last_updated: Instant::now(),
        total_duration: 0,
    };
}

impl UI {
    pub fn start(&self) {
        self.print_stats();
    }

    pub fn add_point(&mut self, ttfb: f64, total_time: f64, status: StatusCode) {
        self.ttfb_points.push(ttfb);
        self.total_time_points.push(total_time);

        if !self.status_map.contains_key(&status) {
            self.status_map.insert(status, 0);
        }
        let count = self.status_map.get(&status).unwrap();
        self.status_map.insert(status, count + 1);

        if self.last_updated.elapsed().as_millis() > REFRESH_TIME {
            self.print_stats();
            self.last_updated = Instant::now();
        }
    }

    pub fn add_error(&mut self, error: String) {
        if !self.errors.contains_key(&error) {
            self.errors.insert(error.clone(), 0);
        }
        let count = self.errors.get(&error).unwrap();
        self.errors.insert(error, count + 1);
    }

    pub fn done(&mut self, duration_secs: u64) {
        self.is_done = true;
        self.total_duration = duration_secs;
        self.print_stats();
    }

    fn render_histogram(&self, title: String, vect: Vec<f64>) {
        println!("=== {} (ms) ===", title);
        let options = plot::HistogramOptions {
            intervals: 10,
            ..Default::default()
        };
        let histogram = plot::Histogram::new(&vect, options);
        println!("{}", histogram);
    }

    fn render_map(&self, title: String, rows: Vec<MatchBarRow>) {
        println!();
        println!("=== {} ===", title);
        let match_bar = plot::MatchBar::new(rows);
        println!("{}", match_bar);
    }

    fn print_stats(&self) {
        clearscreen::clear().expect("Error clearing screen");
        let test_emoji = Emoji::new("🧪", "<>");
        println!("=== {} MiniLoad {} ===", test_emoji, test_emoji);
        println!();

        let wait_emoji = Emoji::new("⏱️", "<>");
        let done_emoji = Emoji::new("✅", "<>");

        if self.is_done {
            println!(
                "{} Done! < {} successfull reqs/sec >",
                done_emoji,
                (self.ttfb_points.len() as u64 / self.total_duration)
            );
        } else {
            println!(
                "{} Waiting... ({} requests completed)",
                wait_emoji,
                self.ttfb_points.len() + self.errors.len()
            );
        }

        if self.ttfb_points.len() > 0 {
            self.render_histogram("TTFB".to_owned(), self.ttfb_points.clone());
        }

        if self.total_time_points.len() > 0 {
            self.render_histogram("Total Time".to_owned(), self.total_time_points.clone());
        }

        if self.status_map.len() > 0 {
            let mut status_rows = Vec::new();
            for (status, count) in &self.status_map {
                status_rows.push(MatchBarRow {
                    label: status.to_string(),
                    count: *count,
                });
            }
            self.render_map("Status Codes".to_owned(), status_rows);
        }

        if self.errors.len() > 0 {
            let mut error_rows = Vec::new();
            for error in &self.errors {
                error_rows.push(MatchBarRow {
                    label: error.0.to_string(),
                    count: *error.1,
                });
            }
            self.render_map("Errors".to_owned(), error_rows);
        };
    }
}
