/// 5x5 block font patterns (█ = on). Used to render live logo_text + kernel
/// so the centered "text in the middle of the screen" is always the actual OS.
fn get_5x5_pattern(ch: char) -> Option<[&'static str; 5]> {
    let u = ch.to_ascii_uppercase();
    match u {
        'A' => Some([" ███ ", "█   █", "█████", "█   █", "█   █"]),
        'B' => Some(["████ ", "█   █", "████ ", "█   █", "████ "]),
        'C' => Some([" ████", "█    ", "█    ", "█    ", " ████"]),
        'D' => Some(["████ ", "█   █", "█   █", "█   █", "████ "]),
        'E' => Some(["█████", "█    ", "████ ", "█    ", "█████"]),
        'F' => Some(["█████", "█    ", "████ ", "█    ", "█    "]),
        'H' => Some(["█   █", "█   █", "█████", "█   █", "█   █"]),
        'I' => Some(["█████", "  █  ", "  █  ", "  █  ", "█████"]),
        'L' => Some(["█    ", "█    ", "█    ", "█    ", "█████"]),
        'M' => Some(["█   █", "██ ██", "█ █ █", "█   █", "█   █"]),
        'N' => Some(["█   █", "██  █", "█ █ █", "█  ██", "█   █"]),
        'O' => Some([" ███ ", "█   █", "█   █", "█   █", " ███ "]),
        'P' => Some(["████ ", "█   █", "████ ", "█    ", "█    "]),
        'R' => Some(["████ ", "█   █", "████ ", "█  █ ", "█   █"]),
        'S' => Some([" ████", "█    ", " ███ ", "    █", "████ "]),
        'T' => Some(["█████", "  █  ", "  █  ", "  █  ", "  █  "]),
        'U' => Some(["█   █", "█   █", "█   █", "█   █", " ███ "]),
        'W' => Some(["█   █", "█   █", "█ █ █", "██ ██", "█   █"]),
        'X' => Some(["█   █", " █ █ ", "  █  ", " █ █ ", "█   █"]),
        '0' => Some([" ███ ", "█  ██", "█ █ █", "██  █", " ███ "]),
        '1' => Some(["  █  ", " ██  ", "  █  ", "  █  ", "█████"]),
        '2' => Some([" ███ ", "█   █", "   █ ", " █   ", "█████"]),
        '3' => Some(["████ ", "    █", " ███ ", "    █", "████ "]),
        '4' => Some(["█   █", "█   █", "█████", "    █", "    █"]),
        '5' => Some(["█████", "█    ", "████ ", "    █", "████ "]),
        '6' => Some([" ███ ", "█    ", "████ ", "█   █", " ███ "]),
        '7' => Some(["█████", "    █", "   █ ", "  █  ", "  █  "]),
        '8' => Some([" ███ ", "█   █", " ███ ", "█   █", " ███ "]),
        '9' => Some([" ███ ", "█   █", " ████", "    █", " ███ "]),
        '_' => Some(["     ", "     ", "     ", "     ", "█████"]),
        '!' => Some(["  █  ", "  █  ", "  █  ", "     ", "  █  "]),
        _ => Some([" ███ ", "█   █", "█   █", "█   █", " ███ "]), // generic rounded box fallback
    }
}

type LogoCacheEntry = (String, Option<String>, Vec<String>);

/// Renders the live centered logo block (logo_text as big block letters
/// using the 5x5 font above + optional sub_text line underneath).
/// Perfect for retro TUI effects and dashboards.
pub fn render_logo_block(text: &str, sub_text: Option<&str>) -> Vec<String> {
    static CACHE: std::sync::Mutex<Option<LogoCacheEntry>> = std::sync::Mutex::new(None);
    let mut lock = CACHE.lock().unwrap();
    if let Some((cached_text, cached_sub, cached_val)) = &*lock {
        if cached_text == text && cached_sub.as_deref() == sub_text {
            return cached_val.clone();
        }
    }

    let val = render_logo_block_uncached(text, sub_text);
    *lock = Some((text.to_string(), sub_text.map(|s| s.to_string()), val.clone()));
    val
}

fn render_logo_block_uncached(text: &str, sub_text: Option<&str>) -> Vec<String> {
    let chars: Vec<char> = text.chars().filter(|c| !c.is_whitespace()).collect();
    if chars.is_empty() {
        let fallback = sub_text.unwrap_or("LINUX").to_string();
        return vec![fallback];
    }

    let ch_h = 5usize;
    let ch_w = 5usize;
    let gap = 2usize; // spacing between letters
    let num = chars.len();
    let logo_w = num * ch_w + (num.saturating_sub(1)) * gap;

    let mut block: Vec<String> = vec![String::new(); ch_h];

    for (ci, &c) in chars.iter().enumerate() {
        let pat = get_5x5_pattern(c).unwrap();
        let x0 = ci * (ch_w + gap);
        for r in 0..ch_h {
            let cur_len = block[r].len();
            if cur_len < x0 {
                block[r].push_str(&" ".repeat(x0 - cur_len));
            }
            block[r].push_str(pat[r]);
            if ci < num - 1 {
                block[r].push_str(&" ".repeat(gap));
            }
        }
    }

    let mut out = block;

    if let Some(k) = sub_text {
        let k = k.trim();
        if !k.is_empty() {
            out.push(String::new()); // visual separator
            let k_len = k.chars().count();
            let pad = if k_len >= logo_w { 0 } else { (logo_w - k_len) / 2 };
            let mut kline = " ".repeat(pad);
            kline.push_str(k);
            out.push(kline);
        }
    }

    out
}

/// Dynamic system information getter.
pub use crate::platform::native::sys_info::get_system_info;
