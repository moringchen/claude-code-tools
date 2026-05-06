use std::{fs, io, path::Path};

use crate::model::TaskSnapshot;

pub fn load_snapshot(path: &Path) -> io::Result<TaskSnapshot> {
    if !path.exists() {
        return Ok(TaskSnapshot::default());
    }

    let contents = fs::read_to_string(path)?;
    serde_json::from_str(&contents).map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))
}

pub fn save_snapshot(path: &Path, snapshot: &TaskSnapshot) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let contents = serde_json::to_vec_pretty(snapshot)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
    let temp_path = path.with_extension("json.tmp");
    {
        let mut file = fs::File::create(&temp_path)?;
        use std::io::Write;
        file.write_all(&contents)?;
        file.sync_all()?;
    }
    fs::rename(temp_path, path)
}
