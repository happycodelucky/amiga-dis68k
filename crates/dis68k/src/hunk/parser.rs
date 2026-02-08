use super::error::HunkError;
use super::types::*;

/// A zero-copy cursor over a byte slice for big-endian binary parsing.
///
/// All Amiga hunk data is big-endian (68k native byte order) and
/// longword-aligned. The cursor tracks a read position and provides
/// checked reads that return `HunkError` on out-of-bounds access.
struct Cursor<'a> {
    data: &'a [u8],
    pos: usize,
}

impl<'a> Cursor<'a> {
    fn new(data: &'a [u8]) -> Self {
        Cursor { data, pos: 0 }
    }

    fn position(&self) -> usize {
        self.pos
    }

    fn remaining(&self) -> usize {
        self.data.len().saturating_sub(self.pos)
    }

    fn read_u32_be(&mut self) -> Result<u32, HunkError> {
        if self.pos + 4 > self.data.len() {
            return Err(HunkError::TooShort {
                offset: self.pos,
                needed: 4,
                available: self.remaining(),
            });
        }
        let bytes = &self.data[self.pos..self.pos + 4];
        self.pos += 4;
        Ok(u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
    }

    fn read_u16_be(&mut self) -> Result<u16, HunkError> {
        if self.pos + 2 > self.data.len() {
            return Err(HunkError::TooShort {
                offset: self.pos,
                needed: 2,
                available: self.remaining(),
            });
        }
        let bytes = &self.data[self.pos..self.pos + 2];
        self.pos += 2;
        Ok(u16::from_be_bytes([bytes[0], bytes[1]]))
    }

    fn read_bytes(&mut self, n: usize) -> Result<&'a [u8], HunkError> {
        if self.pos + n > self.data.len() {
            return Err(HunkError::TooShort {
                offset: self.pos,
                needed: n,
                available: self.remaining(),
            });
        }
        let slice = &self.data[self.pos..self.pos + n];
        self.pos += n;
        Ok(slice)
    }

    fn skip(&mut self, n: usize) -> Result<(), HunkError> {
        if self.pos + n > self.data.len() {
            return Err(HunkError::TooShort {
                offset: self.pos,
                needed: n,
                available: self.remaining(),
            });
        }
        self.pos += n;
        Ok(())
    }

    /// Align the cursor position up to the next longword (4-byte) boundary.
    fn align_to_longword(&mut self) {
        let rem = self.pos % 4;
        if rem != 0 {
            let skip = 4 - rem;
            // Don't error if we're exactly at the end after alignment
            self.pos = (self.pos + skip).min(self.data.len());
        }
    }

    /// Read an Amiga-style length-prefixed string.
    ///
    /// The first longword gives the string length in longwords (4-byte units).
    /// The string data follows, null-padded to a longword boundary.
    fn read_amiga_string(&mut self) -> Result<String, HunkError> {
        let num_longs = self.read_u32_be()?;
        if num_longs == 0 {
            return Ok(String::new());
        }
        // Sanity check: strings shouldn't be megabytes long
        if num_longs > 0x10000 {
            return Err(HunkError::InvalidStringLength {
                length: num_longs,
                offset: self.pos - 4,
            });
        }
        let byte_len = (num_longs as usize) * 4;
        let bytes = self.read_bytes(byte_len)?;
        // Find the actual string end (first null byte or full length)
        let str_end = bytes.iter().position(|&b| b == 0).unwrap_or(byte_len);
        Ok(String::from_utf8_lossy(&bytes[..str_end]).into_owned())
    }

    fn is_eof(&self) -> bool {
        self.pos >= self.data.len()
    }
}

