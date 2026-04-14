pub fn run(input: &str, output: Option<&str>) -> Result<(), String> {
    if !input.ends_with(".png") {
        return Err(format!("Extract requires a .png file, got '{}'", input));
    }
    let bytes = std::fs::read(input)
        .map_err(|e| format!("Failed to read '{}': {}", input, e))?;
    let source = di_ag_render::extract_png_source(&bytes).ok_or_else(|| {
        format!(
            "'{}' does not contain an embedded di-ag source chunk",
            input
        )
    })?;
    match output {
        Some(path) => {
            write_atomic(path, source.as_bytes())?;
            eprintln!("Wrote {}", path);
        }
        None => print!("{}", source),
    }
    Ok(())
}

/// Atomic file write: write to `<path>.tmp` in the same directory, then
/// rename over the target. Readers either see the old file or the new file,
/// never a truncated one.
fn write_atomic(path: &str, bytes: &[u8]) -> Result<(), String> {
    let tmp = format!("{}.tmp", path);
    std::fs::write(&tmp, bytes)
        .map_err(|e| format!("Failed to write '{}': {}", tmp, e))?;
    std::fs::rename(&tmp, path)
        .map_err(|e| format!("Failed to rename '{}' -> '{}': {}", tmp, path, e))?;
    Ok(())
}
