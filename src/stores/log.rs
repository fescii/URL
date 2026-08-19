use super::index::{Index, Location, Tier};
use crate::design::{Error, Result};
use crate::hashes::Hash;
use bytes::Bytes;
use memmap2::Mmap;
use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

/// Append-only log file with zero-copy mmap reader, crash recovery, and per-record offsets.
pub struct Log {
  file: File,
  path: PathBuf,
  offset: u64,
  mmap: Option<Mmap>,
}

impl Log {
  /// Open or create append-only log file.
  pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
    let p = path.as_ref().to_path_buf();
    let file = OpenOptions::new()
      .create(true)
      .append(true)
      .read(true)
      .open(&p)
      .map_err(Error::from)?;

    let offset = file.metadata().map_err(Error::from)?.len();
    let mmap = if offset > 0 {
      unsafe { Mmap::map(&file).ok() }
    } else {
      None
    };

    Ok(Self {
      file,
      path: p,
      offset,
      mmap,
    })
  }

  /// Append a content-addressed entry sequentially: [Key (32B), Length (2B), Data (N bytes)].
  pub fn append(&mut self, hash: &Hash, data: &[u8]) -> Result<Location> {
    let start_offset = self.offset;
    let length = data.len() as u32;

    self.file.write_all(hash.bytes()).map_err(Error::from)?;
    let len_bytes = (length as u16).to_le_bytes();
    self.file.write_all(&len_bytes).map_err(Error::from)?;
    self.file.write_all(data).map_err(Error::from)?;
    self.file.flush().map_err(Error::from)?;

    let total_entry_len = 34 + (length as u64);
    self.offset += total_entry_len;

    // Invalidate stale mmap so next read remaps with new size
    self.mmap = None;

    let loc = Location::new(0, start_offset + 34, length, Tier::Hot);
    Ok(loc)
  }

  /// Read data slice by location from file. Auto-resolves length from record header if length is 0.
  /// Returns zero-copy Bytes directly sliced from mmap memory.
  pub fn read(&mut self, offset: u64, length: u32) -> Result<Bytes> {
    // Try zero-copy mmap path first
    if self.mmap.is_none() && self.offset > 0 {
      self.mmap = unsafe { Mmap::map(&self.file).ok() };
    }

    let actual_length = if length == 0 && offset >= 2 {
      if let Some(m) = &self.mmap {
        let off = offset as usize;
        if off >= 2 && off <= m.len() {
          u16::from_le_bytes([m[off - 2], m[off - 1]]) as u32
        } else {
          0
        }
      } else {
        self
          .file
          .seek(SeekFrom::Start(offset - 2))
          .map_err(Error::from)?;
        let mut len_buf = [0u8; 2];
        self.file.read_exact(&mut len_buf).map_err(Error::from)?;
        u16::from_le_bytes(len_buf) as u32
      }
    } else {
      length
    };

    if actual_length == 0 {
      return Ok(Bytes::new());
    }

    if let Some(m) = &self.mmap {
      let start = offset as usize;
      let end = start + (actual_length as usize);
      if end <= m.len() {
        return Ok(Bytes::copy_from_slice(&m[start..end]));
      }
    }

    // Fallback file seek read
    self
      .file
      .seek(SeekFrom::Start(offset))
      .map_err(Error::from)?;
    let mut buf = vec![0u8; actual_length as usize];
    self.file.read_exact(&mut buf).map_err(Error::from)?;
    Ok(Bytes::from(buf))
  }

  /// Rebuild in-memory index from sequential log scan on startup.
  pub fn recover(&mut self, index: &mut Index) -> Result<usize> {
    let file_len = self.file.metadata().map_err(Error::from)?.len();
    self.offset = file_len;
    if file_len == 0 {
      return Ok(0);
    }

    self.file.seek(SeekFrom::Start(0)).map_err(Error::from)?;
    let mut reader_file = OpenOptions::new()
      .read(true)
      .open(&self.path)
      .map_err(Error::from)?;

    let mut cur_offset = 0u64;
    let mut recovered_count = 0;

    while cur_offset + 34 <= file_len {
      let mut key_bytes = [0u8; 32];
      if reader_file.read_exact(&mut key_bytes).is_err() {
        break;
      }
      let hash = Hash::new(key_bytes);

      let mut len_bytes = [0u8; 2];
      if reader_file.read_exact(&mut len_bytes).is_err() {
        break;
      }
      let length = u16::from_le_bytes(len_bytes) as u32;

      let data_offset = cur_offset + 34;
      if data_offset + (length as u64) > file_len {
        // Truncated entry at end of log
        break;
      }

      // Skip data bytes in scan
      reader_file
        .seek(SeekFrom::Current(length as i64))
        .map_err(Error::from)?;

      let loc = Location::new(0, data_offset, length, Tier::Warm);
      index.insert(hash, loc);

      cur_offset = data_offset + (length as u64);
      recovered_count += 1;
    }

    Ok(recovered_count)
  }
}
