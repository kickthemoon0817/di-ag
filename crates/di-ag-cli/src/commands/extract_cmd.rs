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
            std::fs::write(path, source.as_bytes())
                .map_err(|e| format!("Failed to write '{}': {}", path, e))?;
            eprintln!("Wrote {}", path);
        }
        None => print!("{}", source),
    }
    Ok(())
}