/// Parse a complete Amiga hunk executable from raw bytes.
///
/// This is the main entry point for the hunk parser. It expects the
/// complete file contents as a byte slice and returns a structured
/// representation of all hunks, their data, relocations, and symbols.
///
/// # Example
///
/// ```no_run
/// use dis68k::hunk::parser::parse_hunk_file;
///
/// let file_data = std::fs::read("program.exe").unwrap();
/// let hunk_file = parse_hunk_file(&file_data).unwrap();
/// for hunk in &hunk_file.hunks {
///     println!("Hunk {}: {} ({} bytes)", hunk.index, hunk.hunk_type, hunk.data.len());
/// }
/// ```
pub fn parse_hunk_file(data: &[u8]) -> Result<HunkFile, HunkError> {
    let mut cursor = Cursor::new(data);

    // --- Read and validate the HUNK_HEADER ---
    let magic = cursor.read_u32_be()?;
    if magic != hunk_ids::HUNK_HEADER {
        return Err(HunkError::BadMagic { found: magic });
    }

    // Skip resident library names (sequence of strings terminated by empty string)
    loop {
        let name_len = cursor.read_u32_be()?;
        if name_len == 0 {
            break;
        }
        // Skip the string data
        cursor.skip((name_len as usize) * 4)?;
    }

    // Read hunk table
    let num_hunks = cursor.read_u32_be()? as usize;
    if num_hunks > 65536 {
        return Err(HunkError::InvalidValue {
            context: "hunk count > 65536",
            value: num_hunks as u32,
        });
    }

    let first_hunk = cursor.read_u32_be()?;
    let last_hunk = cursor.read_u32_be()?;

    // Read the size table: one entry per hunk
    let mut hunk_sizes = Vec::with_capacity(num_hunks);
    let mut hunk_mem_types = Vec::with_capacity(num_hunks);
    for _ in 0..num_hunks {
        let size_word = cursor.read_u32_be()?;
        let mem_type = MemoryType::from_flags(size_word);
        let size_longs = size_word & 0x3FFFFFFF;
        let size_bytes = size_longs * 4;

        // If extended memory type (both bits set), read the additional spec word
        if matches!(mem_type, MemoryType::Extended(_)) {
            let _ext_attr = cursor.read_u32_be()?;
        }

        hunk_mem_types.push(mem_type);
        hunk_sizes.push(size_bytes);
    }

    // --- Parse the hunk content ---
    let mut hunks: Vec<Hunk> = Vec::with_capacity(num_hunks);
    let mut current_hunk_idx: usize = 0;

    // Use loop with explicit break conditions instead of just while
    loop {
        if cursor.is_eof() {
            break;
        }
        
        // Peek or read next word. If we have parsed all expected hunks,
        // we should expect EOF or handle extra data (overlay? debug?).
        // For now, if we've reached the count, we stop unless there's more.
        // But the standard format is strictly sequential.
        
        // If we are about to read a hunk type but have already fulfilled the count?
        // Some linkers might append debug info or overlay. 
        // We'll stick to the loop condition but check consistency at the end.

        let type_word = match cursor.read_u32_be() {
            Ok(w) => w,
            Err(_) => break, // EOF handled gracefully if between hunks
        };

        let mem_flags = MemoryType::from_flags(type_word);
        let hunk_type = HunkType::from_raw(type_word).ok_or(HunkError::UnknownHunkType {
            raw: type_word,
            offset: cursor.position() - 4,
        })?;

        match hunk_type {
            HunkType::Code | HunkType::Data => {
                let data_longs = cursor.read_u32_be()? as usize;
                let data_bytes = data_longs * 4;
                let content = cursor.read_bytes(data_bytes)?.to_vec();

                let alloc_size = if current_hunk_idx < hunk_sizes.len() {
                    hunk_sizes[current_hunk_idx]
                } else {
                    data_bytes as u32
                };

                let memory_type = if current_hunk_idx < hunk_mem_types.len() {
                    // Content hunk's own flags override header if non-Any
                    if matches!(mem_flags, MemoryType::Any) {
                        hunk_mem_types[current_hunk_idx]
                    } else {
                        mem_flags
                    }
                } else {
                    mem_flags
                };

                hunks.push(Hunk {
                    index: current_hunk_idx,
                    hunk_type,
                    memory_type,
                    alloc_size,
                    data: content,
                    relocations: Vec::new(),
                    symbols: Vec::new(),
                    name: None,
                    debug_data: None,
                });
            }

            HunkType::Bss => {
                let bss_longs = cursor.read_u32_be()?;
                let alloc_size = if current_hunk_idx < hunk_sizes.len() {
                    hunk_sizes[current_hunk_idx]
                } else {
                    bss_longs * 4
                };

                let memory_type = if current_hunk_idx < hunk_mem_types.len() {
                    if matches!(mem_flags, MemoryType::Any) {
                        hunk_mem_types[current_hunk_idx]
                    } else {
                        mem_flags
                    }
                } else {
                    mem_flags
                };

                hunks.push(Hunk {
                    index: current_hunk_idx,
                    hunk_type,
                    memory_type,
                    alloc_size,
                    data: Vec::new(),
                    relocations: Vec::new(),
                    symbols: Vec::new(),
                    name: None,
                    debug_data: None,
                });
            }

            HunkType::Reloc32 => {
                parse_reloc32(&mut cursor, &mut hunks)?;
            }

            HunkType::Reloc32Short => {
                parse_reloc32_short(&mut cursor, &mut hunks)?;
            }

            HunkType::Symbol => {
                parse_symbols(&mut cursor, &mut hunks)?;
            }

            HunkType::Debug => {
                let debug_longs = cursor.read_u32_be()? as usize;
                let debug_bytes = debug_longs * 4;
                let debug_data = cursor.read_bytes(debug_bytes)?.to_vec();
                if let Some(hunk) = hunks.last_mut() {
                    hunk.debug_data = Some(debug_data);
                }
            }

            HunkType::End => {
                current_hunk_idx += 1;
                // If we've parsed all hunks, we can stop
                if current_hunk_idx >= num_hunks {
                    break;
                }
            }

            HunkType::Name => {
                let name = cursor.read_amiga_string()?;
                // The name applies to the next content hunk, or the
                // current one if it already exists
                if let Some(hunk) = hunks.last_mut() {
                    if hunk.index == current_hunk_idx {
                        hunk.name = Some(name);
                    }
                }
                // If the name came before the content hunk, we'll need
                // to attach it after — for now we skip that case.
            }

            HunkType::Ext => {
                // Skip HUNK_EXT for now — we'll parse it in a later phase
                skip_ext_block(&mut cursor)?;
            }

            // Relocation types we'll handle later — skip their data
            HunkType::RelReloc32
            | HunkType::RelReloc16
            | HunkType::RelReloc8
            | HunkType::DReloc32
            | HunkType::DReloc16
            | HunkType::DReloc8
            | HunkType::AbsReloc16 => {
                skip_reloc_block(&mut cursor)?;
            }

            HunkType::Overlay | HunkType::Break => {
                // Overlay executables — skip for now
                break;
            }

            HunkType::Header => {
                return Err(HunkError::InvalidValue {
                    context: "unexpected HUNK_HEADER in body",
                    value: type_word,
                });
            }

            HunkType::Unit | HunkType::Lib | HunkType::Index => {
                // Object file / library format — not supported in load files
                return Err(HunkError::InvalidValue {
                    context: "object/library hunk in executable",
                    value: type_word,
                });
            }
        }
    }

    if hunks.len() != num_hunks {
        return Err(HunkError::HunkCountMismatch {
            expected: num_hunks,
            found: hunks.len(),
        });
    }

    Ok(HunkFile {
        hunks,
        first_hunk,
        last_hunk,
    })
}

