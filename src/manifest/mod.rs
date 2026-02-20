use std::collections::BTreeMap;
use std::path::Path;

const MANIFEST_FILE: &str = ".manifest";

pub fn read(dst_dir: &Path, module_name: &str) -> Vec<String> {
    let path = dst_dir.join(MANIFEST_FILE);
    let Ok(content) = std::fs::read_to_string(&path) else {
        return Vec::new();
    };
    let Ok(map) = serde_yaml::from_str::<BTreeMap<String, Vec<String>>>(&content) else {
        return Vec::new();
    };
    map.get(module_name).cloned().unwrap_or_default()
}

pub fn update(dst_dir: &Path, module_name: &str, entries: &[String]) -> Result<(), String> {
    let path = dst_dir.join(MANIFEST_FILE);
    let mut map: BTreeMap<String, Vec<String>> = std::fs::read_to_string(&path)
        .ok()
        .and_then(|c| serde_yaml::from_str(&c).ok())
        .unwrap_or_default();

    if entries.is_empty() {
        map.remove(module_name);
    } else {
        map.insert(module_name.to_string(), entries.to_vec());
    }

    if map.is_empty() {
        let _ = std::fs::remove_file(&path);
    } else {
        let yaml = serde_yaml::to_string(&map)
            .map_err(|e| format!("failed to serialize manifest: {e}"))?;
        std::fs::write(&path, yaml)
            .map_err(|e| format!("failed to write {}: {e}", path.display()))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests;
