// Copyright 2012-2013 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Base64 binary-to-text encoding

/// Contains configuration parameters for to_base64
pub struct Config {
    /// True to use the url-safe encoding format ('-' and '_'), false to use
    /// the standard encoding format ('+' and '/')
    pub url_safe: bool,
    /// True to pad output with '=' characters
    pub pad: bool,
    /// Some(len) to wrap lines at len, None to disable line wrapping
    pub line_length: Option<uint>
}

/// Configuration for RFC 4648 standard base64 encoding
pub static standard: Config =
    Config {url_safe: false, pad: true, line_length: None};

/// Configuration for RFC 4648 base64url encoding
pub static url_safe: Config =
    Config {url_safe: true, pad: false, line_length: None};

/// Configuration for RFC 2045 MIME base64 encoding
pub static mime: Config =
    Config {url_safe: false, pad: true, line_length: Some(76)};

static STANDARD_CHARS: [char, ..64] = [
    'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M',
    'N', 'O', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z',
    'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm',
    'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z',
    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', '+', '/'
];

static URLSAFE_CHARS: [char, ..64] = [
    'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M',
    'N', 'O', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z',
    'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm',
    'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z',
    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', '-', '_'
];

/// A trait for converting a value to base64 encoding.
pub trait ToBase64 {
    /// Converts the value of `self` to a base64 value following the specified
    /// format configuration, returning the owned string.
    fn to_base64(&self, config: Config) -> ~str;
}

impl<'self> ToBase64 for &'self [u8] {
    /**
     * Turn a vector of `u8` bytes into a base64 string.
     *
     * # Example
     *
     * ~~~ {.rust}
     * use std::base64::{ToBase64, standard};
     *
     * fn main () {
     *     let str = [52,32].to_base64(standard);
     *     println(fmt!("%s", str));
     * }
     * ~~~
     */
    fn to_base64(&self, config: Config) -> ~str {
        let chars = match config.url_safe {
            true => URLSAFE_CHARS,
            false => STANDARD_CHARS
        };

        let mut s = ~"";
        let mut i = 0;
        let mut cur_length = 0;
        let len = self.len();
        while i < len - (len % 3) {
            match config.line_length {
                Some(line_length) =>
                    if cur_length >= line_length {
                        s.push_str("\r\n");
                        cur_length = 0;
                    },
                None => ()
            }

            let n = (self[i] as u32) << 16 |
                    (self[i + 1] as u32) << 8 |
                    (self[i + 2] as u32);

            // This 24-bit number gets separated into four 6-bit numbers.
            s.push_char(chars[(n >> 18) & 63]);
            s.push_char(chars[(n >> 12) & 63]);
            s.push_char(chars[(n >> 6 ) & 63]);
            s.push_char(chars[n & 63]);

            cur_length += 4;
            i += 3;
        }

        if len % 3 != 0 {
            match config.line_length {
                Some(line_length) =>
                    if cur_length >= line_length {
                        s.push_str("\r\n");
                    },
                None => ()
            }
        }

        // Heh, would be cool if we knew this was exhaustive
        // (the dream of bounded integer types)
        match len % 3 {
            0 => (),
            1 => {
                let n = (self[i] as u32) << 16;
                s.push_char(chars[(n >> 18) & 63]);
                s.push_char(chars[(n >> 12) & 63]);
                if config.pad {
                    s.push_str("==");
                }
            }
            2 => {
                let n = (self[i] as u32) << 16 |
                    (self[i + 1u] as u32) << 8;
                s.push_char(chars[(n >> 18) & 63]);
                s.push_char(chars[(n >> 12) & 63]);
                s.push_char(chars[(n >> 6 ) & 63]);
                if config.pad {
                    s.push_char('=');
                }
            }
            _ => fail!("Algebra is broken, please alert the math police")
        }
        s
    }
}

impl<'self> ToBase64 for &'self str {
    /**
     * Convert any string (literal, `@`, `&`, or `~`) to base64 encoding.
     *
     *
     * # Example
     *
     * ~~~ {.rust}
     * use std::base64::{ToBase64, standard};
     *
     * fn main () {
     *     let str = "Hello, World".to_base64(standard);
     *     println(fmt!("%s",str));
     * }
     * ~~~
     *
     */
    fn to_base64(&self, config: Config) -> ~str {
        self.as_bytes().to_base64(config)
    }
}

/// A trait for converting from base64 encoded values.
pub trait FromBase64 {
    /// Converts the value of `self`, interpreted as base64 encoded data, into
    /// an owned vector of bytes, returning the vector.
    fn from_base64(&self) -> ~[u8];
}