/// Parse HUNK_RELOC32: groups of (count, target_hunk, offsets...) until count == 0.
fn parse_reloc32(cursor: &mut Cursor<'_>, hunks: &mut [Hunk]) -> Result<(), HunkError> {
    loop {
        let count = cursor.read_u32_be()?;
        if count == 0 {
            break;
        }
        // Safety check: verify we have enough data for `count` offsets (4 bytes each).
        // Plus 4 bytes for the target hunk index.
        let needed = (count as usize * 4) + 4;
        if cursor.remaining() < needed {
            return Err(HunkError::TooShort {
                offset: cursor.position(),
                needed,
                available: cursor.remaining(),
            });
        }

        let target_hunk = cursor.read_u32_be()?;
        let mut offsets = Vec::with_capacity(count as usize);
        for _ in 0..count {
            offsets.push(cursor.read_u32_be()?);
        }
        if let Some(hunk) = hunks.last_mut() {
            hunk.relocations.push(Relocation {
                target_hunk,
                offsets,
            });
        }
    }
    Ok(())
}

/// Parse HUNK_RELOC32SHORT: same structure but with 16-bit values.
fn parse_reloc32_short(cursor: &mut Cursor<'_>, hunks: &mut [Hunk]) -> Result<(), HunkError> {
    loop {
        let count = cursor.read_u16_be()? as u32;
        if count == 0 {
            break;
        }
        // Safety check: verify we have enough data for `count` offsets (2 bytes each).
        // Plus 2 bytes for the target hunk index.
        let needed = (count as usize * 2) + 2;
        if cursor.remaining() < needed {
            return Err(HunkError::TooShort {
                offset: cursor.position(),
                needed,
                available: cursor.remaining(),
            });
        }

        let target_hunk = cursor.read_u16_be()? as u32;
        let mut offsets = Vec::with_capacity(count as usize);
        for _ in 0..count {
            offsets.push(cursor.read_u16_be()? as u32);
        }
        if let Some(hunk) = hunks.last_mut() {
            hunk.relocations.push(Relocation {
                target_hunk,
                offsets,
            });
        }
    }
    // RELOC32SHORT must be padded to longword boundary
    cursor.align_to_longword();
    Ok(())
}

