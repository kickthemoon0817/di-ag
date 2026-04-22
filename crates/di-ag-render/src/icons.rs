//! Built-in icon library for node decorations.
//!
//! Each icon is a short SVG fragment (circles/paths/etc.) designed to render
//! inside a 20x20 viewbox. The fragments use `currentColor` for stroke/fill so
//! the renderer can set the color via the wrapping `<g>` element. We expand
//! `currentColor` at render time to a concrete theme color — the web editor's
//! SVG sanitizer does not allow the `style` attribute, so we can't rely on
//! the CSS `color` inheritance.

/// Single source of truth for a built-in icon: name, human-readable
/// description, and the raw SVG fragment. Aliases are simply two entries
/// with the same `svg` but different `name` + `description`.
pub struct IconEntry {
    pub name: &'static str,
    pub description: &'static str,
    pub svg: &'static str,
}

/// All built-in icons. `ICON_NAMES` and the `icon_svg` / `icon_description`
/// helpers are derived from this single table so they cannot drift.
pub const ICONS: &[IconEntry] = &[
    IconEntry {
        name: "user",
        description: "Person / actor / human user",
        svg: r##"<circle cx="10" cy="7" r="3" fill="none" stroke="currentColor" stroke-width="1.5"/><path d="M3 17 Q3 12 10 12 Q17 12 17 17" fill="none" stroke="currentColor" stroke-width="1.5"/>"##,
    },
    IconEntry {
        name: "database",
        description: "SQL or NoSQL database",
        svg: r##"<ellipse cx="10" cy="5" rx="6" ry="2" fill="none" stroke="currentColor" stroke-width="1.5"/><path d="M4 5 V15 Q4 17 10 17 Q16 17 16 15 V5" fill="none" stroke="currentColor" stroke-width="1.5"/><path d="M4 10 Q4 12 10 12 Q16 12 16 10" fill="none" stroke="currentColor" stroke-width="1.5"/>"##,
    },
    IconEntry {
        name: "server",
        description: "Physical or virtual server / rack",
        svg: r##"<rect x="3" y="3" width="14" height="4" rx="1" fill="none" stroke="currentColor" stroke-width="1.5"/><rect x="3" y="9" width="14" height="4" rx="1" fill="none" stroke="currentColor" stroke-width="1.5"/><circle cx="6" cy="5" r="0.5" fill="currentColor"/><circle cx="6" cy="11" r="0.5" fill="currentColor"/>"##,
    },
    IconEntry {
        name: "cloud",
        description: "Cloud service / SaaS",
        svg: r##"<path d="M6 14 Q3 14 3 11 Q3 8 6 8 Q7 5 10 5 Q13 5 14 8 Q17 8 17 11 Q17 14 14 14 Z" fill="none" stroke="currentColor" stroke-width="1.5"/>"##,
    },
    IconEntry {
        name: "api",
        description: "API endpoint / service interface",
        svg: r##"<path d="M5 6 L8 10 L5 14 M15 6 L12 10 L15 14 M10 4 L10 16" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/>"##,
    },
    IconEntry {
        name: "web",
        description: "Web browser / web app",
        svg: r##"<circle cx="10" cy="10" r="7" fill="none" stroke="currentColor" stroke-width="1.5"/><ellipse cx="10" cy="10" rx="3" ry="7" fill="none" stroke="currentColor" stroke-width="1.5"/><path d="M3 10 H17" fill="none" stroke="currentColor" stroke-width="1.5"/>"##,
    },
    IconEntry {
        name: "mobile",
        description: "Mobile phone / native app",
        svg: r##"<rect x="6" y="2" width="8" height="16" rx="1.5" fill="none" stroke="currentColor" stroke-width="1.5"/><circle cx="10" cy="15" r="0.7" fill="currentColor"/>"##,
    },
    IconEntry {
        name: "cache",
        description: "In-memory cache (Redis, Memcached)",
        svg: r##"<path d="M4 6 Q4 3 10 3 Q16 3 16 6 V14 Q16 17 10 17 Q4 17 4 14 Z" fill="none" stroke="currentColor" stroke-width="1.5"/><path d="M4 6 Q4 9 10 9 Q16 9 16 6" fill="none" stroke="currentColor" stroke-width="1.5"/>"##,
    },
    IconEntry {
        name: "queue",
        description: "Message queue / job queue",
        svg: r##"<rect x="3" y="7" width="3" height="6" fill="none" stroke="currentColor" stroke-width="1.5"/><rect x="8" y="7" width="3" height="6" fill="none" stroke="currentColor" stroke-width="1.5"/><rect x="13" y="7" width="3" height="6" fill="none" stroke="currentColor" stroke-width="1.5"/>"##,
    },
    // `auth` and `lock` share the same SVG fragment (aliases) but have
    // distinct descriptions so CLI help and the editor picker can explain
    // both terms.
    IconEntry {
        name: "auth",
        description: "Authentication service",
        svg: r##"<rect x="4" y="9" width="12" height="9" rx="1" fill="none" stroke="currentColor" stroke-width="1.5"/><path d="M7 9 V6 Q7 3 10 3 Q13 3 13 6 V9" fill="none" stroke="currentColor" stroke-width="1.5"/><circle cx="10" cy="13" r="1.2" fill="currentColor"/>"##,
    },
    IconEntry {
        name: "lock",
        description: "Security / access control (alias of auth)",
        svg: r##"<rect x="4" y="9" width="12" height="9" rx="1" fill="none" stroke="currentColor" stroke-width="1.5"/><path d="M7 9 V6 Q7 3 10 3 Q13 3 13 6 V9" fill="none" stroke="currentColor" stroke-width="1.5"/><circle cx="10" cy="13" r="1.2" fill="currentColor"/>"##,
    },
    IconEntry {
        name: "storage",
        description: "Block storage / object storage",
        svg: r##"<path d="M3 5 L10 2 L17 5 V15 L10 18 L3 15 Z" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linejoin="round"/><path d="M3 5 L10 8 L17 5 M10 8 V18" fill="none" stroke="currentColor" stroke-width="1.5"/>"##,
    },
    // `gear` and `settings` share the same SVG fragment (aliases).
    IconEntry {
        name: "gear",
        description: "Configuration / settings / service",
        svg: r##"<circle cx="10" cy="10" r="2.5" fill="none" stroke="currentColor" stroke-width="1.5"/><path d="M10 3 V5 M10 15 V17 M3 10 H5 M15 10 H17 M5 5 L6.5 6.5 M13.5 13.5 L15 15 M5 15 L6.5 13.5 M13.5 6.5 L15 5" fill="none" stroke="currentColor" stroke-width="1.5"/>"##,
    },
    IconEntry {
        name: "settings",
        description: "Configuration (alias of gear)",
        svg: r##"<circle cx="10" cy="10" r="2.5" fill="none" stroke="currentColor" stroke-width="1.5"/><path d="M10 3 V5 M10 15 V17 M3 10 H5 M15 10 H17 M5 5 L6.5 6.5 M13.5 13.5 L15 15 M5 15 L6.5 13.5 M13.5 6.5 L15 5" fill="none" stroke="currentColor" stroke-width="1.5"/>"##,
    },
    IconEntry {
        name: "file",
        description: "Document / file",
        svg: r##"<path d="M6 3 H12 L15 6 V17 Q15 18 14 18 H6 Q5 18 5 17 V4 Q5 3 6 3 Z" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linejoin="round"/><path d="M12 3 V6 H15" fill="none" stroke="currentColor" stroke-width="1.5"/>"##,
    },
    IconEntry {
        name: "chart",
        description: "Analytics / reporting",
        svg: r##"<path d="M3 17 H17 M5 17 V11 M9 17 V7 M13 17 V13 M15 17 V9" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>"##,
    },
    IconEntry {
        name: "mail",
        description: "Email / notification service",
        svg: r##"<rect x="3" y="5" width="14" height="10" rx="1" fill="none" stroke="currentColor" stroke-width="1.5"/><path d="M3 6 L10 11 L17 6" fill="none" stroke="currentColor" stroke-width="1.5"/>"##,
    },
];

