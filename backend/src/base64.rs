
/*
Value Encoding  Value Encoding  Value Encoding  Value Encoding
         0 A            17 R            34 i            51 z
         1 B            18 S            35 j            52 0
         2 C            19 T            36 k            53 1
         3 D            20 U            37 l            54 2
         4 E            21 V            38 m            55 3
         5 F            22 W            39 n            56 4
         6 G            23 X            40 o            57 5
         7 H            24 Y            41 p            58 6
         8 I            25 Z            42 q            59 7
         9 J            26 a            43 r            60 8
        10 K            27 b            44 s            61 9
        11 L            28 c            45 t            62 +
        12 M            29 d            46 u            63 /
        13 N            30 e            47 v
        14 O            31 f            48 w         (pad) =
        15 P            32 g            49 x
        16 Q            33 h            50 y
        */
static BASE64_STANDARD: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

#[inline(always)]
fn index(i: u8) -> char {
    BASE64_STANDARD[i as usize] as char
}

enum Decoded {
    Ok(u8),
    Padding,
    Fault
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
                break
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
        let expected = "YXNrZHpoZnl4Y2xranZwb2l1d2lvcWVqcmthb3NwbGRmdnB5b3hjanZtb2Frc2Rq".to_string();
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
        let result = decode(&("YXNrZHpoZnl4Y2xranZwb2l1d2lvcWVqcmthb3NwbGRmdnB5b3hjanZtb2Frc2Rq".to_string())).unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_decode_no_padding() {
        let expected = b"abcd";
        let result = decode(&("YWJjZA".to_string())).unwrap();
        assert_eq!(result, expected);
    }
}