/// Parse HUNK_SYMBOL: pairs of (name, value) until name_length == 0.
fn parse_symbols(cursor: &mut Cursor<'_>, hunks: &mut [Hunk]) -> Result<(), HunkError> {
    loop {
        let name_longs = cursor.read_u32_be()?;
        if name_longs == 0 {
            break;
        }
        if name_longs > 0x10000 {
            return Err(HunkError::InvalidStringLength {
                length: name_longs,
                offset: cursor.position() - 4,
            });
        }
        let byte_len = (name_longs as usize) * 4;
        let name_bytes = cursor.read_bytes(byte_len)?;
        let str_end = name_bytes
            .iter()
            .position(|&b| b == 0)
            .unwrap_or(byte_len);
        let name = String::from_utf8_lossy(&name_bytes[..str_end]).into_owned();
        let value = cursor.read_u32_be()?;

        if let Some(hunk) = hunks.last_mut() {
            hunk.symbols.push(Symbol { name, value });
        }
    }
    Ok(())
}

/// Skip a HUNK_EXT block (same termination pattern as HUNK_SYMBOL but more complex entries).
fn skip_ext_block(cursor: &mut Cursor<'_>) -> Result<(), HunkError> {
    loop {
        let header = cursor.read_u32_be()?;
        if header == 0 {
            break;
        }
        let ext_type = (header >> 24) & 0xFF;
        let name_longs = header & 0x00FFFFFF;
        // Skip the name
        cursor.skip((name_longs as usize) * 4)?;

        if ext_type < 128 {
            // Definition: has a value longword
            cursor.skip(4)?;
        } else if ext_type == 130 || ext_type == 137 {
            // Common symbol: value + reference count + offsets
            cursor.skip(4)?; // common size
            let ref_count = cursor.read_u32_be()?;
            cursor.skip((ref_count as usize) * 4)?;
        } else {
            // Reference: has a count + offsets
            let ref_count = cursor.read_u32_be()?;
            cursor.skip((ref_count as usize) * 4)?;
        }
    }
    Ok(())
}

