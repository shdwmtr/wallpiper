use std::collections::VecDeque;
use std::time::{Duration, Instant};

pub const WIDTH: u16 = 210;
pub const HEIGHT: u16 = 120;
pub const REDRAW_INTERVAL: Duration = Duration::from_millis(250);
pub const STATS_WINDOW: Duration = Duration::from_secs(3);
pub const SPARKLINE_SAMPLES: usize = 40;

type Rgba = [u8; 4];

const BG: Rgba = [20, 20, 24, 200];
const PRIMARY: Rgba = [255, 255, 255, 255];
const SECONDARY: Rgba = [150, 150, 150, 255];
const BAR_OK: Rgba = [130, 190, 255, 255];
const BAR_WARN: Rgba = [255, 90, 70, 255];

fn put_pixel(
    pixels: &mut [u8],
    stride: usize,
    buf_w: i32,
    buf_h: i32,
    x: i32,
    y: i32,
    color: Rgba,
) {
    if x < 0 || y < 0 || x >= buf_w || y >= buf_h {
        return;
    }
    let offset = y as usize * stride + x as usize * 4;
    pixels[offset] = color[2];
    pixels[offset + 1] = color[1];
    pixels[offset + 2] = color[0];
    pixels[offset + 3] = color[3];
}

#[allow(clippy::too_many_arguments)]
fn fill_rect(
    pixels: &mut [u8],
    stride: usize,
    buf_w: i32,
    buf_h: i32,
    x: i32,
    y: i32,
    w: i32,
    h: i32,
    color: Rgba,
) {
    for row in y..y + h {
        for col in x..x + w {
            put_pixel(pixels, stride, buf_w, buf_h, col, row, color);
        }
    }
}

fn fill_bg(pixels: &mut [u8], stride: usize, buf_w: usize, buf_h: usize) {
    fill_rect(
        pixels,
        stride,
        buf_w as i32,
        buf_h as i32,
        0,
        0,
        buf_w as i32,
        buf_h as i32,
        BG,
    );
}

// Standard 7-segment encoding, bit0=a(top) 1=b(top-right) 2=c(bottom-right) 3=d(bottom)
// 4=e(bottom-left) 5=f(top-left) 6=g(middle).
const SEGMENTS: [u8; 10] = [
    0b0111111, // 0
    0b0000110, // 1
    0b1011011, // 2
    0b1001111, // 3
    0b1100110, // 4
    0b1101101, // 5
    0b1111101, // 6
    0b0000111, // 7
    0b1111111, // 8
    0b1101111, // 9
];

#[allow(clippy::too_many_arguments)]
fn draw_digit(
    pixels: &mut [u8],
    stride: usize,
    buf_w: i32,
    buf_h: i32,
    x: i32,
    y: i32,
    digit: u8,
    cell_w: i32,
    cell_h: i32,
    thick: i32,
    color: Rgba,
) {
    let bits = SEGMENTS[(digit % 10) as usize];
    let half = cell_h / 2;
    if bits & 0b0000001 != 0 {
        fill_rect(pixels, stride, buf_w, buf_h, x, y, cell_w, thick, color);
        // a
    }
    if bits & 0b0000010 != 0 {
        fill_rect(
            pixels,
            stride,
            buf_w,
            buf_h,
            x + cell_w - thick,
            y,
            thick,
            half,
            color,
        ); // b
    }
    if bits & 0b0000100 != 0 {
        fill_rect(
            pixels,
            stride,
            buf_w,
            buf_h,
            x + cell_w - thick,
            y + half,
            thick,
            half,
            color,
        ); // c
    }
    if bits & 0b0001000 != 0 {
        fill_rect(
            pixels,
            stride,
            buf_w,
            buf_h,
            x,
            y + cell_h - thick,
            cell_w,
            thick,
            color,
        ); // d
    }
    if bits & 0b0010000 != 0 {
        fill_rect(
            pixels,
            stride,
            buf_w,
            buf_h,
            x,
            y + half,
            thick,
            half,
            color,
        ); // e
    }
    if bits & 0b0100000 != 0 {
        fill_rect(pixels, stride, buf_w, buf_h, x, y, thick, half, color); // f
    }
    if bits & 0b1000000 != 0 {
        fill_rect(
            pixels,
            stride,
            buf_w,
            buf_h,
            x,
            y + half - thick / 2,
            cell_w,
            thick,
            color,
        ); // g
    }
}

#[allow(clippy::too_many_arguments)]
fn draw_number(
    pixels: &mut [u8],
    stride: usize,
    buf_w: i32,
    buf_h: i32,
    x: i32,
    y: i32,
    value: u32,
    cell_w: i32,
    cell_h: i32,
    thick: i32,
    color: Rgba,
) -> i32 {
    let value = value.min(999);
    let digits: Vec<u8> = if value == 0 {
        vec![0]
    } else {
        let mut v = value;
        let mut ds = Vec::new();
        while v > 0 {
            ds.push((v % 10) as u8);
            v /= 10;
        }
        ds.reverse();
        ds
    };
    let gap = thick;
    let mut cursor = x;
    for d in digits {
        draw_digit(
            pixels, stride, buf_w, buf_h, cursor, y, d, cell_w, cell_h, thick, color,
        );
        cursor += cell_w + gap;
    }
    cursor - x
}

// 5x5 bitmap font, only the glyphs this overlay actually uses. Bit4 = leftmost column.
fn glyph_5x5(ch: char) -> Option<[u8; 5]> {
    match ch {
        'D' => Some([0b11100, 0b10010, 0b10001, 0b10010, 0b11100]),
        'C' => Some([0b01111, 0b10000, 0b10000, 0b10000, 0b01111]),
        'F' => Some([0b11111, 0b10000, 0b11110, 0b10000, 0b10000]),
        _ => None,
    }
}

