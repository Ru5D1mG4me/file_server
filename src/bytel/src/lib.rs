use std::io::{Error, ErrorKind};
use std::result::Result;

#[derive(Debug)]
pub enum BytelError {
    ASCIIParseError,
    NumberParseError,
    UIntOverflow,
}

impl From<BytelError> for Error {
    fn from(error: BytelError) -> Error {
        match error {
            BytelError::ASCIIParseError => Error::new(ErrorKind::Other, "Not printable or Non-ASCII characters"),
            BytelError::NumberParseError => Error::new(ErrorKind::Other, "Not number symbol"),
            BytelError::UIntOverflow => Error::new(ErrorKind::Other, "Unsigned integer overflow"),
        }
    }
}

// pub fn parse_u16(bytes: &[u8]) -> Result<u16, BytelError> {
//     let mut i = (bytes.len() - 1) as u8;
//     let mut result: u64 = 0;
//     for byte in bytes {
//         if 0x30 <= *byte && *byte <= 0x39 {
//             result += (((*byte - 0x30) as u32) * 10_u32.pow(i as u32)) as u64;
//
//             if i != 0 {
//                 i -= 1;
//             }
//             continue;
//         }
//
//         return Err(BytelError::ParseError);
//     }
//
//     if result > 0xFFFF {
//         return Ok(0xFFFF)
//     }
//
//     Ok((result & 0xFFFF) as u16)
// }

pub fn parse_u64(bytes: &[u8]) -> Result<u64, BytelError> {
    if bytes.len() > 20 || (bytes.len() == 20 && bytes[0] > 0x31) {
        return Err(BytelError::UIntOverflow);
    }

    let mut i = (bytes.len() - 1) as u8;
    let mut result: u64 = 0;
    for byte in bytes {
        if 0x30 <= *byte && *byte <= 0x39 {
            result += ((*byte - 0x30) as u64) * 10_u64.pow(i as u32);

            if i != 0 {
                i -= 1;
            }
            continue;
        }

        return Err(BytelError::NumberParseError);
    }

    Ok(result)
}

pub fn u64_to_str(value: u64) -> String {
    let mut symbols: Vec<u8> = Vec::with_capacity(20);

    let mut num = value;
    let mut len: u8 = 0;
    while num != 0 {
        symbols.push(0x30 + (num % 10) as u8);
        num /= 10;

        len += 1;
    }

    let mut result: String = String::with_capacity(len as usize);
    for i in (0..len).rev() {
        result.push(symbols[i as usize] as char);
    }

    result
}

pub fn parse_str(bytes: &[u8]) -> Result<String, BytelError> {
    let mut result: String = String::with_capacity(bytes.len());
    for byte in bytes {
        if 0x20 <= *byte && *byte < 0x7E {
            result.push(*byte as char);
            continue;
        }

        return Err(BytelError::ASCIIParseError);
    }

    Ok(result)
}