/// Skip a standard relocation block (same pattern as RELOC32: count/target/offsets groups).
fn skip_reloc_block(cursor: &mut Cursor<'_>) -> Result<(), HunkError> {
    loop {
        let count = cursor.read_u32_be()?;
        if count == 0 {
            break;
        }
        // target hunk + offsets
        cursor.skip(4 + (count as usize) * 4)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper to build a minimal valid hunk executable in memory.
    fn build_executable(hunks_data: &[HunkBuilder]) -> Vec<u8> {
        let mut out = Vec::new();

        // HUNK_HEADER
        out.extend_from_slice(&hunk_ids::HUNK_HEADER.to_be_bytes());
        // No resident library names
        out.extend_from_slice(&0u32.to_be_bytes());
        // Number of hunks
        let num_hunks = hunks_data.len() as u32;
        out.extend_from_slice(&num_hunks.to_be_bytes());
        // First/last hunk
        out.extend_from_slice(&0u32.to_be_bytes());
        out.extend_from_slice(&(num_hunks.saturating_sub(1)).to_be_bytes());
        // Size table
        for hunk in hunks_data {
            let size_longs = (hunk.data.len() / 4) as u32;
            out.extend_from_slice(&size_longs.to_be_bytes());
        }
        // Hunk bodies
        for hunk in hunks_data {
            // Hunk type
            let type_id = match hunk.hunk_type {
                HunkType::Code => hunk_ids::HUNK_CODE,
                HunkType::Data => hunk_ids::HUNK_DATA,
                HunkType::Bss => hunk_ids::HUNK_BSS,
                _ => panic!("unsupported hunk type in test builder"),
            };
            out.extend_from_slice(&type_id.to_be_bytes());

            if hunk.hunk_type == HunkType::Bss {
                let size_longs = (hunk.data.len() / 4) as u32;
                out.extend_from_slice(&size_longs.to_be_bytes());
            } else {
                let size_longs = (hunk.data.len() / 4) as u32;
                out.extend_from_slice(&size_longs.to_be_bytes());
                out.extend_from_slice(&hunk.data);
            }

            // HUNK_END
            out.extend_from_slice(&hunk_ids::HUNK_END.to_be_bytes());
        }
        out
    }

    struct HunkBuilder {
        hunk_type: HunkType,
        data: Vec<u8>,
    }

    #[test]
    fn parse_minimal_code_hunk() {
        // A single code hunk containing just RTS (0x4E75), padded to longword
        let exe = build_executable(&[HunkBuilder {
            hunk_type: HunkType::Code,
            data: vec![0x4E, 0x75, 0x00, 0x00],
        }]);

        let result = parse_hunk_file(&exe).unwrap();
        assert_eq!(result.hunks.len(), 1);
        assert_eq!(result.hunks[0].hunk_type, HunkType::Code);
        assert_eq!(result.hunks[0].data, vec![0x4E, 0x75, 0x00, 0x00]);
        assert_eq!(result.hunks[0].alloc_size, 4);
        assert_eq!(result.first_hunk, 0);
        assert_eq!(result.last_hunk, 0);
    }

    #[test]
    fn parse_code_and_data_hunks() {
        let exe = build_executable(&[
            HunkBuilder {
                hunk_type: HunkType::Code,
                data: vec![0x4E, 0x75, 0x00, 0x00],
            },
            HunkBuilder {
                hunk_type: HunkType::Data,
                data: vec![0x00, 0x00, 0x00, 0x42],
            },
        ]);

        let result = parse_hunk_file(&exe).unwrap();
        assert_eq!(result.hunks.len(), 2);
        assert_eq!(result.hunks[0].hunk_type, HunkType::Code);
        assert_eq!(result.hunks[1].hunk_type, HunkType::Data);
        assert_eq!(result.hunks[1].data, vec![0x00, 0x00, 0x00, 0x42]);
    }

    #[test]
    fn parse_bss_hunk() {
        let exe = build_executable(&[HunkBuilder {
            hunk_type: HunkType::Bss,
            data: vec![0; 256], // 256 bytes = 64 longwords of BSS
        }]);

        let result = parse_hunk_file(&exe).unwrap();
        assert_eq!(result.hunks.len(), 1);
        assert_eq!(result.hunks[0].hunk_type, HunkType::Bss);
        assert!(result.hunks[0].data.is_empty()); // BSS has no content data
        assert_eq!(result.hunks[0].alloc_size, 256);
    }

    #[test]
    fn parse_reloc32() {
        // Build a code hunk with HUNK_RELOC32 manually
        let mut out = Vec::new();

        // Header
        out.extend_from_slice(&hunk_ids::HUNK_HEADER.to_be_bytes());
        out.extend_from_slice(&0u32.to_be_bytes()); // no lib names
        out.extend_from_slice(&1u32.to_be_bytes()); // 1 hunk
        out.extend_from_slice(&0u32.to_be_bytes()); // first
        out.extend_from_slice(&0u32.to_be_bytes()); // last
        out.extend_from_slice(&2u32.to_be_bytes()); // size: 2 longs = 8 bytes

        // HUNK_CODE
        out.extend_from_slice(&hunk_ids::HUNK_CODE.to_be_bytes());
        out.extend_from_slice(&2u32.to_be_bytes()); // 2 longs of code
        out.extend_from_slice(&[0x4E, 0xB9, 0x00, 0x00, 0x00, 0x00, 0x4E, 0x75]);

        // HUNK_RELOC32
        out.extend_from_slice(&hunk_ids::HUNK_RELOC32.to_be_bytes());
        out.extend_from_slice(&1u32.to_be_bytes()); // 1 offset
        out.extend_from_slice(&0u32.to_be_bytes()); // target hunk 0
        out.extend_from_slice(&2u32.to_be_bytes()); // offset 2 (the address in JSR)
        out.extend_from_slice(&0u32.to_be_bytes()); // end of relocs

        // HUNK_END
        out.extend_from_slice(&hunk_ids::HUNK_END.to_be_bytes());

        let result = parse_hunk_file(&out).unwrap();
        assert_eq!(result.hunks[0].relocations.len(), 1);
        assert_eq!(result.hunks[0].relocations[0].target_hunk, 0);
        assert_eq!(result.hunks[0].relocations[0].offsets, vec![2]);
    }

    #[test]
    fn parse_symbols() {
        let mut out = Vec::new();

        // Header
        out.extend_from_slice(&hunk_ids::HUNK_HEADER.to_be_bytes());
        out.extend_from_slice(&0u32.to_be_bytes());
        out.extend_from_slice(&1u32.to_be_bytes());
        out.extend_from_slice(&0u32.to_be_bytes());
        out.extend_from_slice(&0u32.to_be_bytes());
        out.extend_from_slice(&1u32.to_be_bytes()); // 1 longword = 4 bytes

        // HUNK_CODE
        out.extend_from_slice(&hunk_ids::HUNK_CODE.to_be_bytes());
        out.extend_from_slice(&1u32.to_be_bytes());
        out.extend_from_slice(&[0x4E, 0x75, 0x00, 0x00]); // RTS + pad

        // HUNK_SYMBOL
        out.extend_from_slice(&hunk_ids::HUNK_SYMBOL.to_be_bytes());
        // Symbol: "_main" (5 chars -> 2 longwords, null-padded)
        out.extend_from_slice(&2u32.to_be_bytes());
        out.extend_from_slice(b"_mai");
        out.extend_from_slice(b"n\x00\x00\x00");
        out.extend_from_slice(&0u32.to_be_bytes()); // value = 0
        // End of symbols
        out.extend_from_slice(&0u32.to_be_bytes());

        // HUNK_END
        out.extend_from_slice(&hunk_ids::HUNK_END.to_be_bytes());

        let result = parse_hunk_file(&out).unwrap();
        assert_eq!(result.hunks[0].symbols.len(), 1);
        assert_eq!(result.hunks[0].symbols[0].name, "_main");
        assert_eq!(result.hunks[0].symbols[0].value, 0);
    }

    #[test]
    fn error_on_bad_magic() {
        let data = [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
        let result = parse_hunk_file(&data);
        assert!(matches!(result, Err(HunkError::BadMagic { found: 0 })));
    }

    #[test]
    fn error_on_truncated_header() {
        let data = [0x00, 0x00, 0x03, 0xF3]; // Just the magic, nothing else
        let result = parse_hunk_file(&data);
        assert!(result.is_err());
    }

    #[test]
    fn error_on_too_many_hunks() {
        let mut out = Vec::new();
        // Header
        out.extend_from_slice(&hunk_ids::HUNK_HEADER.to_be_bytes());
        out.extend_from_slice(&0u32.to_be_bytes()); // no lib names
        
        // Number of hunks - 70000 (exceeds 65536 limit)
        out.extend_from_slice(&70000u32.to_be_bytes());
        
        let result = parse_hunk_file(&out);
        assert!(matches!(result, Err(HunkError::InvalidValue { context: "hunk count > 65536", .. })));
    }

    #[test]
    fn error_on_huge_reloc_count() {
        let mut out = Vec::new();

        // Header
        out.extend_from_slice(&hunk_ids::HUNK_HEADER.to_be_bytes());
        out.extend_from_slice(&0u32.to_be_bytes()); 
        out.extend_from_slice(&1u32.to_be_bytes()); 
        out.extend_from_slice(&0u32.to_be_bytes()); 
        out.extend_from_slice(&0u32.to_be_bytes()); 
        out.extend_from_slice(&1u32.to_be_bytes()); // size: 1 long

        // HUNK_CODE
        out.extend_from_slice(&hunk_ids::HUNK_CODE.to_be_bytes());
        out.extend_from_slice(&1u32.to_be_bytes()); // 1 long of code
        out.extend_from_slice(&[0x4E, 0x75, 0x00, 0x00]); // RTS

        // HUNK_RELOC32 with huge count
        out.extend_from_slice(&hunk_ids::HUNK_RELOC32.to_be_bytes());
        out.extend_from_slice(&0x100000u32.to_be_bytes()); // Huge count
        // We stop here. The file is now too short to contain 0x100000 * 4 bytes.

        let result = parse_hunk_file(&out);
        // Should fail with TooShort, NOT panic with OOM
        assert!(matches!(result, Err(HunkError::TooShort { .. })));
    }
}
