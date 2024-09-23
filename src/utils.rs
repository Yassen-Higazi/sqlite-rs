use anyhow::{anyhow, Result};

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

pub fn convert_u32_to_bytes(x: u32) -> [u8; 4] {
    let b1: u8 = ((x >> 24) & 0xff) as u8;
    let b2: u8 = ((x >> 16) & 0xff) as u8;
    let b3: u8 = ((x >> 8) & 0xff) as u8;
    let b4: u8 = (x & 0xff) as u8;

    [b1, b2, b3, b4].to_owned()
}
