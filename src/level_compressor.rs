extern crate brotli;
use std::io::{Read, Write};

// Compresses the given string using the highest setting for brotli
// and returns a base64 encoded string
pub fn compress_level(level_string: &str) -> String {
    let level_bytes = level_string.as_bytes().to_vec();
    let mut compressed_bytes = Vec::new();
    {
        let mut compressed_writer: std::io::Cursor<&mut Vec<u8>> =
            std::io::Cursor::new(&mut compressed_bytes);
        let mut writer = brotli::CompressorWriter::new(&mut compressed_writer, 4096, 11, 22);
        writer
            .write_all(&level_bytes)
            .expect("could not write to compressed writer");
    }
    base_62::encode(&compressed_bytes)
}

pub fn decompress_level(compressed_level: &str) -> Result<String, String> {
    let compressed_bytes = base_62::decode(compressed_level).map_err(|e| e.to_string())?;
    let mut decompressed_bytes = Vec::new();
    {
        let mut decompressed_writer: std::io::Cursor<&mut Vec<u8>> =
            std::io::Cursor::new(&mut decompressed_bytes);
        let mut writer = brotli::DecompressorWriter::new(&mut decompressed_writer, 4096);
        writer
            .write_all(&compressed_bytes)
            .map_err(|e| e.to_string())?;
    }
    String::from_utf8(decompressed_bytes).map_err(|e| e.to_string())
}
