use alloc::boxed::Box;

use crate::{framebuffer::screen::SCREEN, println};

pub fn test_println() {
    for _ in 0..200 {
        println!("test_println_many output");
    }
}

const SRC_W: usize = 256;
const SRC_H: usize = 240;

pub fn test_screen() {
    // 1) allocate uninitialized boxed array on the heap (no big stack temp)
    let mut boxed_uninit = Box::<[[u32; SRC_W]; SRC_H]>::new_uninit();

    // 2) get a pointer to the first row so we can initialize in-place
    //    as *mut [u32; SRC_W] so row-by-row writes are easy
    let row_ptr = boxed_uninit.as_mut_ptr() as *mut [u32; SRC_W];

    // 3) initialize every element in-place (don't create any large stack arrays)
    unsafe {
        for r in 0..SRC_H {
            // pointer to the first u32 of row r
            let row_u32_ptr = (row_ptr.add(r)) as *mut u32;
            for c in 0..SRC_W {
                // initial value doesn't matter much; fill with black (0)
                core::ptr::write(row_u32_ptr.add(c), 0x00000000u32);
            }
        }
    }

    // 4) mark initialized and turn into Box<[[u32;SRC_W];SRC_H]>
    let boxed = unsafe { boxed_uninit.assume_init() };

    // 5) leak the box to obtain a &'static mut reference (keeps it alive for APs)
    let buf_static: &'static mut [[u32; SRC_W]; SRC_H] = Box::leak(boxed);

    // From now on `buf_static` is a &'static mut; we can reuse it with your fill_* helpers.
    {
        // SOLID WHITE
        fill_solid(buf_static, 0x00FFFFFF);
        let mut screen = SCREEN.lock();
        screen.write_buffer(&*buf_static);
    }

    {
        // VERTICAL BARS
        fill_vertical_bars(buf_static, 8);
        let mut screen = SCREEN.lock();
        screen.write_buffer(&*buf_static);
    }

    {
        // HORIZONTAL GRADIENT
        fill_horizontal_gradient(buf_static, 0x00FF0000, 0x000000FF);
        let mut screen = SCREEN.lock();
        screen.write_buffer(&*buf_static);
    }

    {
        // CHECKERBOARD
        fill_checkerboard(buf_static, 16, 0x00CCCCCC, 0x00333333);
        let mut screen = SCREEN.lock();
        screen.write_buffer(&*buf_static);
    }

    {
        // PALETTE TILES
        fill_palette_tiles(buf_static, 32, 32);
        let mut screen = SCREEN.lock();
        screen.write_buffer(&*buf_static);
    }

    {
        // BORDER + CROSS
        fill_border_cross(buf_static, 4, 0x00FF00FF, 0x0000FF00);
        let mut screen = SCREEN.lock();
        screen.write_buffer(&*buf_static);
    }

    // NOTE: We intentionally leaked the box (Box::leak). If you want to free it later,
    // store the raw pointer / Layout so you can reconstruct and free it. For tests/demos
    // leaking once is normally acceptable.
}

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
