// written by cwa on 12/aug/2025

// this file exposes the function 'fetch_current_utc_datetime'
// which returns the current UTC datetime
// by calculating a fixed-trim median on several (13)
// stratum-1 ntp servers in Canada

use anyhow::{Result, anyhow};
use chrono::{DateTime, Utc};
use futures::{StreamExt, stream::FuturesUnordered};
use rsntp::AsyncSntpClient;
use tokio::time::{Duration, timeout};

// stratum-1 ntp server hosts in Canada
const HOSTS: [&str; 13] = [
    "clock.uregina.ca:123",             // University of Regina (Regina, SK)
    "subitaneous.cpsc.ucalgary.ca:123", // University of Calgary (Calgary, AB)
    "ntp2.torix.ca:123",                // Toronto Internet Exchange ntp2 (Toronto, ON)
    "ntp3.torix.ca:123",                // Toronto Internet Exchange ntp3 (Toronto, ON)
    "ntp1.acorn-ns.ca:123",             // Acorn ntp1 (Halifax, NS)
    "ntp2.acorn-ns.ca:123",             // Acorn ntp2 (Halifax, NS)
    "ntp.nyy.ca:123",                   // Andrew Wright (Saskatoon, SK)
    "ntp.zaf.ca:123",                   // Jeff Fisher (Regina, SK)
    "ntp1.qix.ca:123",                  // Montreal Internet Exchange ntp1 (Montreal, QC)
    "ntp2.qix.ca:123",                  // Montreal Internet Exchange ntp2 (Montreal, QC)
    "tick.usask.ca:123",                // University of Saskatchewan ntp1 (Saskatoon, SK)
    "tock.usask.ca:123",                // University of Saskatchewan ntp2 (Saskatoon, SK)
    "ntp.wetmore.ca:123",               // wetmore.ca (Saint John, NB)
];

// calculate median
fn calculate_median(mut datetimes: Vec<DateTime<Utc>>) -> Option<DateTime<Utc>> {
    if datetimes.is_empty() {
        return None;
    }
    datetimes.sort_unstable();
    Some(datetimes[datetimes.len() / 2])
}

// calculate trimmed median using a fixed cutoff in ms to filter outliers
fn calculate_trimmed_median(
    datetimes: &[DateTime<Utc>],
    cutoff_ms: i64,
    minimum_keep: usize,
) -> Option<DateTime<Utc>> {
    let median = calculate_median(datetimes.to_vec())?;
    let kept_datetimes: Vec<_> = datetimes
        .iter()
        .cloned()
        .filter(|dt| (*dt - median).num_milliseconds().abs() <= cutoff_ms)
        .collect();
    if kept_datetimes.len() < minimum_keep {
        return None;
    }
    calculate_median(kept_datetimes)
}

// query an ntp server
async fn query_ntp_server(host: &str) -> Result<DateTime<Utc>> {
    let response = AsyncSntpClient::new().synchronize(host).await?;
    Ok(response.datetime().into_chrono_datetime()?)
}

// query all ntp servers concurrently with per-host timeout & fixed-trim median
pub async fn fetch_current_utc_datetime(
    per_host_timeout_ms: u64,
    cutoff_ms: i64,
    minimum_keep: usize,
) -> Result<DateTime<Utc>> {
    let mut pending_queries = FuturesUnordered::new();
    for &host in &HOSTS {
        let timeout_duration = Duration::from_millis(per_host_timeout_ms);
        pending_queries.push(async move {
            let response: Result<DateTime<Utc>> =
                match timeout(timeout_duration, query_ntp_server(host)).await {
                    Ok(Ok(dt)) => Ok(dt),
                    Ok(Err(e)) => Err(anyhow!("{host}: {e}")),
                    Err(_) => Err(anyhow!("{host}: TIMEOUT")),
                };
            (host, response)
        });
    }

    let mut successful_queries: Vec<(&str, DateTime<Utc>)> = Vec::new();
    let mut failed_queries: Vec<String> = Vec::new();

    while let Some((host, response)) = pending_queries.next().await {
        match response {
            Ok(dt) => successful_queries.push((host, dt)),
            Err(e) => failed_queries.push(e.to_string()),
        }
    }

    let datetimes: Vec<DateTime<Utc>> = successful_queries.iter().map(|&(_, dt)| dt).collect();
    let current_utc_datetime = calculate_trimmed_median(&datetimes, cutoff_ms, minimum_keep)
        .ok_or_else(|| anyhow!("not enough agreeing servers for consensus"))?;

    Ok(current_utc_datetime)
}
