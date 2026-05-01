use std::net::SocketAddr;
use std::process::Command;
use std::sync::{Arc, Mutex};

use claude_board::{
    scan::parse_scan_row,
    server::{build_router, refresh_scan},
    store::TaskStore,
};
use time::{format_description::well_known::Rfc3339, OffsetDateTime};

fn main() {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    runtime.block_on(async_main());
}

async fn async_main() {
    let store = Arc::new(Mutex::new(TaskStore::default()));
    eprintln!("[claudeBoard] daemon initial_refresh start");
    match refresh_scan(&store, scan_rows, current_timestamp) {
        Ok(()) => eprintln!("[claudeBoard] daemon initial_refresh completed"),
        Err(error) => eprintln!("[claudeBoard] daemon initial_refresh failed: {error}"),
    }
    let app = build_router(store, scan_rows, current_timestamp);
    let addr = SocketAddr::from(([127, 0, 0, 1], 46123));
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

fn current_timestamp() -> String {
    OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .unwrap_or_else(|_| String::from("1970-01-01T00:00:00Z"))
}

fn scan_rows() -> std::io::Result<Vec<String>> {
    eprintln!("[claudeBoard] scan_rows start");
    let output = Command::new("ps")
        .args(["-axo", "pid=,command="])
        .output()?;

    eprintln!("[claudeBoard] scan_rows ps_status={}", output.status);
    if !output.status.success() {
        eprintln!("[claudeBoard] scan_rows ps failed status={}", output.status);
        return Ok(Vec::new());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let raw_line_count = stdout.lines().count();
    let rows = stdout.lines().filter_map(parse_scan_row).collect::<Vec<_>>();
    eprintln!(
        "[claudeBoard] scan_rows completed raw_line_count={} accepted_row_count={}",
        raw_line_count,
        rows.len()
    );

    Ok(rows)
}
