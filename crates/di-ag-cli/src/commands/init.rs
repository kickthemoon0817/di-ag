pub fn run() -> Result<(), String> {
    let template = r##"@preset flowchart
@theme light
@layout direction=TB spacing=60

# Simple flowchart example
node start "Start" {
    shape: rounded_rect
    style: { fill: "#4CAF50", stroke: "#388E3C" }
}

node process "Process Data" {
    shape: rect
}

node decision "Valid?" {
    shape: diamond
}

node success "Done" {
    shape: rounded_rect
    style: { fill: "#2196F3", stroke: "#1565C0" }
}

node retry "Retry" {
    shape: rect
}

edge start -> process
edge process -> decision
edge decision -> success {
    label: "yes"
}
edge decision -> retry {
    label: "no"
}
edge retry -> process
"##;

    let filename = "diagram.diag";
    if std::path::Path::new(filename).exists() {
        return Err(format!("'{}' already exists", filename));
    }
    std::fs::write(filename, template)
        .map_err(|e| format!("Failed to write '{}': {}", filename, e))?;
    eprintln!("Created {}", filename);
    Ok(())
}
