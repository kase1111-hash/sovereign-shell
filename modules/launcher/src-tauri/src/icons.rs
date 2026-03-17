//! Icon extraction for the Sovereign Launcher.
//!
//! Extracts application icons from .exe and .lnk targets using Win32 APIs,
//! converts them to PNG, and caches them in the data directory.
//! The frontend displays these via local file:// URLs.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

/// In-memory cache mapping executable paths to their extracted icon PNG paths.
pub struct IconCache {
    cache_dir: PathBuf,
    map: Mutex<HashMap<String, Option<String>>>,
}

impl IconCache {
    /// Create a new icon cache. Icons are saved as PNGs in `cache_dir`.
    pub fn new(cache_dir: PathBuf) -> Self {
        let _ = std::fs::create_dir_all(&cache_dir);
        Self {
            cache_dir,
            map: Mutex::new(HashMap::new()),
        }
    }

    /// Get the cached icon path for an executable, extracting it if needed.
    /// Returns None if extraction fails or the platform doesn't support it.
    pub fn get_or_extract(&self, exe_path: &str) -> Option<String> {
        let mut map = self.map.lock().ok()?;

        if let Some(cached) = map.get(exe_path) {
            return cached.clone();
        }

        let result = extract_icon_to_png(exe_path, &self.cache_dir);
        map.insert(exe_path.to_string(), result.clone());
        result
    }
}

/// Extract an icon from a file and save it as a PNG.
/// Returns the path to the saved PNG file, or None on failure.
#[cfg(windows)]
fn extract_icon_to_png(file_path: &str, cache_dir: &Path) -> Option<String> {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    use windows::Win32::Foundation::HWND;
    use windows::Win32::Graphics::Gdi::{
        CreateCompatibleDC, DeleteDC, DeleteObject, GetDIBits, SelectObject,
        BITMAPINFO, BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS,
    };
    use windows::Win32::UI::Shell::{
        SHGetFileInfoW, SHFILEINFOW, SHGFI_ICON, SHGFI_LARGEICON,
    };
    use windows::Win32::UI::WindowsAndMessaging::{
        DestroyIcon, GetIconInfo, ICONINFO,
    };

    // Generate a stable filename from the path hash
    let hash = simple_hash(file_path);
    let png_path = cache_dir.join(format!("{:016x}.png", hash));

    // If already extracted, return the cached path
    if png_path.exists() {
        return Some(png_path.to_string_lossy().to_string());
    }

    unsafe {
        let wide_path: Vec<u16> = OsStr::new(file_path)
            .encode_wide()
            .chain(Some(0))
            .collect();

        let mut shfi = SHFILEINFOW::default();
        let result = SHGetFileInfoW(
            windows::core::PCWSTR(wide_path.as_ptr()),
            windows::Win32::Storage::FileSystem::FILE_ATTRIBUTE_NORMAL,
            Some(&mut shfi),
            std::mem::size_of::<SHFILEINFOW>() as u32,
            SHGFI_ICON | SHGFI_LARGEICON,
        );

        if result == 0 || shfi.hIcon.is_invalid() {
            return None;
        }

        let icon = shfi.hIcon;

        // Get icon bitmap info
        let mut icon_info = ICONINFO::default();
        if GetIconInfo(icon, &mut icon_info).is_err() {
            DestroyIcon(icon).ok();
            return None;
        }

        let hdc = CreateCompatibleDC(None);
        if hdc.is_invalid() {
            DestroyIcon(icon).ok();
            return None;
        }

        // Get bitmap dimensions
        let mut bmi = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: 32,
                biHeight: -32, // top-down
                biPlanes: 1,
                biBitCount: 32,
                biCompression: BI_RGB.0 as u32,
                ..Default::default()
            },
            ..Default::default()
        };

        let bitmap = if !icon_info.hbmColor.is_invalid() {
            icon_info.hbmColor
        } else {
            icon_info.hbmMask
        };

        let old = SelectObject(hdc, bitmap);
        let mut pixels = vec![0u8; 32 * 32 * 4];

        let lines = GetDIBits(
            hdc,
            bitmap,
            0,
            32,
            Some(pixels.as_mut_ptr() as *mut _),
            &mut bmi,
            DIB_RGB_COLORS,
        );

        SelectObject(hdc, old);
        DeleteDC(hdc).ok();

        // Clean up GDI objects
        if !icon_info.hbmColor.is_invalid() {
            DeleteObject(icon_info.hbmColor).ok();
        }
        if !icon_info.hbmMask.is_invalid() {
            DeleteObject(icon_info.hbmMask).ok();
        }
        DestroyIcon(icon).ok();

        if lines == 0 {
            return None;
        }

        // Convert BGRA to RGBA
        for chunk in pixels.chunks_exact_mut(4) {
            chunk.swap(0, 2); // swap B and R
        }

        // Encode as PNG using a minimal encoder
        let png_data = encode_rgba_png(32, 32, &pixels)?;
        std::fs::write(&png_path, png_data).ok()?;

        Some(png_path.to_string_lossy().to_string())
    }
}

