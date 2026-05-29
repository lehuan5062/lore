// SPDX-FileCopyrightText: 2026 Epic Games, Inc.
// SPDX-License-Identifier: MIT
use std::path::Path;

use tokio::io;
use tokio::io::AsyncBufReadExt;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncSeekExt;

use crate::util::encoding::decode_text_for_display;
use crate::util::encoding::is_utf16_bom;

async fn infer_into_buffer(path: &Path, max: u64) -> io::Result<Vec<u8>> {
    let mut file = tokio::fs::File::open(path).await?;

    let metadata = file.metadata().await?;

    let to_read: usize = std::cmp::min(max, metadata.len()) as usize;
    if to_read == 0 {
        return Ok(vec![]);
    }

    let mut buffer = vec![0u8; to_read];
    file.read_exact(&mut buffer).await?;

    Ok(buffer)
}

pub fn infer_type_by_slice(buffer: &[u8]) -> Option<&str> {
    // Is it containing a magic marker known to the infer crate?
    if let Some(kind) = infer::get(buffer) {
        return Some(kind.mime_type());
    }

    None
}

pub fn infer_is_utf8_by_slice(buffer: &[u8]) -> bool {
    // Is it containing a valid utf8 string?
    std::str::from_utf8(buffer).is_ok()
}

pub fn infer_is_upackage_by_slice(buffer: &[u8]) -> bool {
    // Is it containing an unreal magic marker?
    if buffer.len() >= 4 {
        let package_file_tag = vec![0x9E, 0x2A, 0x83, 0xC1];
        if buffer[..3] == package_file_tag {
            return true;
        }

        let package_file_tag_swapped = vec![0xC1, 0x83, 0x2A, 0x9E];
        if buffer[..3] == package_file_tag_swapped {
            return true;
        }
    }

    false
}

pub fn infer_is_diffable_by_slice(buffer: &[u8]) -> bool {
    // Check if it's an unreal asset.
    if infer_is_upackage_by_slice(buffer) {
        return false;
    }

    // Check if it's a non-diffable mime type.
    if let Some(mime_type) = infer_type_by_slice(buffer)
        && mime_type != "text/html"
        && mime_type != "text/x-shellscript"
        && mime_type != "text/xml"
    {
        return false;
    }

    // Check if it's a utf8 string.
    // Do this on substrings to disregard utf8 truncation at the end of the buffer.
    for n in 0..3 {
        let len = buffer.len() - n;
        if len == 0 {
            return false;
        }

        if infer_is_utf8_by_slice(&buffer[..len]) {
            return true;
        }
    }

    false
}

/// Check if conflict markers are present in line.
///
/// # Arguments
///
/// * `line` - A &str that holds the text to inspect.
///
/// # Return value
///
/// * `true` if there are conflict markers in `line`.
/// * `false` if there are no conflict markers in `line`.
///
fn infer_is_conflicted_by_line(line: &str) -> bool {
    if line.starts_with("||||||| ") {
        return true;
    }
    if line.starts_with("<<<<<<< ") {
        return true;
    }
    if line.starts_with(">>>>>>> ") {
        return true;
    }

    false
}

/// Check if conflict markers are present in text.
///
/// # Arguments
///
/// * `text` - A &str that holds the text to inspect.
///
/// # Return value
///
/// * `true` if there are conflict markers in `text`.
/// * `false` if there are no conflict markers in `text`.
///
pub fn infer_is_conflicted_by_str(text: &str) -> bool {
    for line in text.lines() {
        if infer_is_conflicted_by_line(line) {
            return true;
        }
    }

    false
}

/// Check if conflict markers are present in file.
///
/// # Arguments
///
/// * `path` - A &Path that holds the path to inspect.
///
/// # Return value
///
/// * `Ok(true)` if there are conflict markers in `path`.
/// * `Ok(false)` if there are no conflict markers in `path`.
/// * `Ok(false)` if `path` does not exist.
/// * `Error()` if an I/O error occurs.
///
/// # Notes
///
/// Streams line-by-line for UTF-8 (the hot path for large generated text).
/// UTF-16 BOM-prefixed files — which `BufReader::lines` cannot decode — are
/// read whole and routed through [`decode_text_for_display`].
pub async fn infer_is_conflicted_by_path(path: &Path) -> Result<bool, std::io::Error> {
    if tokio::fs::metadata(path).await.is_err() {
        return Ok(false);
    }

    let mut file = tokio::fs::File::open(path).await?;

    let mut bom = [0u8; 2];
    let bom_len = file.read(&mut bom).await?;
    if bom_len == 2 && is_utf16_bom(&bom) {
        let mut bytes = bom.to_vec();
        file.read_to_end(&mut bytes).await?;
        return Ok(infer_is_conflicted_by_str(&decode_text_for_display(&bytes)));
    }

    file.seek(std::io::SeekFrom::Start(0)).await?;
    let reader = tokio::io::BufReader::new(file);

    let mut lines = reader.lines();
    while let Ok(Some(line)) = lines.next_line().await {
        if infer_is_conflicted_by_line(line.as_str()) {
            return Ok(true);
        }
    }
    Ok(false)
}

/// Checks if a file contains diffable data.
///
/// # Arguments
///
/// * `path` - An absolute path to the file to check.
pub async fn infer_is_diffable_by_path(path: &Path) -> io::Result<bool> {
    // Inspect the first 4 KiB of the file at most.
    let buffer = infer_into_buffer(path, 4 * 1024).await?;
    Ok(infer_is_diffable_by_slice(buffer.as_slice()))
}

#[cfg(test)]
mod tests {
    use super::infer_is_diffable_by_slice;
    use super::infer_is_utf8_by_slice;

    #[test]
    fn is_utf8() {
        let one_sparkle_heart = vec![240, 159, 146, 150];
        assert!(
            infer_is_utf8_by_slice(&one_sparkle_heart),
            "One sparkle heart is UTF-8"
        );

        let two_sparkle_hearts = vec![240, 159, 146, 150, 240, 159, 146, 150];
        assert!(
            infer_is_utf8_by_slice(&two_sparkle_hearts),
            "Two sparkle hearts are UTF-8"
        );

        let two_sparkle_hearts_truncated = vec![240, 159, 146, 150, 240, 159, 146];
        assert!(
            !infer_is_utf8_by_slice(&two_sparkle_hearts_truncated),
            "Two sparkle hearts with the last one truncated does not count as UTF-8"
        );

        let three_sparkle_heart_invalid = vec![240, 159, 146, 150, 240, 159, 240, 159, 146, 150];
        assert!(
            !infer_is_utf8_by_slice(&three_sparkle_heart_invalid),
            "Three sparkle hearts with the middle one being invalid does not count as UTF-8"
        );
    }

    #[test]
    fn non_diffable_utf16_le_bom() {
        let mut bytes = vec![0xFF, 0xFE];
        bytes.extend("Hello\nWorld\n".encode_utf16().flat_map(u16::to_le_bytes));
        assert!(
            !infer_is_diffable_by_slice(&bytes),
            "UTF-16 LE BOM must be non-diffable so merge falls into the binary-conflict path that preserves bytes"
        );
    }

    #[test]
    fn non_diffable_utf16_be_bom() {
        let mut bytes = vec![0xFE, 0xFF];
        bytes.extend("Hello\nWorld\n".encode_utf16().flat_map(u16::to_be_bytes));
        assert!(
            !infer_is_diffable_by_slice(&bytes),
            "UTF-16 BE BOM must be non-diffable so merge falls into the binary-conflict path that preserves bytes"
        );
    }
}
