use anyhow::{anyhow, Context, Result};

pub fn parse_varint(bytes: &[u8]) -> Result<(u64, &[u8], usize)> {
    let mut result = 0;
    let mut shift = 0;
    let mut bytes_read = 0;
    let mut bs = bytes.iter().copied();

    loop {
        let byte = bs
            .next()
            .ok_or_else(|| anyhow!("Unexpected end of bytes"))?;

        bytes_read += 1;

        if bytes_read == 9 {
            result = (result << shift) | u64::from(byte);
            break;
        }

        result = (result << shift) | u64::from(byte & 0b0111_1111);

        shift += 7;

        if byte & 0b1000_0000 == 0 {
            break;
        }
    }

    Ok((result, &bytes[bytes_read..], bytes_read))
    // u32::from_be_bytes([num_vec[6], num_vec[7], num_vec[8], num_vec[9]])
}

pub fn read_be_utf16(slice: &[u8], size: usize) -> Result<String> {
    assert!(2 * size <= slice.len());

    let iter = (0..size)
        .map(|i| u16::from_be_bytes([slice[2 * i], slice[2 * i + 1]]));

    std::char::decode_utf16(iter).collect::<Result<String, _>>().with_context(|| "Could not decode utf16 le")
}

pub fn read_le_utf16(slice: &[u8], size: usize) -> Result<String> {
    assert!(2 * size <= slice.len());

    let iter = (0..size)
        .map(|i| u16::from_le_bytes([slice[2 * i], slice[2 * i + 1]]));

    std::char::decode_utf16(iter).collect::<Result<String, _>>().with_context(|| "Could not decode utf16 be")
}