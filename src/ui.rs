use std::{collections::HashMap, time::{Instant, SystemTime}};
use lowcharts::plot::{self, MatchBarRow};
use surf::StatusCode;
use terminal_emoji::Emoji;

const REFRESH_TIME: u128 = 300;

pub struct UI {
    ttfb_points: Vec<f64>,
    total_time_points: Vec<f64>,
    errors: Vec<String>,
    status_map: HashMap<StatusCode, usize>,
    is_done: bool,
    last_updated: Instant
}

pub fn new() -> UI {
    return UI {
        ttfb_points: Vec::new(),
        total_time_points: Vec::new(),
        errors: Vec::new(),
        status_map: HashMap::new(),
        is_done: false,
        last_updated: Instant::now()
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
        self.errors.push(error);
    }

    pub fn done(&mut self) {
        self.is_done = true;
        self.print_stats();
    }

    fn print_stats(&self) {
        clearscreen::clear().expect("Error clearing screen");
        let test_emoji = Emoji::new("🧪", "<>");
        println!("=== {} MiniLoad {} ===",test_emoji, test_emoji);
        println!();

        let wait_emoji = Emoji::new("⏱️", "<>");
        let done_emoji = Emoji::new("✅", "<>");

        if self.is_done {
            println!("{} Done!", done_emoji);
        } else {
            println!("{} Waiting... ({} requests completed)", wait_emoji, self.ttfb_points.len() + self.errors.len());
        }

        if self.ttfb_points.len() > 0 {
            println!("=== TTFB (ms) ===");
            let options = plot::HistogramOptions { intervals: 10, ..Default::default() };
            let histogram = plot::Histogram::new(&self.ttfb_points, options);
            println!("{}", histogram);
        }
        
        if self.total_time_points.len() > 0 {
            println!();
            println!("=== Total Time (ms) ===");
            let options = plot::HistogramOptions { intervals: 10, ..Default::default() };
            let histogram = plot::Histogram::new(&self.total_time_points, options);
            println!("{}", histogram);
        }

        if self.status_map.len() > 0 {
            let mut status_rows = Vec::new();
            for (status, count) in &self.status_map {
                status_rows.push(MatchBarRow {
                    label: status.to_string(),
                    count: *count,
                });
            }
            println!();
            println!("=== Status Codes ===");
            let match_bar = plot::MatchBar::new(status_rows);
            println!("{}", match_bar);
        }

        if self.errors.len() > 0 {
            println!();
            println!("=== Errors ===");
            for error in &self.errors {
                println!("{}", error);
            }
        };
    }
}


