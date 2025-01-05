use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

use base64::{engine::general_purpose::STANDARD, Engine as _};
use clap::Parser;
use linemux::MuxedLines;
use log::{debug, error, info, warn};
use reqwest::header::HeaderMap;
use serde_json;

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

fn is_valid_json(json_str: &str) -> bool {
    match serde_json::from_str::<serde_json::Value>(json_str) {
        Ok(_) => true,
        Err(_) => false,
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    let args = Args::parse();

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

    // Encode credentials
    let creds = format!("{}:{}", args.ob_username, args.ob_password);
    let base64encoded_creds = STANDARD.encode(creds);
    let authorization = format!("Basic {}", base64encoded_creds);

    // Create headers to authorize
    let mut headers = HeaderMap::new();
    headers.insert("Content-Type", "application/json".parse().unwrap());
    headers.insert("Authorization", authorization.parse().unwrap());

    // Read lines and send them to an API
    while let Ok(Some(line)) = lines.next_line().await {
        // let source = line.source().display();
        let json_row = line.line().to_string();

        debug!("a new line: {}", json_row);

        // Check emptiness
        if json_row.is_empty() {
            warn!("A line is empty");
            continue;
        }

        // Validate JSON
        if !is_valid_json(json_row.as_str()) {
            warn!("A line is not valid JSON");
            continue;
        }

        // Construct the endpoint of OpenObserve
        let openobserve_url = format!(
            "{}/api/{}/{}/_json",
            args.ob_url, args.ob_org, args.ob_stream
        );

        match client
            .post(openobserve_url)
            .body(json_row)
            .headers(headers.clone())
            .send()
            .await
        {
            Ok(response) => {
                let status = response.status();
                let resp = response.text().await?;

                if status != 200 {
                    warn!("Something went wrong:");
                    warn!("Status code: {:#?}", status);
                    warn!("{:#?}", resp);
                }

                info!("Success response:");
                info!("{:#?}", resp);
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
