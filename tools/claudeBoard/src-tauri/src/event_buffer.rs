use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

use crate::model::HookEvent;

pub fn append_event(path: &Path, event: &HookEvent) -> std::io::Result<()> {
    let mut file = OpenOptions::new().create(true).append(true).open(path)?;
    writeln!(file, "{}", serde_json::to_string(event).unwrap())?;
    Ok(())
}

pub fn drain_events(path: &Path) -> std::io::Result<Vec<HookEvent>> {
    if !path.exists() {
        return Ok(Vec::new());
    }

    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let events = reader
        .lines()
        .map(|line| serde_json::from_str::<HookEvent>(&line.unwrap()).unwrap())
        .collect::<Vec<_>>();

    std::fs::remove_file(path)?;
    Ok(events)
}