#[cfg(not(windows))]
fn extract_icon_to_png(_file_path: &str, _cache_dir: &Path) -> Option<String> {
    None
}

/// Simple non-cryptographic hash for generating cache filenames.
fn simple_hash(s: &str) -> u64 {
    let mut hash: u64 = 0xcbf29ce484222325;
    for byte in s.bytes() {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

/// Minimal PNG encoder for 32x32 RGBA images.
/// Avoids needing an external image crate dependency.
fn encode_rgba_png(width: u32, height: u32, rgba: &[u8]) -> Option<Vec<u8>> {
    use std::io::Write;

    let mut png = Vec::new();

    // PNG signature
    png.extend_from_slice(&[137, 80, 78, 71, 13, 10, 26, 10]);

    // IHDR chunk
    let mut ihdr = Vec::new();
    ihdr.extend_from_slice(&width.to_be_bytes());
    ihdr.extend_from_slice(&height.to_be_bytes());
    ihdr.push(8);  // bit depth
    ihdr.push(6);  // color type: RGBA
    ihdr.push(0);  // compression
    ihdr.push(0);  // filter
    ihdr.push(0);  // interlace
    write_png_chunk(&mut png, b"IHDR", &ihdr);

    // IDAT chunk — raw pixel data with zlib wrapping
    let row_bytes = (width as usize) * 4;
    let mut raw = Vec::with_capacity((row_bytes + 1) * height as usize);
    for y in 0..height as usize {
        raw.push(0); // filter: None
        let start = y * row_bytes;
        let end = start + row_bytes;
        if end <= rgba.len() {
            raw.extend_from_slice(&rgba[start..end]);
        } else {
            raw.extend_from_slice(&vec![0u8; row_bytes]);
        }
    }

    let compressed = deflate_raw(&raw);
    write_png_chunk(&mut png, b"IDAT", &compressed);

    // IEND chunk
    write_png_chunk(&mut png, b"IEND", &[]);

    Some(png)
}

fn write_png_chunk(out: &mut Vec<u8>, chunk_type: &[u8; 4], data: &[u8]) {
    out.extend_from_slice(&(data.len() as u32).to_be_bytes());
    out.extend_from_slice(chunk_type);
    out.extend_from_slice(data);
    let mut crc_data = Vec::with_capacity(4 + data.len());
    crc_data.extend_from_slice(chunk_type);
    crc_data.extend_from_slice(data);
    let crc = crc32(&crc_data);
    out.extend_from_slice(&crc.to_be_bytes());
}

fn crc32(data: &[u8]) -> u32 {
    let mut crc: u32 = 0xFFFFFFFF;
    for &byte in data {
        crc ^= byte as u32;
        for _ in 0..8 {
            if crc & 1 != 0 {
                crc = (crc >> 1) ^ 0xEDB88320;
            } else {
                crc >>= 1;
            }
        }
    }
    !crc
}

/// Minimal deflate using zlib stored blocks (uncompressed).
/// Not optimal for size, but correct and dependency-free.
fn deflate_raw(data: &[u8]) -> Vec<u8> {
    let mut out = Vec::new();

    // Zlib header: CM=8 (deflate), CINFO=7 (32K window), FCHECK
    out.push(0x78);
    out.push(0x01);

    // Split into stored blocks of max 65535 bytes
    let chunks: Vec<&[u8]> = data.chunks(65535).collect();
    for (i, chunk) in chunks.iter().enumerate() {
        let is_last = i == chunks.len() - 1;
        out.push(if is_last { 0x01 } else { 0x00 }); // BFINAL + BTYPE=00 (stored)
        let len = chunk.len() as u16;
        out.extend_from_slice(&len.to_le_bytes());
        out.extend_from_slice(&(!len).to_le_bytes());
        out.extend_from_slice(chunk);
    }

    // Adler-32 checksum
    let adler = adler32(data);
    out.extend_from_slice(&adler.to_be_bytes());

    out
}

fn adler32(data: &[u8]) -> u32 {
    let mut a: u32 = 1;
    let mut b: u32 = 0;
    for &byte in data {
        a = (a + byte as u32) % 65521;
        b = (b + a) % 65521;
    }
    (b << 16) | a
}
