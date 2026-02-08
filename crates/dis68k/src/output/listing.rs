use crate::hunk::types::{HunkFile, HunkType};
use crate::m68k::decode::{decode_instruction, DecodeError};
use crate::m68k::variants::CpuVariant;

use super::formatter::{format_instruction, FormatOptions};

/// Options controlling the listing output.
#[derive(Debug, Clone)]
pub struct ListingOptions {
    pub show_hex: bool,
    pub show_addresses: bool,
    pub show_line_numbers: bool,
    pub uppercase: bool,
    pub cpu: CpuVariant,
}

impl Default for ListingOptions {
    fn default() -> Self {
        ListingOptions {
            show_hex: true,
            show_addresses: true,
            show_line_numbers: true,
            uppercase: false,
            cpu: CpuVariant::M68000,
        }
    }
}

/// A single line of the disassembly listing.
#[derive(Debug, Clone)]
pub struct ListingLine {
    pub line_number: u32,
    pub text: String,
}

/// Generate a complete disassembly listing from a parsed hunk file.
///
/// Walks each hunk in order. Code hunks are disassembled instruction by
/// instruction. Data hunks are formatted as `dc.b`/`dc.l` directives.
/// BSS hunks show `ds.b` reservations.
pub fn generate_listing(hunk_file: &HunkFile, options: &ListingOptions) -> Vec<ListingLine> {
    let mut lines = Vec::new();
    let mut line_num: u32 = 1;

    let fmt_opts = FormatOptions {
        uppercase: options.uppercase,
    };

    // File header comment
    push_line(
        &mut lines,
        &mut line_num,
        options,
        "; Amiga Hunk Executable Disassembly".to_string(),
    );
    push_line(
        &mut lines,
        &mut line_num,
        options,
        format!("; Hunks: {}", hunk_file.hunks.len()),
    );
    push_line(&mut lines, &mut line_num, options, String::new());

    for hunk in &hunk_file.hunks {
        // Section header
        let section_type = match hunk.hunk_type {
            HunkType::Code => "CODE",
            HunkType::Data => "DATA",
            HunkType::Bss => "BSS",
            _ => "UNKNOWN",
        };
        let default_name = format!("hunk_{}", hunk.index);
        let name = hunk.name.as_deref().unwrap_or(&default_name);
        push_line(&mut lines, &mut line_num, options, String::new());
        push_line(
            &mut lines,
            &mut line_num,
            options,
            format!(
                "; ──── SECTION {}, {} (hunk {}, {} bytes, mem={}) ────",
                name,
                section_type,
                hunk.index,
                hunk.alloc_size,
                hunk.memory_type
            ),
        );

        // Emit symbols as comments
        if !hunk.symbols.is_empty() {
            push_line(
                &mut lines,
                &mut line_num,
                options,
                "; Symbols:".to_string(),
            );
            for sym in &hunk.symbols {
                push_line(
                    &mut lines,
                    &mut line_num,
                    options,
                    format!(";   ${:08X}  {}", sym.value, sym.name),
                );
            }
        }

        push_line(&mut lines, &mut line_num, options, String::new());

        match hunk.hunk_type {
            HunkType::Code => {
                disassemble_code(
                    &hunk.data,
                    &mut lines,
                    &mut line_num,
                    options,
                    &fmt_opts,
                );
            }
            HunkType::Data => {
                format_data_section(
                    &hunk.data,
                    &mut lines,
                    &mut line_num,
                    options,
                );
            }
            HunkType::Bss => {
                let text = format_bss_line(hunk.alloc_size, options);
                push_line(&mut lines, &mut line_num, options, text);
            }
            _ => {}
        }
    }

    lines
}

fn disassemble_code(
    data: &[u8],
    lines: &mut Vec<ListingLine>,
    line_num: &mut u32,
    options: &ListingOptions,
    fmt_opts: &FormatOptions,
) {
    let mut offset = 0usize;

    while offset < data.len() {
        match decode_instruction(data, offset, 0, options.cpu) {
            Ok(inst) => {
                let formatted = format_instruction(&inst, fmt_opts);
                let text = format_code_line(
                    offset as u32,
                    &formatted.hex_bytes,
                    &formatted.mnemonic,
                    &formatted.operands,
                    options,
                );
                push_line(lines, line_num, options, text);
                offset += inst.size_bytes as usize;
            }
            Err(DecodeError::UnexpectedEof { .. }) => {
                // Remaining bytes that don't form a complete instruction
                while offset < data.len() {
                    let byte = data[offset];
                    let text = format_code_line(
                        offset as u32,
                        &format!("{byte:02X}"),
                        "dc.b",
                        &format!("${byte:02X}"),
                        options,
                    );
                    push_line(lines, line_num, options, text);
                    offset += 1;
                }
                break;
            }
            Err(_) => {
                // Unknown opcode — emit dc.w and advance 2 bytes
                if offset + 1 < data.len() {
                    let w = u16::from_be_bytes([data[offset], data[offset + 1]]);
                    let text = format_code_line(
                        offset as u32,
                        &format!("{:04X}", w),
                        "dc.w",
                        &format!("${w:04X}"),
                        options,
                    );
                    push_line(lines, line_num, options, text);
                    offset += 2;
                } else {
                    let byte = data[offset];
                    let text = format_code_line(
                        offset as u32,
                        &format!("{byte:02X}"),
                        "dc.b",
                        &format!("${byte:02X}"),
                        options,
                    );
                    push_line(lines, line_num, options, text);
                    offset += 1;
                }
            }
        }
    }
}

