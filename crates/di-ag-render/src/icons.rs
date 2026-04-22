//! Built-in icon library for node decorations.
//!
//! Each icon is a short SVG fragment (circles/paths/etc.) designed to render
//! inside a 20x20 viewbox. The fragments use `currentColor` for stroke/fill so
//! the renderer can set the color via the wrapping `<g>` element. We expand
//! `currentColor` at render time to a concrete theme color — the web editor's
//! SVG sanitizer does not allow the `style` attribute, so we can't rely on
//! the CSS `color` inheritance.

/// List of all built-in icon names. Exposed so other crates (web UI, CLI
/// help, validators) can enumerate the set.
pub const ICON_NAMES: &[&str] = &[
    "user",
    "database",
    "server",
    "cloud",
    "api",
    "web",
    "mobile",
    "cache",
    "queue",
    "auth",
    "lock",
    "storage",
    "gear",
    "settings",
    "file",
    "chart",
    "mail",
];

/// Return the raw SVG fragment for a built-in icon, or `None` if the name is
/// not recognized. The fragment is meant to be wrapped in a `<g>` that
/// positions and colors it; the inner shapes use `currentColor`.
pub fn icon_svg(name: &str) -> Option<&'static str> {
    match name {
        "user" => Some(
            r##"<circle cx="10" cy="7" r="3" fill="none" stroke="currentColor" stroke-width="1.5"/><path d="M3 17 Q3 12 10 12 Q17 12 17 17" fill="none" stroke="currentColor" stroke-width="1.5"/>"##,
        ),
        "database" => Some(
            r##"<ellipse cx="10" cy="5" rx="6" ry="2" fill="none" stroke="currentColor" stroke-width="1.5"/><path d="M4 5 V15 Q4 17 10 17 Q16 17 16 15 V5" fill="none" stroke="currentColor" stroke-width="1.5"/><path d="M4 10 Q4 12 10 12 Q16 12 16 10" fill="none" stroke="currentColor" stroke-width="1.5"/>"##,
        ),
        "server" => Some(
            r##"<rect x="3" y="3" width="14" height="4" rx="1" fill="none" stroke="currentColor" stroke-width="1.5"/><rect x="3" y="9" width="14" height="4" rx="1" fill="none" stroke="currentColor" stroke-width="1.5"/><circle cx="6" cy="5" r="0.5" fill="currentColor"/><circle cx="6" cy="11" r="0.5" fill="currentColor"/>"##,
        ),
        "cloud" => Some(
            r##"<path d="M6 14 Q3 14 3 11 Q3 8 6 8 Q7 5 10 5 Q13 5 14 8 Q17 8 17 11 Q17 14 14 14 Z" fill="none" stroke="currentColor" stroke-width="1.5"/>"##,
        ),
        "api" => Some(
            r##"<path d="M5 6 L8 10 L5 14 M15 6 L12 10 L15 14 M10 4 L10 16" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/>"##,
        ),
        "web" => Some(
            r##"<circle cx="10" cy="10" r="7" fill="none" stroke="currentColor" stroke-width="1.5"/><ellipse cx="10" cy="10" rx="3" ry="7" fill="none" stroke="currentColor" stroke-width="1.5"/><path d="M3 10 H17" fill="none" stroke="currentColor" stroke-width="1.5"/>"##,
        ),
        "mobile" => Some(
            r##"<rect x="6" y="2" width="8" height="16" rx="1.5" fill="none" stroke="currentColor" stroke-width="1.5"/><circle cx="10" cy="15" r="0.7" fill="currentColor"/>"##,
        ),
        "cache" => Some(
            r##"<path d="M4 6 Q4 3 10 3 Q16 3 16 6 V14 Q16 17 10 17 Q4 17 4 14 Z" fill="none" stroke="currentColor" stroke-width="1.5"/><path d="M4 6 Q4 9 10 9 Q16 9 16 6" fill="none" stroke="currentColor" stroke-width="1.5"/>"##,
        ),
        "queue" => Some(
            r##"<rect x="3" y="7" width="3" height="6" fill="none" stroke="currentColor" stroke-width="1.5"/><rect x="8" y="7" width="3" height="6" fill="none" stroke="currentColor" stroke-width="1.5"/><rect x="13" y="7" width="3" height="6" fill="none" stroke="currentColor" stroke-width="1.5"/>"##,
        ),
        "auth" | "lock" => Some(
            r##"<rect x="4" y="9" width="12" height="9" rx="1" fill="none" stroke="currentColor" stroke-width="1.5"/><path d="M7 9 V6 Q7 3 10 3 Q13 3 13 6 V9" fill="none" stroke="currentColor" stroke-width="1.5"/><circle cx="10" cy="13" r="1.2" fill="currentColor"/>"##,
        ),
        "storage" => Some(
            r##"<path d="M3 5 L10 2 L17 5 V15 L10 18 L3 15 Z" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linejoin="round"/><path d="M3 5 L10 8 L17 5 M10 8 V18" fill="none" stroke="currentColor" stroke-width="1.5"/>"##,
        ),
        "gear" | "settings" => Some(
            r##"<circle cx="10" cy="10" r="2.5" fill="none" stroke="currentColor" stroke-width="1.5"/><path d="M10 3 V5 M10 15 V17 M3 10 H5 M15 10 H17 M5 5 L6.5 6.5 M13.5 13.5 L15 15 M5 15 L6.5 13.5 M13.5 6.5 L15 5" fill="none" stroke="currentColor" stroke-width="1.5"/>"##,
        ),
        "file" => Some(
            r##"<path d="M6 3 H12 L15 6 V17 Q15 18 14 18 H6 Q5 18 5 17 V4 Q5 3 6 3 Z" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linejoin="round"/><path d="M12 3 V6 H15" fill="none" stroke="currentColor" stroke-width="1.5"/>"##,
        ),
        "chart" => Some(
            r##"<path d="M3 17 H17 M5 17 V11 M9 17 V7 M13 17 V13 M15 17 V9" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>"##,
        ),
        "mail" => Some(
            r##"<rect x="3" y="5" width="14" height="10" rx="1" fill="none" stroke="currentColor" stroke-width="1.5"/><path d="M3 6 L10 11 L17 6" fill="none" stroke="currentColor" stroke-width="1.5"/>"##,
        ),
        _ => None,
    }
}

/// Return the icon SVG with `currentColor` replaced by the given color, ready
/// to be embedded inside a `<g transform="...">…</g>`. Returns `None` if the
/// icon name is unknown.
pub fn icon_svg_colored(name: &str, color: &str) -> Option<String> {
    icon_svg(name).map(|raw| raw.replace("currentColor", color))
}
