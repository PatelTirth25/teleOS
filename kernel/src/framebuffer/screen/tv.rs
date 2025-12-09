pub const BUFFER1: [[u32; 256]; 240] = {
    let mut buf = [[0u32; 256]; 240];
    let mut seed = 0x12345678u32;
    let colors = [0x00FFFFFF, 0x007F7F7F, 0x00000000];

    let mut y = 0;
    while y < 240 {
        let mut x = 0;
        while x < 256 {
            // simple LCG pseudo-random
            seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
            let idx = (seed >> 16) as usize % 3;
            buf[y][x] = colors[idx];
            x += 1;
        }
        y += 1;
    }
    buf
};

pub const BUFFER2: [[u32; 256]; 240] = {
    let mut buf = [[0u32; 256]; 240];
    let mut seed = 0xCAFEBABEu32;
    let colors = [0x00FFFFFF, 0x007F7F7F, 0x00000000];

    let mut y = 0;
    while y < 240 {
        let mut x = 0;
        while x < 256 {
            seed ^= seed << 13;
            seed ^= seed >> 17;
            seed ^= seed << 5;
            let idx = (seed as usize) % 3;
            buf[y][x] = colors[idx];
            x += 1;
        }
        y += 1;
    }
    buf
};

pub const BUFFER3: [[u32; 256]; 240] = {
    let mut buf = [[0u32; 256]; 240];
    let mut seed = 0xA1B2C3D4u32;
    let colors = [0x00FFFFFF, 0x007F7F7F, 0x00000000];

    let mut y = 0;
    while y < 240 {
        let mut x = 0;
        while x < 256 {
            seed = seed.rotate_left(5) ^ 0x9E3779B9;
            let idx = (seed as usize) & 2 | ((seed >> 7) as usize & 1);
            buf[y][x] = colors[idx % 3];
            x += 1;
        }
        y += 1;
    }
    buf
};

pub const BUFFER4: [[u32; 256]; 240] = {
    let mut buf = [[0u32; 256]; 240];
    let mut seed = 0xDEADBEEFu32;
    let colors = [0x00FFFFFF, 0x007F7F7F, 0x00000000];

    let mut y = 0;
    while y < 240 {
        let mut x = 0;
        while x < 256 {
            // XOR-shift-like noise
            seed ^= seed >> 3;
            seed ^= seed << 7;
            seed ^= seed >> 11;
            let idx = ((seed >> 10) as usize) % 3;
            buf[y][x] = colors[idx];
            x += 1;
        }
        y += 1;
    }
    buf
};