fn format_code_line(
    address: u32,
    hex: &str,
    mnemonic: &str,
    operands: &str,
    options: &ListingOptions,
) -> String {
    let mut parts = Vec::new();

    if options.show_addresses {
        parts.push(format!("{address:08X}"));
    }

    if options.show_hex {
        parts.push(format!("{hex:<20}"));
    }

    if operands.is_empty() {
        parts.push(format!("{mnemonic:<8}"));
    } else {
        parts.push(format!("{mnemonic:<8} {operands}"));
    }

    parts.join("  ")
}

fn format_data_section(
    data: &[u8],
    lines: &mut Vec<ListingLine>,
    line_num: &mut u32,
    options: &ListingOptions,
) {
    // Try to detect ASCII strings; otherwise emit as hex dc.l/dc.b
    let mut offset = 0usize;

    while offset < data.len() {
        // Check for ASCII string run (at least 4 printable chars ending in null)
        if let Some(str_end) = detect_string(data, offset) {
            let s = String::from_utf8_lossy(&data[offset..str_end]);
            let text = format_data_line(
                offset as u32,
                "dc.b",
                &format!("\"{s}\",0"),
                options,
            );
            push_line(lines, line_num, options, text);
            offset = str_end + 1; // skip the null terminator
            // Align to even boundary
            if offset % 2 != 0 && offset < data.len() {
                offset += 1;
            }
            continue;
        }

        // Emit as dc.l if aligned and enough bytes
        if offset % 4 == 0 && offset + 4 <= data.len() {
            let val = u32::from_be_bytes([
                data[offset],
                data[offset + 1],
                data[offset + 2],
                data[offset + 3],
            ]);
            let hex = format!(
                "{:02X}{:02X}{:02X}{:02X}",
                data[offset],
                data[offset + 1],
                data[offset + 2],
                data[offset + 3]
            );
            let mut text = String::new();
            if options.show_addresses {
                text.push_str(&format!("{:08X}  ", offset));
            }
            if options.show_hex {
                text.push_str(&format!("{hex:<20}  "));
            }
            text.push_str(&format!("dc.l     ${val:08X}"));
            push_line(lines, line_num, options, text);
            offset += 4;
        } else {
            let byte = data[offset];
            let text = format_data_line(
                offset as u32,
                "dc.b",
                &format!("${byte:02X}"),
                options,
            );
            push_line(lines, line_num, options, text);
            offset += 1;
        }
    }
}

fn format_data_line(address: u32, directive: &str, value: &str, options: &ListingOptions) -> String {
    let mut parts = Vec::new();
    if options.show_addresses {
        parts.push(format!("{address:08X}"));
    }
    if options.show_hex {
        parts.push(format!("{:<20}", ""));
    }
    parts.push(format!("{directive:<8} {value}"));
    parts.join("  ")
}

fn format_bss_line(size: u32, options: &ListingOptions) -> String {
    let mut parts = Vec::new();
    if options.show_addresses {
        parts.push("00000000".to_string());
    }
    if options.show_hex {
        parts.push(format!("{:<20}", ""));
    }
    parts.push(format!("ds.b     {size}"));
    parts.join("  ")
}

fn detect_string(data: &[u8], offset: usize) -> Option<usize> {
    // Look for at least 4 printable ASCII bytes followed by a null
    let mut end = offset;
    while end < data.len() && data[end] != 0 {
        if !data[end].is_ascii_graphic() && data[end] != b' ' {
            return None;
        }
        end += 1;
    }
    if end - offset >= 4 && end < data.len() && data[end] == 0 {
        Some(end)
    } else {
        None
    }
}

fn push_line(
    lines: &mut Vec<ListingLine>,
    line_num: &mut u32,
    options: &ListingOptions,
    text: String,
) {
    let display_text = if options.show_line_numbers {
        format!("{:5}  {text}", *line_num)
    } else {
        text
    };
    lines.push(ListingLine {
        line_number: *line_num,
        text: display_text,
    });
    *line_num += 1;
}
