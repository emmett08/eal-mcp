//! Bounded newline-delimited session frames with exact rejected-byte retention.
use std::io::BufRead;

pub(super) struct ReadFailure {
    pub bytes: Vec<u8>,
    pub message: String,
}

pub(super) fn read_frame<R: BufRead>(
    reader: &mut R,
    maximum: usize,
) -> Result<Option<Vec<u8>>, ReadFailure> {
    let mut bytes = Vec::new();
    loop {
        let available = reader.fill_buf().map_err(|error| ReadFailure {
            bytes: bytes.clone(),
            message: format!("session input failed: {error}"),
        })?;
        if available.is_empty() {
            return if bytes.is_empty() {
                Ok(None)
            } else {
                Err(ReadFailure {
                    bytes,
                    message: "session input ended before the required frame newline".into(),
                })
            };
        }
        let count = available
            .iter()
            .position(|byte| *byte == b'\n')
            .map_or(available.len(), |index| index + 1)
            .min(maximum.saturating_add(1).saturating_sub(bytes.len()));
        bytes.extend_from_slice(&available[..count]);
        reader.consume(count);
        if bytes.len() > maximum {
            return Err(ReadFailure {
                bytes,
                message: "session frame exceeds its byte bound; retained prefix is truncated"
                    .into(),
            });
        }
        if bytes.last() == Some(&b'\n') {
            return Ok(Some(bytes));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn frames_require_newlines_and_bound_rejected_prefixes() {
        let mut reader = std::io::Cursor::new(b"{}\n[]\n");
        assert_eq!(
            read_frame(&mut reader, 3).unwrap_or_else(|_| panic!()),
            Some(b"{}\n".to_vec())
        );
        assert_eq!(
            read_frame(&mut reader, 3).unwrap_or_else(|_| panic!()),
            Some(b"[]\n".to_vec())
        );
        assert!(
            read_frame(&mut reader, 3)
                .unwrap_or_else(|_| panic!())
                .is_none()
        );
        let error = read_frame(&mut std::io::Cursor::new(b"abcdef\n"), 3)
            .err()
            .unwrap();
        assert_eq!(error.bytes, b"abcd");
        let error = read_frame(&mut std::io::Cursor::new(b"{}"), 3)
            .err()
            .unwrap();
        assert_eq!(error.bytes, b"{}");
        assert!(error.message.contains("newline"));
    }
}
