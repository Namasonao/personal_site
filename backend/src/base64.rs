static BASE64_STANDARD: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

#[inline(always)]
fn index(i: u8) -> char {
    BASE64_STANDARD[i as usize] as char
}

enum Decoded {
    Ok(u8),
    Padding,
    Fault,
}
fn reverse(c: char) -> Decoded {
    let cu = c as u8;
    if cu >= 'A' as u8 && cu <= 'Z' as u8 {
        return Decoded::Ok(cu - ('A' as u8));
    }

    if cu >= 'a' as u8 && cu <= 'z' as u8 {
        return Decoded::Ok(cu - ('a' as u8) + 26);
    }

    if cu >= '0' as u8 && cu <= '9' as u8 {
        return Decoded::Ok(cu - ('0' as u8) + 52);
    }
    match c {
        '+' => Decoded::Ok(62),
        '/' => Decoded::Ok(63),
        '=' => Decoded::Padding,
        _ => Decoded::Fault,
    }
}

pub fn encode(bytes: &[u8]) -> String {
    let mut s = String::new();
    let mut bytes = bytes.iter();
    loop {
        let b1 = match bytes.next() {
            Some(b) => b,
            None => break,
        };
        let bits = b1 >> 2;
        s.push(index(bits));

        let b2 = match bytes.next() {
            Some(b) => b,
            None => {
                let bits = (b1 & 0b11) << 4;
                s.push(index(bits));
                s.push('=');
                s.push('=');
                break;
            }
        };
        let bits = (b1 & 0b11) << 4;
        let bits = bits | (b2 >> 4);
        s.push(index(bits));

        let b3 = match bytes.next() {
            Some(b) => b,
            None => {
                let bits = (b2 & 0b1111) << 2;
                s.push(index(bits));
                s.push('=');
                break;
            }
        };

        let bits = (b2 & 0b1111) << 2;
        let bits = bits | (b3 >> 6);
        s.push(index(bits));
        let bits = b3 & 0b111111;
        s.push(index(bits));
    }
    s
}

#[derive(Debug)]
pub enum DecodeError {
    SkillIssue,
}

impl std::fmt::Display for DecodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        todo!()
    }
}
impl std::error::Error for DecodeError {}

pub fn decode(string: &str) -> Result<Vec<u8>, DecodeError> {
    let mut ret = Vec::new();
    let mut string = string.chars();
    loop {
        let c = match string.next() {
            Some(c) => c,
            None => break,
        };
        let i1 = match reverse(c) {
            Decoded::Ok(i) => i,
            Decoded::Padding => break,
            Decoded::Fault => return Err(DecodeError::SkillIssue),
        };

        let c = match string.next() {
            Some(c) => c,
            None => break,
        };
        let i2 = match reverse(c) {
            Decoded::Ok(i) => i,
            Decoded::Padding => break,
            Decoded::Fault => return Err(DecodeError::SkillIssue),
        };
        ret.push((i1 << 2) | (i2 >> 4));
        let i2 = i2 & 0b1111;

        let c = match string.next() {
            Some(c) => c,
            None => break,
        };
        let i3 = match reverse(c) {
            Decoded::Ok(i) => i,
            Decoded::Padding => break,
            Decoded::Fault => return Err(DecodeError::SkillIssue),
        };

        ret.push((i2 << 4) | (i3 >> 2));
        let i3 = i3 & 0b11;

        let c = match string.next() {
            Some(c) => c,
            None => break,
        };
        let i4 = match reverse(c) {
            Decoded::Ok(i) => i,
            Decoded::Padding => break,
            Decoded::Fault => return Err(DecodeError::SkillIssue),
        };

        ret.push((i3 << 6) | (i4 >> 0));
    }
    Ok(ret)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_abcd_encode() {
        let result = encode(b"abcd");
        let expected = "YWJjZA==".to_string();
        assert_eq!(result, expected);
    }
    #[test]
    #[test]
    fn test_dcba() {
        let result = encode(b"dcba");
        let expected = "ZGNiYQ==".to_string();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_long() {
        let result = encode(b"aaabbbcccddd");
        let expected = "YWFhYmJiY2NjZGRk".to_string();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_random() {
        let result = encode(b"askdzhfyxclkjvpoiuwioqejrkaospldfvpyoxcjvmoaksdj");
        let expected =
            "YXNrZHpoZnl4Y2xranZwb2l1d2lvcWVqcmthb3NwbGRmdnB5b3hjanZtb2Frc2Rq".to_string();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_decode_1() {
        let expected = b"a";
        let result = decode(&("YQ==".to_string())).unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_decode_2() {
        let expected = b"ab";
        let result = decode(&("YWI=".to_string())).unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_decode_3() {
        let expected = b"abc";
        let result = decode(&("YWJj".to_string())).unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_decode_4() {
        let expected = b"abcd";
        let result = decode(&("YWJjZA==".to_string())).unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_decode_random() {
        let expected = b"askdzhfyxclkjvpoiuwioqejrkaospldfvpyoxcjvmoaksdj";
        let result = decode(
            &("YXNrZHpoZnl4Y2xranZwb2l1d2lvcWVqcmthb3NwbGRmdnB5b3hjanZtb2Frc2Rq".to_string()),
        )
        .unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_decode_no_padding() {
        let expected = b"abcd";
        let result = decode(&("YWJjZA".to_string())).unwrap();
        assert_eq!(result, expected);
    }
}