#[allow(clippy::too_many_arguments)]
fn draw_letter(
    pixels: &mut [u8],
    stride: usize,
    buf_w: i32,
    buf_h: i32,
    x: i32,
    y: i32,
    ch: char,
    scale: i32,
    color: Rgba,
) {
    let Some(rows) = glyph_5x5(ch) else {
        return;
    };
    for (row_idx, row_bits) in rows.iter().enumerate() {
        for col_idx in 0..5 {
            if row_bits & (1 << (4 - col_idx)) != 0 {
                fill_rect(
                    pixels,
                    stride,
                    buf_w,
                    buf_h,
                    x + col_idx * scale,
                    y + row_idx as i32 * scale,
                    scale,
                    scale,
                    color,
                );
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn draw_row(
    pixels: &mut [u8],
    stride: usize,
    buf_w: i32,
    buf_h: i32,
    x: i32,
    y: i32,
    label: char,
    primary: u32,
    secondary: Option<u32>,
) {
    draw_letter(pixels, stride, buf_w, buf_h, x, y + 2, label, 3, PRIMARY);
    let number_x = x + 22;
    let consumed = draw_number(
        pixels, stride, buf_w, buf_h, number_x, y, primary, 12, 20, 3, PRIMARY,
    );
    if let Some(secondary) = secondary {
        draw_number(
            pixels,
            stride,
            buf_w,
            buf_h,
            number_x + consumed + 10,
            y + 7,
            secondary,
            8,
            13,
            2,
            SECONDARY,
        );
    }
}

#[allow(clippy::too_many_arguments)]
fn draw_sparkline(
    pixels: &mut [u8],
    stride: usize,
    buf_w: i32,
    buf_h: i32,
    x: i32,
    y: i32,
    w: usize,
    h: i32,
    values: &[f64],
    warn_threshold_ms: f64,
) {
    const MAX_SCALE_MS: f64 = 40.0;

    let slot_w = ((w / SPARKLINE_SAMPLES).max(1)) as i32;
    for (i, &delta_ms) in values.iter().enumerate() {
        let bar_h = ((delta_ms / MAX_SCALE_MS).clamp(0.02, 1.0) * h as f64) as i32;
        let color = if delta_ms > warn_threshold_ms {
            BAR_WARN
        } else {
            BAR_OK
        };
        let bar_x = x + i as i32 * slot_w;
        fill_rect(
            pixels,
            stride,
            buf_w,
            buf_h,
            bar_x,
            y + h - bar_h,
            (slot_w - 1).max(1),
            bar_h,
            color,
        );
    }
}

#[derive(Default)]
pub struct FrameStats {
    display_times: VecDeque<Instant>,
    capture_times: VecDeque<Instant>,
}

impl FrameStats {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record_display(&mut self) {
        record(&mut self.display_times);
    }

    pub fn record_capture(&mut self) {
        record(&mut self.capture_times);
    }
}

fn record(times: &mut VecDeque<Instant>) {
    let now = Instant::now();
    times.push_back(now);
    while let Some(&front) = times.front() {
        if now.duration_since(front) > STATS_WINDOW {
            times.pop_front();
        } else {
            break;
        }
    }
}

#[derive(Default)]
pub struct DebugThrottle {
    last_draw: Option<Instant>,
}

impl DebugThrottle {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn reset(&mut self) {
        self.last_draw = None;
    }

    pub fn should_redraw(&mut self) -> bool {
        let now = Instant::now();
        if let Some(last) = self.last_draw {
            if now.duration_since(last) < REDRAW_INTERVAL {
                return false;
            }
        }
        self.last_draw = Some(now);
        true
    }
}

fn fps_in_last_second(times: &VecDeque<Instant>) -> usize {
    let now = Instant::now();
    times
        .iter()
        .filter(|&&t| now.duration_since(t) <= Duration::from_secs(1))
        .count()
}

pub fn render_stats_panel(stats: &FrameStats) -> Vec<u8> {
    let display_fps = fps_in_last_second(&stats.display_times);
    let capture_fps = fps_in_last_second(&stats.capture_times);

    let mut deltas_ms: Vec<f64> = Vec::new();
    for pair in stats.display_times.iter().collect::<Vec<_>>().windows(2) {
        let dt = pair[1].duration_since(*pair[0]).as_secs_f64() * 1000.0;
        deltas_ms.push(dt);
    }
    let last_ms = deltas_ms.last().copied().unwrap_or(0.0);
    let peak_ms = deltas_ms.iter().copied().fold(0.0_f64, f64::max);
    let sparkline: Vec<f64> = deltas_ms
        .iter()
        .rev()
        .take(SPARKLINE_SAMPLES)
        .rev()
        .copied()
        .collect();

    let width = WIDTH as usize;
    let height = HEIGHT as usize;
    let stride = width * 4;
    let mut pixels = vec![0u8; stride * height];
    let (bw, bh) = (width as i32, height as i32);

    fill_bg(&mut pixels, stride, width, height);
    draw_row(
        &mut pixels,
        stride,
        bw,
        bh,
        10,
        8,
        'D',
        display_fps as u32,
        None,
    );
    draw_row(
        &mut pixels,
        stride,
        bw,
        bh,
        10,
        40,
        'C',
        capture_fps as u32,
        None,
    );
    draw_row(
        &mut pixels,
        stride,
        bw,
        bh,
        10,
        72,
        'F',
        last_ms.round() as u32,
        Some(peak_ms.round() as u32),
    );
    draw_sparkline(
        &mut pixels,
        stride,
        bw,
        bh,
        10,
        100,
        width - 20,
        14,
        &sparkline,
        33.3,
    );

    pixels
}
