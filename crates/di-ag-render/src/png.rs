use crate::RenderError;

/// Maximum rendered pixel dimension on either axis. Prevents OOM on pathological
/// inputs (massive viewBox * high DPI). 16384 px matches common GPU limits.
const MAX_PNG_DIMENSION: u32 = 16_384;

/// PNG iTXt chunk keyword used to embed the original DSL source in a rendered
/// PNG so the file can be opened and edited later, like `.drawio.png`.
pub const DIAG_SOURCE_KEYWORD: &str = "di-ag-source";

pub struct PngOptions {
    pub dpi_scale: f64,
    pub transparent: bool,
}

impl Default for PngOptions {
    fn default() -> Self {
        PngOptions {
            dpi_scale: 2.0,
            transparent: false,
        }
    }
}

pub fn svg_to_png(svg_str: &str, options: &PngOptions) -> Result<Vec<u8>, RenderError> {
    svg_to_png_with_source(svg_str, options, None)
}

/// Render an SVG string to PNG bytes, optionally embedding the original `.diag`
/// source as an iTXt chunk. When `source` is `Some(_)`, the produced PNG can be
/// re-opened by di-ag with full fidelity (similar to `.drawio.png`).
pub fn svg_to_png_with_source(
    svg_str: &str,
    options: &PngOptions,
    source: Option<&str>,
) -> Result<Vec<u8>, RenderError> {
    let tree = resvg::usvg::Tree::from_str(svg_str, &resvg::usvg::Options::default())
        .map_err(|e| RenderError::Failed(format!("SVG parse error: {}", e)))?;

    let size = tree.size();
    let scale = options.dpi_scale as f32;
    let width = (size.width() * scale) as u32;
    let height = (size.height() * scale) as u32;

    if width == 0 || height == 0 {
        return Err(RenderError::Failed("SVG has zero dimensions".into()));
    }
    if width > MAX_PNG_DIMENSION || height > MAX_PNG_DIMENSION {
        return Err(RenderError::Failed(format!(
            "Rendered dimensions {}x{} exceed maximum {} — lower --dpi or shrink the diagram",
            width, height, MAX_PNG_DIMENSION
        )));
    }

    let mut pixmap = tiny_skia::Pixmap::new(width, height)
        .ok_or_else(|| RenderError::Failed("Failed to create pixmap".into()))?;

    if !options.transparent {
        pixmap.fill(tiny_skia::Color::WHITE);
    }

    let transform =
        tiny_skia::Transform::from_scale(options.dpi_scale as f32, options.dpi_scale as f32);

    resvg::render(&tree, transform, &mut pixmap.as_mut());

    let png_bytes = pixmap
        .encode_png()
        .map_err(|e| RenderError::Failed(format!("PNG encode error: {}", e)))?;

    match source {
        Some(src) if !src.is_empty() => Ok(embed_itxt(&png_bytes, DIAG_SOURCE_KEYWORD, src)),
        _ => Ok(png_bytes),
    }
}

/// Extract an embedded di-ag source string from a PNG produced by di-ag.
/// Returns `None` if the PNG has no such chunk, or if the bytes are malformed.
pub fn extract_source(png_bytes: &[u8]) -> Option<String> {
    read_itxt(png_bytes, DIAG_SOURCE_KEYWORD)
}

/// Insert an uncompressed iTXt chunk right before the IEND chunk so the PNG
/// carries the keyword/text pair. Overwrites any existing iTXt with the same
/// keyword so round-trips remain idempotent.
fn embed_itxt(png: &[u8], keyword: &str, text: &str) -> Vec<u8> {
    // PNG signature is 8 bytes. After that is a sequence of chunks:
    //   length(4) | type(4) | data(length) | crc(4)
    // We copy chunks through, drop any existing iTXt with the same keyword,
    // then insert our new iTXt immediately before IEND.
    if png.len() < 8 || &png[..8] != b"\x89PNG\r\n\x1a\n" {
        return png.to_vec();
    }

    let new_chunk = build_itxt_chunk(keyword, text);
    let mut out: Vec<u8> = Vec::with_capacity(png.len() + new_chunk.len());
    out.extend_from_slice(&png[..8]);

    let mut i = 8usize;
    while i + 8 <= png.len() {
        let len = u32::from_be_bytes([png[i], png[i + 1], png[i + 2], png[i + 3]]) as usize;
        let type_start = i + 4;
        let type_end = type_start + 4;
        if type_end > png.len() {
            break;
        }
        let ctype = &png[type_start..type_end];
        let data_start = type_end;
        let data_end = data_start + len;
        let crc_end = data_end + 4;
        if crc_end > png.len() {
            break;
        }

        let skip_existing =
            ctype == b"iTXt" && chunk_keyword_matches(&png[data_start..data_end], keyword);
        let is_iend = ctype == b"IEND";

        if is_iend {
            out.extend_from_slice(&new_chunk);
            out.extend_from_slice(&png[i..crc_end]);
            i = crc_end;
            continue;
        }

        if !skip_existing {
            out.extend_from_slice(&png[i..crc_end]);
        }
        i = crc_end;
    }

    out
}

