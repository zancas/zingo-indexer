//! Utility functions for Nym-Proxy

use crate::blockcache::utils::CompactSize;
use std::io::Cursor;

use crate::blockcache::utils::{read_bytes, ParseError};

/// Reads a RPC method name from a Vec<u8> and returns this as a string along with the remaining data in the input.
fn read_nym_method(data: &[u8]) -> Result<(String, &[u8]), ParseError> {
    let mut cursor = Cursor::new(data);
    let method_len = CompactSize::read(&mut cursor)? as usize;
    let method = String::from_utf8(read_bytes(&mut cursor, method_len, "failed to read")?)?;
    Ok((method, &data[cursor.position() as usize..]))
}

/// Check the body of the request is the correct length.
fn check_nym_body(data: &[u8]) -> Result<&[u8], ParseError> {
    let mut cursor = Cursor::new(data);
    let body_len = CompactSize::read(&mut cursor)? as usize;
    if &body_len != &data[cursor.position() as usize..].len() {
        return Err(ParseError::InvalidData(
            "Incorrect request body size read.".to_string(),
        ));
    };
    Ok(&data[cursor.position() as usize..])
}

/// Extracts metadata from a NymRequest.
///
/// Returns [ID, Method, RequestData].
pub fn read_nym_request_data(data: &[u8]) -> Result<(u64, String, &[u8]), ParseError> {
    let mut cursor = Cursor::new(data);
    let id = CompactSize::read(&mut cursor)?;
    let (method, data) = read_nym_method(&data[cursor.position() as usize..])?;
    let body = check_nym_body(data)?;
    Ok((id, method, body))
}