/// List of all built-in icon names. Hand-maintained but kept in step with
/// `ICONS` via the `icon_names_match_icons_table` unit test.
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
    ICONS.iter().find(|e| e.name == name).map(|e| e.svg)
}

/// Return the short human-readable description for a built-in icon, or
/// `None` if the name is not recognized.
pub fn icon_description(name: &str) -> Option<&'static str> {
    ICONS.iter().find(|e| e.name == name).map(|e| e.description)
}

/// Return the icon SVG with `currentColor` replaced by the given color, ready
/// to be embedded inside a `<g transform="...">…</g>`. Returns `None` if the
/// icon name is unknown.
///
/// `color` is validated against a safe CSS color allowlist (hex #RGB/#RRGGBB/
/// #RRGGBBAA or a well-known CSS color keyword). Unsafe input falls back to
/// `currentColor` so attribute-injection via icon color cannot escape the
/// surrounding `<g fill="...">` context in an exported SVG.
pub fn icon_svg_colored(name: &str, color: &str) -> Option<String> {
    let safe = if is_safe_css_color(color) {
        color
    } else {
        "currentColor"
    };
    icon_svg(name).map(|raw| raw.replace("currentColor", safe))
}

/// True if the input is a safe CSS color literal for injection into an SVG
/// attribute value. Accepts `#RGB`, `#RRGGBB`, `#RRGGBBAA`, the literal
/// `currentColor`, and a small set of CSS named colors commonly used in
/// diagrams. Anything else (including arbitrary identifiers, `rgb(...)`,
/// or strings with quotes/spaces) is rejected.
fn is_safe_css_color(s: &str) -> bool {
    if s == "currentColor" || s == "none" || s == "transparent" {
        return true;
    }
    if let Some(rest) = s.strip_prefix('#') {
        let len = rest.len();
        if (len == 3 || len == 4 || len == 6 || len == 8)
            && rest.chars().all(|c| c.is_ascii_hexdigit())
        {
            return true;
        }
        return false;
    }
    // A short allowlist of common CSS named colors. The renderer default
    // palette already falls in this set; anything outside it is rejected
    // rather than risk attribute injection in exported SVGs.
    matches!(
        s,
        "black"
            | "white"
            | "red"
            | "green"
            | "blue"
            | "yellow"
            | "cyan"
            | "magenta"
            | "gray"
            | "grey"
            | "silver"
            | "orange"
            | "purple"
            | "pink"
            | "brown"
            | "navy"
            | "teal"
            | "olive"
            | "maroon"
            | "lime"
            | "aqua"
            | "fuchsia"
    )
}
