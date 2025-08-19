pub fn analyse_utc_fetch(
    utc_datetime: DateTime<Utc>,
    successful_queries: &[(&str, DateTime<utc>)],
    failed_queries: &[String],
    cutoff_ms: i64,
    initial_median: DateTime<Utc>,
) {
    // which/how many servers failed include their error message

    // which/how many servers were trimmed include offset from initial_median

    // which host is the median

    // list offset of kept servers from median

    Ok(())
}
