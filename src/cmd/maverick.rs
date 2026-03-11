use std::fmt::Write as _;
use std::io::{self, BufWriter, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use console::Term;

use crate::events::OutputMode;

const GIF_DATA: &[u8] = include_bytes!("../topgun.gif");

// --- Tuning knobs ---
const COLOR_LEVELS: u8 = 8; // quantization steps per channel (fewer = blockier, more color runs)
const COLOR_MIN: u8 = 20; // black floor — values below this become 0
const COLOR_MAX: u8 = 220; // white ceiling — values above this become 255

struct AltScreen;

impl AltScreen {
    fn enter() -> Self {
        eprint!("\x1b[?1049h\x1b[?25l");
        let _ = io::stderr().flush();
        Self
    }
}

impl Drop for AltScreen {
    fn drop(&mut self) {
        eprint!("\x1b[?1049l\x1b[?25h");
        let _ = io::stderr().flush();
    }
}

struct Frame {
    rgba: Vec<u8>, // raw RGBA pixels
    w: u32,
    h: u32,
    delay_ms: u64,
}

pub fn run(mode: OutputMode) {
    if mode != OutputMode::Human {
        return;
    }

    let term = Term::stderr();
    let (th, tw) = (term.size().0 as u32, term.size().1 as u32);

    let frames = match pre_render(tw, th) {
        Ok(f) if !f.is_empty() => f,
        _ => return,
    };

    let stop = Arc::new(AtomicBool::new(false));
    signal_hook::flag::register(signal_hook::consts::SIGINT, Arc::clone(&stop)).ok();

    let _screen = AltScreen::enter();
    let mut out = BufWriter::new(io::stderr());

    for buf in &frames {
        if stop.load(Ordering::Relaxed) {
            break;
        }
        let _ = out.write_all(b"\x1b[H");
        let _ = out.write_all(buf.0.as_bytes());
        let _ = out.flush();
        std::thread::sleep(std::time::Duration::from_millis(buf.1));
    }
}

fn pre_render(tw: u32, th: u32) -> Result<Vec<(String, u64)>, io::Error> {
    let pixel_h = th * 2;
    let frames = decode_gif()?;

    Ok(frames
        .iter()
        .map(|f| {
            let scaled = nearest_scale(&f.rgba, f.w, f.h, tw, pixel_h);
            let sw = tw.min(f.w * pixel_h / f.h); // maintain aspect
            let sh = pixel_h.min(f.h * tw / f.w);
            (render_half_blocks(&scaled, sw, sh), f.delay_ms)
        })
        .collect())
}

fn decode_gif() -> Result<Vec<Frame>, io::Error> {
    let mut decoder = gif::DecodeOptions::new();
    decoder.set_color_output(gif::ColorOutput::RGBA);
    let mut reader = decoder
        .read_info(GIF_DATA)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

    let mut canvas = vec![0u8; reader.width() as usize * reader.height() as usize * 4];
    let gw = reader.width() as u32;
    let gh = reader.height() as u32;
    let mut frames = Vec::new();

    while let Some(frame) = reader
        .read_next_frame()
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?
    {
        let delay_ms = (frame.delay as u64 * 10).max(1); // GIF delay is in centiseconds

        // Blit frame onto canvas (handles disposal/offsets)
        let fx = frame.left as usize;
        let fy = frame.top as usize;
        let fw = frame.width as usize;
        let fh = frame.height as usize;
        for row in 0..fh {
            for col in 0..fw {
                let src = (row * fw + col) * 4;
                let dst = ((fy + row) * gw as usize + (fx + col)) * 4;
                if src + 3 < frame.buffer.len()
                    && dst + 3 < canvas.len()
                    && frame.buffer[src + 3] > 0
                {
                    canvas[dst..dst + 4].copy_from_slice(&frame.buffer[src..src + 4]);
                }
            }
        }

        frames.push(Frame {
            rgba: canvas.clone(),
            w: gw,
            h: gh,
            delay_ms,
        });
    }

    Ok(frames)
}

fn nearest_scale(rgba: &[u8], src_w: u32, src_h: u32, dst_w: u32, dst_h: u32) -> Vec<u8> {
    // Fit within dst, maintain aspect ratio
    let scale_w = dst_w as f64 / src_w as f64;
    let scale_h = dst_h as f64 / src_h as f64;
    let scale = scale_w.min(scale_h);
    let out_w = ((src_w as f64 * scale) as u32).max(1);
    let out_h = ((src_h as f64 * scale) as u32).max(1);

    let mut out = vec![0u8; out_w as usize * out_h as usize * 4];
    for y in 0..out_h {
        let sy = (y as f64 / scale).min((src_h - 1) as f64) as u32;
        for x in 0..out_w {
            let sx = (x as f64 / scale).min((src_w - 1) as f64) as u32;
            let si = (sy * src_w + sx) as usize * 4;
            let di = (y * out_w + x) as usize * 4;
            out[di..di + 4].copy_from_slice(&rgba[si..si + 4]);
        }
    }
    out
}

fn quantize(c: u8) -> u8 {
    if c <= COLOR_MIN {
        return 0;
    }
    if c >= COLOR_MAX {
        return 255;
    }
    let range = (COLOR_MAX - COLOR_MIN) as u16;
    let normed = (c - COLOR_MIN) as u16;
    let step = range / COLOR_LEVELS as u16;
    let q = (normed / step) * step + step / 2 + COLOR_MIN as u16;
    q.min(255) as u8
}

fn render_half_blocks(rgba: &[u8], w: u32, h: u32) -> String {
    let mut buf = String::with_capacity(w as usize * (h as usize / 2) * 20);

    let px = |x: u32, y: u32| -> [u8; 3] {
        if y < h && x < w {
            let i = (y * w + x) as usize * 4;
            [quantize(rgba[i]), quantize(rgba[i + 1]), quantize(rgba[i + 2])]
        } else {
            [0, 0, 0]
        }
    };

    let mut y = 0u32;
    while y < h {
        let mut prev_fg: [u8; 3] = [255, 255, 255];
        let mut prev_bg: [u8; 3] = [255, 255, 255];

        for x in 0..w {
            let fg = px(x, y);
            let bg = px(x, y + 1);

            if fg != prev_fg {
                write!(buf, "\x1b[38;2;{};{};{}m", fg[0], fg[1], fg[2]).ok();
                prev_fg = fg;
            }
            if bg != prev_bg {
                write!(buf, "\x1b[48;2;{};{};{}m", bg[0], bg[1], bg[2]).ok();
                prev_bg = bg;
            }
            buf.push('\u{2580}');
        }
        buf.push_str("\x1b[0m\n");
        y += 2;
    }

    buf
}
