use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

use base64::prelude::*;
use clap::Parser;
use http::StatusCode;
use linemux::MuxedLines;
use log::{debug, error, info, warn};
use reqwest::header::HeaderMap;
use serde_json::Value;

#[derive(Parser, Debug)]
#[command(version)]
struct Args {
    /// Name of the person to greet
    #[arg(long)]
    source_file: String,

    /// URL for OpenObserve
    #[arg(long)]
    ob_url: String,

    /// API username for OpenObserve
    #[arg(long)]
    ob_username: String,

    /// API password for OpenObserve
    #[arg(long)]
    ob_password: String,

    /// OpenObserve: organization
    #[arg(long)]
    ob_org: String,

    /// OpenObserve: stream
    #[arg(long)]
    ob_stream: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    let args = Args::parse();

    debug!("Arguments:");
    debug!("source_file: {}", args.source_file);
    debug!("ob_url: {}", args.ob_url);
    debug!("ob_username: {}", args.ob_username);
    debug!("ob_password: {}", args.ob_password);
    debug!("ob_org: {}", args.ob_org);
    debug!("ob_stream: {}", args.ob_stream);

    let mut lines = MuxedLines::new()?;

    // Register log sources
    let file = File::open(args.source_file)?;
    let reader = BufReader::new(file);

    for line in reader.lines() {
        let source = line?;
        let source_filename = source.clone();

        lines.add_file(source).await?;

        info!("Registered source: {}", source_filename);
    }

    let client = reqwest::Client::new();

    // Construct the endpoint of OpenObserve
    let openobserve_ingest_url = format!(
        "{}/api/{}/{}/_json",
        args.ob_url, args.ob_org, args.ob_stream
    );

    // Encode credentials
    let creds = format!("{}:{}", args.ob_username, args.ob_password);
    let creds_encoded = BASE64_STANDARD.encode(creds);
    let authorization = format!("Basic {}", creds_encoded);

    // Create headers to authorize
    let mut headers = HeaderMap::new();
    headers.insert("Content-Type", "application/json".parse().unwrap());
    headers.insert("Authorization", authorization.parse().unwrap());

    // Read lines and send them to an API
    while let Ok(Some(line)) = lines.next_line().await {
        let row = line.line().to_string();

        debug!("A new line: {}", row);

        // Check emptiness
        if row.is_empty() {
            warn!("A line is empty");
            continue;
        }

        // Validate JSON
        let ok = serde_json::from_str::<Value>(row.as_str()).is_ok();
        if !ok {
            warn!("A line is not valid JSON");
            continue;
        }

        match client
            .post(openobserve_ingest_url.clone())
            .body(row)
            .headers(headers.clone())
            .send()
            .await
        {
            Ok(r) => {
                let status = r.status();
                let text = r.text().await?;

                if status != StatusCode::OK {
                    warn!(
                        "Something went wrong, status code: {:#?}, response:",
                        status
                    );
                } else {
                    info!("Success response:");
                }

                info!("{:#?}", text);
            }
            Err(e) => {
                if e.is_connect() {
                    error!("Connection error: {}", e);
                } else {
                    error!("Request error: {}", e);
                }
            }
        }
    }

    Ok(())
}
