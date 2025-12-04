use crate::{framebuffer::screen::SCREEN, println};

pub fn test_println() {
    for _ in 0..200 {
        println!("test_println_many output");
    }
}

pub fn test_screen() {
    let mut buf: [[u32; SRC_W]; SRC_H] = [[0; SRC_W]; SRC_H];

    // solid white
    fill_solid(&mut buf, 0x00FFFFFF);
    {
        let mut screen = SCREEN.lock();
        screen.write_buffer(&buf);
    }

    // vertical bars
    fill_vertical_bars(&mut buf, 8);
    {
        let mut screen = SCREEN.lock();
        screen.write_buffer(&buf);
    }

    // gradient
    fill_horizontal_gradient(&mut buf, 0x00FF0000, 0x000000FF);
    {
        let mut screen = SCREEN.lock();
        screen.write_buffer(&buf);
    }

    // checkerboard
    fill_checkerboard(&mut buf, 16, 0x00CCCCCC, 0x00333333);
    {
        let mut screen = SCREEN.lock();
        screen.write_buffer(&buf);
    }

    // palette tiles
    fill_palette_tiles(&mut buf, 32, 32);
    {
        let mut screen = SCREEN.lock();
        screen.write_buffer(&buf);
    }

    // border cross
    fill_border_cross(&mut buf, 4, 0x00FF00FF, 0x0000FF00);
    {
        let mut screen = SCREEN.lock();
        screen.write_buffer(&buf);
    }
}

const SRC_W: usize = 256;
const SRC_H: usize = 240;

/// Fill whole buffer with one color.
fn fill_solid(dst: &mut [[u32; SRC_W]; SRC_H], color: u32) {
    for y in 0..SRC_H {
        for x in 0..SRC_W {
            dst[y][x] = color;
        }
    }
}

/// Vertical color bars — good for checking channel ordering and alignment.
/// `n_bars` splits width into n equal bars.
fn fill_vertical_bars(dst: &mut [[u32; SRC_W]; SRC_H], n_bars: usize) {
    // a small palette of distinct colors (RRGGBB)
    let palette: [u32; 8] = [
        0x00FF0000, // red
        0x0000FF00, // green
        0x000000FF, // blue
        0x00FFFF00, // yellow
        0x00FF00FF, // magenta
        0x0000FFFF, // cyan
        0x00FFFFFF, // white
        0x00000000, // black
    ];

    let n = if n_bars == 0 { 1 } else { n_bars };
    for y in 0..SRC_H {
        for x in 0..SRC_W {
            let bar = (x * n) / SRC_W;
            dst[y][x] = palette[bar % palette.len()];
        }
    }
}

/// Horizontal gradient from left (color_a) to right (color_b).
/// Colors are 0x00RRGGBB; we interpolate per channel.
fn fill_horizontal_gradient(dst: &mut [[u32; SRC_W]; SRC_H], color_a: u32, color_b: u32) {
    let ar = ((color_a >> 16) & 0xFF) as i32;
    let ag = ((color_a >> 8) & 0xFF) as i32;
    let ab = (color_a & 0xFF) as i32;

    let br = ((color_b >> 16) & 0xFF) as i32;
    let bg = ((color_b >> 8) & 0xFF) as i32;
    let bb = (color_b & 0xFF) as i32;

    for y in 0..SRC_H {
        for x in 0..SRC_W {
            let t_num = x as i32;
            let t_den = (SRC_W - 1) as i32;
            let r = ar + ((br - ar) * t_num) / t_den;
            let g = ag + ((bg - ag) * t_num) / t_den;
            let b = ab + ((bb - ab) * t_num) / t_den;
            dst[y][x] = ((r as u32) << 16) | ((g as u32) << 8) | (b as u32);
        }
    }
}

/// Checkerboard with tile size (in source pixels)
fn fill_checkerboard(dst: &mut [[u32; SRC_W]; SRC_H], tile: usize, c0: u32, c1: u32) {
    let tile = if tile == 0 { 1 } else { tile };
    for y in 0..SRC_H {
        for x in 0..SRC_W {
            let tx = x / tile;
            let ty = y / tile;
            if ((tx + ty) & 1) == 0 {
                dst[y][x] = c0;
            } else {
                dst[y][x] = c1;
            }
        }
    }
}

/// Border + crosshair to test edges and alignment
fn fill_border_cross(
    dst: &mut [[u32; SRC_W]; SRC_H],
    border_thickness: usize,
    color: u32,
    cross_color: u32,
) {
    let bt = if border_thickness == 0 {
        1
    } else {
        border_thickness
    };
    let mid_x = SRC_W / 2;
    let mid_y = SRC_H / 2;
    for y in 0..SRC_H {
        for x in 0..SRC_W {
            let in_border = x < bt || x >= SRC_W - bt || y < bt || y >= SRC_H - bt;
            if in_border {
                dst[y][x] = color;
            } else if x == mid_x || y == mid_y {
                dst[y][x] = cross_color;
            } else {
                dst[y][x] = 0x00000000; // black inside
            }
        }
    }
}

/// Palette test: repeats palette blocks across the screen for visual verification
fn fill_palette_tiles(dst: &mut [[u32; SRC_W]; SRC_H], tile_w: usize, tile_h: usize) {
    let palette: [u32; 16] = [
        0x00000000, 0x00FFFFFF, 0x00FF0000, 0x0000FF00, 0x000000FF, 0x00FFFF00, 0x00FF00FF,
        0x0000FFFF, 0x00800000, 0x00008000, 0x00000080, 0x00808000, 0x00800080, 0x00008080,
        0x00404040, 0x00C0C0C0,
    ];

    let tw = if tile_w == 0 { 16 } else { tile_w };
    let th = if tile_h == 0 { 16 } else { tile_h };

    for y in 0..SRC_H {
        for x in 0..SRC_W {
            let tx = (x / tw) % 4;
            let ty = (y / th) % 4;
            let idx = tx + ty * 4; // 0..15
            dst[y][x] = palette[idx];
        }
    }
}

/// Moving box animation: draws a colored box that moves across the screen.
/// Call repeatedly with increasing frame index to animate.
///
/// box_w/box_h are box size in source pixels.
fn fill_moving_box(
    dst: &mut [[u32; SRC_W]; SRC_H],
    frame: usize,
    box_w: usize,
    box_h: usize,
    color: u32,
    bg: u32,
) {
    // clear to bg
    for y in 0..SRC_H {
        for x in 0..SRC_W {
            dst[y][x] = bg;
        }
    }

    let path_w = SRC_W.saturating_sub(box_w + 2);
    let path_h = SRC_H.saturating_sub(box_h + 2);

    // position along a simple Lissajous-like path using frame
    let px = (frame % (path_w.max(1))) as usize + 1;
    let py = ((frame / (path_w.max(1))).wrapping_mul(3) % (path_h.max(1))) as usize + 1;

    for yo in 0..box_h {
        for xo in 0..box_w {
            let x = px + xo;
            let y = py + yo;
            if x < SRC_W && y < SRC_H {
                dst[y][x] = color;
            }
        }
    }
}