fn read_itxt(png: &[u8], keyword: &str) -> Option<String> {
    if png.len() < 8 || &png[..8] != b"\x89PNG\r\n\x1a\n" {
        return None;
    }
    let mut i = 8usize;
    while i + 8 <= png.len() {
        let len = u32::from_be_bytes([png[i], png[i + 1], png[i + 2], png[i + 3]]) as usize;
        let type_start = i + 4;
        let type_end = type_start + 4;
        if type_end > png.len() {
            return None;
        }
        let ctype = &png[type_start..type_end];
        let data_start = type_end;
        let data_end = data_start + len;
        let crc_end = data_end + 4;
        if crc_end > png.len() {
            return None;
        }
        if ctype == b"iTXt" && chunk_keyword_matches(&png[data_start..data_end], keyword) {
            return parse_itxt_text(&png[data_start..data_end]);
        }
        if ctype == b"IEND" {
            return None;
        }
        i = crc_end;
    }
    None
}

fn chunk_keyword_matches(data: &[u8], keyword: &str) -> bool {
    if let Some(nul) = data.iter().position(|&b| b == 0) {
        &data[..nul] == keyword.as_bytes()
    } else {
        false
    }
}

/// Parse the text body out of an iTXt chunk data payload.
/// Layout: keyword\0 compFlag compMethod language\0 translatedKeyword\0 text
fn parse_itxt_text(data: &[u8]) -> Option<String> {
    let nul1 = data.iter().position(|&b| b == 0)?;
    let after_keyword = nul1 + 1;
    if after_keyword + 2 > data.len() {
        return None;
    }
    let comp_flag = data[after_keyword];
    // Only handle uncompressed for now.
    if comp_flag != 0 {
        return None;
    }
    let mut idx = after_keyword + 2;
    // Language tag terminated by null.
    let lang_end = data[idx..].iter().position(|&b| b == 0)?;
    idx += lang_end + 1;
    // Translated keyword terminated by null.
    let tkw_end = data[idx..].iter().position(|&b| b == 0)?;
    idx += tkw_end + 1;
    std::str::from_utf8(&data[idx..]).ok().map(|s| s.to_string())
}

fn build_itxt_chunk(keyword: &str, text: &str) -> Vec<u8> {
    // Chunk data: keyword\0 compFlag compMethod lang\0 translated\0 text
    let mut data: Vec<u8> = Vec::new();
    data.extend_from_slice(keyword.as_bytes());
    data.push(0);
    data.push(0); // compression flag: uncompressed
    data.push(0); // compression method (ignored when flag=0)
    data.push(0); // empty language tag
    data.push(0); // empty translated keyword
    data.extend_from_slice(text.as_bytes());

    let mut chunk = Vec::with_capacity(12 + data.len());
    chunk.extend_from_slice(&(data.len() as u32).to_be_bytes());
    chunk.extend_from_slice(b"iTXt");
    chunk.extend_from_slice(&data);
    // CRC is over type + data.
    let mut crc_input = Vec::with_capacity(4 + data.len());
    crc_input.extend_from_slice(b"iTXt");
    crc_input.extend_from_slice(&data);
    let crc = crc32_png(&crc_input);
    chunk.extend_from_slice(&crc.to_be_bytes());
    chunk
}

/// Standard PNG CRC-32 (polynomial 0xedb88320, little-endian CRC, reflected).
fn crc32_png(data: &[u8]) -> u32 {
    let mut crc: u32 = 0xffff_ffff;
    for &b in data {
        crc ^= b as u32;
        for _ in 0..8 {
            crc = if crc & 1 != 0 {
                (crc >> 1) ^ 0xedb8_8320
            } else {
                crc >> 1
            };
        }
    }
    !crc
}
