// ShortURL (https://github.com/delight-im/ShortURL)
// Copyright (c) delight.im (https://www.delight.im/), andra.xyz (http://andra.xyz/)
// Licensed under the MIT License (https://opensource.org/licenses/MIT)

/// # ShortURL
/// Bijective conversion between natural numbers (IDs) (`usize`) and short strings (`String`)
///
/// short_url::encode(usize) takes an ID and turns it into a short string
/// short_url::decode(String) takes a short string and turns it into an ID
///
/// ## Features
/// * large alphabet (51 chars) and thus very short resulting strings
/// * proof against offensive words (removed 'a', 'e', 'i', 'o' and 'u')
/// * unambiguous (removed 'I', 'l', '1', 'O' and '0')
///
/// ## Example
/// * 123456789 <=> pgK8p

static ALPHABET: &'static str = "0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ-_";
static BASE: usize = 64;

pub fn encode(mut id: usize) -> String {
    let mut string: String = format!("");
    while id > 0 {
        string.push_str(&ALPHABET[(id % BASE)..(id % BASE + 1)]);
        id = id / BASE;
    }
    string.chars().rev().collect()
}

pub fn decode(string: &str) -> usize {
    let mut number: usize = 0;
    for c in string.chars() {
        number = number * BASE + ALPHABET.find(c).unwrap();
    }
    number
}

#[test]
fn test_short_url(){
    let value = encode(124);
    println!("{}", value);
    let value = decode("ASVasdfe");
    println!("{}", value);

}
