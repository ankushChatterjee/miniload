use std::{sync::Arc, time::Instant};

use clap::Parser;
use futures::AsyncReadExt;
use surf::http::Error;
use surf::{Client, StatusCode};
use tokio::sync::mpsc::{self, Sender};
use ui::UI;

mod ui;

#[derive(Parser, Debug)]
#[command(author="Ankush", version="0.1.0", about = None, long_about = None)]
struct Args {
    #[arg(short, long)]
    url: String,
    #[arg(short, long)]
    num_requests: u32,
    #[arg(short, long)]
    concurrency: Option<u32>,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let ui = ui::new();
    run_tasks(
        args.url,
        args.num_requests,
        args.concurrency.unwrap_or(1),
        ui,
    )
    .await;
}

async fn run_tasks(url: String, n: u32, c: u32, mut ui: UI) {
    let arc_url = Arc::new(url);
    let (tx, mut rx) = mpsc::channel::<Result<(u128, u128, StatusCode), Error>>(64);
    let mut num_concurrent = c;
    let mut extra = n % c;
    ui.start();
    while num_concurrent > 0 {
        let c_url = Arc::clone(&arc_url);
        let txc = tx.clone();
        let mut num_reqs = n / c;
        if extra > 0 {
            num_reqs += 1;
            extra -= 1;
        }
        tokio::spawn(async move {
            return get_response_times(c_url, num_reqs, txc).await;
        });
        num_concurrent -= 1;
    }

    let mut received = 0;
    while let Some(rec) = rx.recv().await {
        received += 1;
        match rec {
            Ok(res) => {
                let (ttfb, total_time, status) = res;
                ui.add_point(ttfb as f64, total_time as f64, status);
            }
            Err(err) => {
                ui.add_error(err.to_string());
            }
        }

        if received == n {
            ui.done();
            break;
        }
    }
}

async fn get_response_times(
    url: Arc<String>,
    n: u32,
    tx: Sender<Result<(u128, u128, StatusCode), Error>>,
) {
    let mut num = 0;
    let client = Client::new();
    while num < n {
        let req = client.get(url.to_string()).build();
        let mut buf: [u8; 8192] = [0; 8192];
        let start_time = Instant::now();
        let response = client.send(req).await;
        match response {
            Ok(mut res) => {
                let ttfb = start_time.elapsed();
                let mut i: u32 = 0;
                loop {
                    let len = res.read(&mut buf).await.unwrap();
                    i = i + 1;
                    if len == 0 {
                        break;
                    }
                }
                let total_time = start_time.elapsed();
                tx.send(Ok((ttfb.as_millis(), total_time.as_millis(), res.status())))
                    .await
                    .unwrap();
            }
            Err(err) => {
                tx.send(Err(err)).await.unwrap();
            }
        }
        num += 1;
    }
}