impl<'self> FromBase64 for &'self [u8] {
    /**
     * Convert base64 `u8` vector into u8 byte values.
     * Every 4 encoded characters is converted into 3 octets, modulo padding.
     *
     * # Example
     *
     * ~~~ {.rust}
     * use std::base64::{ToBase64, FromBase64, standard};
     *
     * fn main () {
     *     let str = [52,32].to_base64(standard);
     *     println(fmt!("%s", str));
     *     let bytes = str.from_base64();
     *     println(fmt!("%?",bytes));
     * }
     * ~~~
     */
    fn from_base64(&self) -> ~[u8] {
        let mut r = ~[];
        let mut buf: u32 = 0;
        let mut modulus = 0;

        let mut it = self.iter();
        for it.advance |&byte| {
            let ch = byte as char;
            let val = byte as u32;

            match ch {
                'A'..'Z'  => buf |= val - 0x41,
                'a'..'z'  => buf |= val - 0x47,
                '0'..'9'  => buf |= val + 0x04,
                '+'|'-'   => buf |= 0x3E,
                '/'|'_'   => buf |= 0x3F,
                '\r'|'\n' => loop,
                '='       => break,
                _         => fail!("Invalid Base64 character")
            }

            buf <<= 6;
            modulus += 1;
            if modulus == 4 {
                modulus = 0;
                r.push((buf >> 22) as u8);
                r.push((buf >> 14) as u8);
                r.push((buf >> 6 ) as u8);
            }
        }

        if !it.all(|&byte| {byte as char == '='}) {
            fail!("Invalid Base64 character");
        }

        match modulus {
            2 => {
                r.push((buf >> 10) as u8);
            }
            3 => {
                r.push((buf >> 16) as u8);
                r.push((buf >> 8 ) as u8);
            }
            0 => (),
            _ => fail!("Invalid Base64 length")
        }

        r
    }
}

impl<'self> FromBase64 for &'self str {
    /**
     * Convert any base64 encoded string (literal, `@`, `&`, or `~`)
     * to the byte values it encodes.
     *
     * You can use the `from_bytes` function in `std::str`
     * to turn a `[u8]` into a string with characters corresponding to those
     * values.
     *
     * # Example
     *
     * This converts a string literal to base64 and back.
     *
     * ~~~ {.rust}
     * use std::base64::{ToBase64, FromBase64, standard};
     * use std::str;
     *
     * fn main () {
     *     let hello_str = "Hello, World".to_base64(standard);
     *     println(fmt!("%s",hello_str));
     *     let bytes = hello_str.from_base64();
     *     println(fmt!("%?",bytes));
     *     let result_str = str::from_bytes(bytes);
     *     println(fmt!("%s",result_str));
     * }
     * ~~~
     */
    fn from_base64(&self) -> ~[u8] {
        self.as_bytes().from_base64()
    }
}

#[test]
fn test_to_base64_basic() {
    assert_eq!("".to_base64(standard), ~"");
    assert_eq!("f".to_base64(standard), ~"Zg==");
    assert_eq!("fo".to_base64(standard), ~"Zm8=");
    assert_eq!("foo".to_base64(standard), ~"Zm9v");
    assert_eq!("foob".to_base64(standard), ~"Zm9vYg==");
    assert_eq!("fooba".to_base64(standard), ~"Zm9vYmE=");
    assert_eq!("foobar".to_base64(standard), ~"Zm9vYmFy");
}

#[test]
fn test_to_base64_line_break() {
    assert!(![0u8, 1000].to_base64(Config {line_length: None, ..standard})
        .contains("\r\n"));
    assert_eq!("foobar".to_base64(Config {line_length: Some(4), ..standard}),
        ~"Zm9v\r\nYmFy");
}

#[test]
fn test_to_base64_padding() {
    assert_eq!("f".to_base64(Config {pad: false, ..standard}), ~"Zg");
    assert_eq!("fo".to_base64(Config {pad: false, ..standard}), ~"Zm8");
}

#[test]
fn test_to_base64_url_safe() {
    assert_eq!([251, 255].to_base64(url_safe), ~"-_8");
    assert_eq!([251, 255].to_base64(standard), ~"+/8=");
}

#[test]
fn test_from_base64_basic() {
    assert_eq!("".from_base64(), "".as_bytes().to_owned());
    assert_eq!("Zg==".from_base64(), "f".as_bytes().to_owned());
    assert_eq!("Zm8=".from_base64(), "fo".as_bytes().to_owned());
    assert_eq!("Zm9v".from_base64(), "foo".as_bytes().to_owned());
    assert_eq!("Zm9vYg==".from_base64(), "foob".as_bytes().to_owned());
    assert_eq!("Zm9vYmE=".from_base64(), "fooba".as_bytes().to_owned());
    assert_eq!("Zm9vYmFy".from_base64(), "foobar".as_bytes().to_owned());
}

#[test]
fn test_from_base64_newlines() {
    assert_eq!("Zm9v\r\nYmFy".from_base64(), "foobar".as_bytes().to_owned());
}

#[test]
fn test_from_base64_urlsafe() {
    assert_eq!("-_8".from_base64(), "+/8=".from_base64());
}

#[test]
fn test_base64_random() {
    use std::rand::random;
    use std::vec;

    for 1000.times {
        let v: ~[u8] = do vec::build |push| {
            for 100.times {
                push(random());
            }
        };
        assert_eq!(v.to_base64(standard).from_base64(), v);
    }
}
