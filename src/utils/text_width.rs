/// From https://github.com/adambisek/string-pixel-width/blob/master/src/widthsMap.js
///
/// MIT License
///
/// Copyright (c) 2023 Adam Ernst Bisek
///
/// Permission is hereby granted, free of charge, to any person obtaining a copy
/// of this software and associated documentation files (the "Software"), to deal
/// in the Software without restriction, including without limitation the rights
/// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
/// copies of the Software, and to permit persons to whom the Software is
/// furnished to do so, subject to the following conditions:
///
/// The above copyright notice and this permission notice shall be included in all
/// copies or substantial portions of the Software.
///
/// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
/// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
/// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
/// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
/// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
/// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
/// SOFTWARE.
const HELVETICA_CHAR_WIDTH: phf::Map<char, u32> = phf::phf_map! {
    '0' => 56,
    '1' => 56,
    '2' => 56,
    '3' => 56,
    '4' => 56,
    '5' => 56,
    '6' => 56,
    '7' => 56,
    '8' => 56,
    '9' => 56,
    ' ' => 28,
    '!' => 28,
    '"' => 35,
    '#' => 56,
    '$' => 56,
    '%' => 89,
    '&' => 67,
    '\'' => 19,
    '(' => 33,
    ')' => 33,
    '*' => 39,
    '+' => 58,
    ',' => 28,
    '-' => 33,
    '.' => 28,
    '/' => 28,
    ':' => 28,
    ';' => 28,
    '<' => 58,
    '=' => 58,
    '>' => 58,
    '?' => 56,
    '@' => 102,
    'A' => 67,
    'B' => 67,
    'C' => 72,
    'D' => 72,
    'E' => 67,
    'F' => 61,
    'G' => 78,
    'H' => 72,
    'I' => 28,
    'J' => 50,
    'K' => 67,
    'L' => 56,
    'M' => 83,
    'N' => 72,
    'O' => 78,
    'P' => 67,
    'Q' => 78,
    'R' => 72,
    'S' => 67,
    'T' => 61,
    'U' => 72,
    'V' => 67,
    'W' => 94,
    'X' => 67,
    'Y' => 67,
    'Z' => 61,
    '[' => 28,
    '\\' => 28,
    ']' => 28,
    '^' => 47,
    '_' => 56,
    '`' => 33,
    'a' => 56,
    'b' => 56,
    'c' => 50,
    'd' => 56,
    'e' => 56,
    'f' => 28,
    'g' => 56,
    'h' => 56,
    'i' => 22,
    'j' => 22,
    'k' => 50,
    'l' => 22,
    'm' => 83,
    'n' => 56,
    'o' => 56,
    'p' => 56,
    'q' => 56,
    'r' => 33,
    's' => 50,
    't' => 28,
    'u' => 56,
    'v' => 50,
    'w' => 72,
    'x' => 50,
    'y' => 50,
    'z' => 50,
    '{' => 33,
    '|' => 26,
    '}' => 33,
    '~' => 58,
};

pub fn helvetica_width(text: &str, font_size: f32) -> f32 {
  text
    .chars()
    .map(|c| {
      *HELVETICA_CHAR_WIDTH
        .get(&c)
        .unwrap_or(HELVETICA_CHAR_WIDTH.get(&'x').unwrap()) as f32
        * (font_size / 100.0)
    })
    .sum()
}
