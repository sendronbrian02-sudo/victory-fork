//! Optional DDS snapshots of sampled textures after detiling.

use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use std::sync::atomic::{AtomicU32, Ordering::Relaxed};

use windows::Win32::Graphics::Direct3D12::*;

const MAX_FILES: u32 = 256;
const MAX_BYTES: u32 = 16 * 1024 * 1024;
static FILES: AtomicU32 = AtomicU32::new(0);

fn directory() -> Option<&'static Path> {
    static DIRECTORY: OnceLock<Option<PathBuf>> = OnceLock::new();
    DIRECTORY
        .get_or_init(|| {
            let path = PathBuf::from(std::env::var_os("VICTORY_TEXTURE_DUMP")?);
            let checked = (|| -> anyhow::Result<PathBuf> {
                victory_core::ensure_outside_repo(&path)?;
                std::fs::create_dir_all(&path)?;
                Ok(path.canonicalize()?)
            })();
            match checked {
                Ok(dir) => Some(dir),
                Err(error) => {
                    tracing::warn!(%error, path = %path.display(), "texture export disabled");
                    None
                }
            }
        })
        .as_deref()
}

pub(super) struct Texture<'a> {
    dir: &'a Path,
    width: u32,
    height: u32,
    dxgi: u32,
    format: victory_formats::texture::Format,
    /// Best exported mip (0 is the full resolution).
    best: AtomicU32,
}

impl Texture<'static> {
    pub(super) fn new(d: &D3D12_RESOURCE_DESC) -> Option<Self> {
        let dir = directory()?;
        if d.Dimension != D3D12_RESOURCE_DIMENSION_TEXTURE2D
            || d.DepthOrArraySize != 1
            || d.SampleDesc.Count != 1
            || d.Flags.contains(D3D12_RESOURCE_FLAG_ALLOW_DEPTH_STENCIL)
        {
            return None;
        }
        // Common color and block compressed formats. For typeless
        // resources, choose a conventional typed interpretation.
        let dxgi = match d.Format.0 {
            27 => 28,
            60 => 61,
            70 | 73 | 76 | 79 | 82 | 94 | 97 => d.Format.0 + 1,
            90 => 87,
            28..=32
            | 61..=65
            | 71..=72
            | 74..=75
            | 77..=78
            | 80..=81
            | 83..=84
            | 87..=88
            | 91
            | 93
            | 95..=96
            | 98..=99 => d.Format.0,
            _ => return None,
        } as u32;
        Some(Self {
            dir,
            width: u32::try_from(d.Width).ok()?,
            height: d.Height,
            dxgi,
            format: victory_formats::texture::format(dxgi)?,
            best: AtomicU32::new(u32::MAX),
        })
    }
}

impl Texture<'_> {
    pub(super) fn wants(&self, mip: u32) -> bool {
        static FIRST_FRAME: OnceLock<u64> = OnceLock::new();
        let first = *FIRST_FRAME.get_or_init(|| {
            std::env::var("VICTORY_TEXTURE_DUMP_AFTER_FRAME")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(0)
        });
        crate::frame::current() >= first
            && mip < self.best.load(Relaxed)
            && FILES.load(Relaxed) < MAX_FILES
    }

    pub(super) fn save(&self, mip: u32, data: &[u8], stride: usize, rows: u32) {
        let width = (self.width >> mip).max(1);
        let height = (self.height >> mip).max(1);
        let pitch = width.div_ceil(self.format.block) * self.format.bytes;
        let size = pitch.saturating_mul(rows);
        if size > MAX_BYTES
            || rows != height.div_ceil(self.format.block)
            || pitch as usize > stride
            || (rows as usize)
                .checked_mul(stride)
                .is_none_or(|len| len > data.len())
        {
            return;
        }
        let rows = || {
            data.chunks_exact(stride)
                .take(rows as usize)
                .map(|row| &row[..pitch as usize])
        };
        if rows().all(|row| row.iter().all(|&b| b == 0)) {
            return;
        }
        let Ok(n) = FILES.fetch_update(Relaxed, Relaxed, |n| (n < MAX_FILES).then_some(n + 1))
        else {
            return;
        };
        let path = self.dir.join(format!(
            "texture-{}-{n:04}-mip{mip}.dds",
            std::process::id()
        ));
        let result = (|| -> anyhow::Result<()> {
            victory_core::ensure_outside_repo(&path)?;
            let mut file = io::BufWriter::new(std::fs::File::create_new(&path)?);
            let written = (|| -> io::Result<()> {
                let compressed = self.format.block != 1;
                file.write_all(&header(
                    width,
                    height,
                    self.dxgi,
                    if compressed { size } else { pitch },
                    compressed,
                ))?;
                for row in rows() {
                    file.write_all(row)?;
                }
                file.flush()
            })();
            drop(file);
            if written.is_err() {
                let _ = std::fs::remove_file(&path);
            }
            written.map_err(Into::into)
        })();
        match result {
            Ok(()) => {
                self.best.store(mip, Relaxed);
                tracing::info!(path = %path.display(), "texture exported");
            }
            Err(error) => tracing::warn!(%error, path = %path.display(), "texture not exported"),
        }
    }
}

/// DDS_HEADER and DDS_HEADER_DXT10 for a single 2D subresource.
fn header(width: u32, height: u32, dxgi: u32, pitch_or_size: u32, compressed: bool) -> [u8; 148] {
    let mut words = [0u32; 37];
    words[0] = u32::from_le_bytes(*b"DDS ");
    words[1] = 124;
    words[2] = 0x1007 | if compressed { 0x80000 } else { 0x8 };
    words[3] = height;
    words[4] = width;
    words[5] = pitch_or_size;
    words[19] = 32; // DDS_PIXELFORMAT size
    words[20] = 4; // DDPF_FOURCC
    words[21] = u32::from_le_bytes(*b"DX10");
    words[27] = 0x1000; // DDSCAPS_TEXTURE
    words[32] = dxgi;
    words[33] = 3; // DDS_DIMENSION_TEXTURE2D
    words[35] = 1; // array size
    let mut bytes = [0; 148];
    for (chunk, word) in bytes.as_chunks_mut::<4>().0.iter_mut().zip(words) {
        *chunk = word.to_le_bytes();
    }
    bytes
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exports_tight_bc_rows_without_gpu_padding() {
        let dir = tempfile::tempdir().unwrap();
        let texture = Texture {
            dir: dir.path(),
            width: 5,
            height: 6,
            dxgi: 71,
            format: victory_formats::texture::format(71).unwrap(),
            best: AtomicU32::new(u32::MAX),
        };
        let mut data = [0x55; 512];
        data[..16].fill(1);
        data[256..272].fill(2);
        texture.save(0, &data, 256, 2);
        let path = std::fs::read_dir(dir.path())
            .unwrap()
            .next()
            .unwrap()
            .unwrap()
            .path();
        let dds = std::fs::read(path).unwrap();
        assert_eq!(&dds[..4], b"DDS ");
        assert_eq!(&dds[84..88], b"DX10");
        assert_eq!(u32::from_le_bytes(dds[20..24].try_into().unwrap()), 32);
        assert_eq!(u32::from_le_bytes(dds[128..132].try_into().unwrap()), 71);
        assert_eq!(&dds[148..], &[[1; 16], [2; 16]].concat());
        assert_eq!(texture.best.load(Relaxed), 0);
    }
}
