#![allow(dead_code)]

pub const STARTALPHANUM: i16 = 2822;
pub const ENDALPHANUM: i16 = 2915;
pub const BIGALPHANUM: i16 = 2940;
pub const MINIFONT: i16 = 3072;
pub const THREE_DIGIT_BASE: i16 = 2472;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DukeFont {
    BigRed,
    SmallBlue,
    Mini,
    DigitalNumbers,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GlyphDrawCall {
    pub tile_id: i16,
    pub x: i32,
    pub y: i32,
}

pub struct DukeFontRenderer;

impl DukeFontRenderer {
    pub fn layout_text(font: DukeFont, text: &str, start_x: i32, start_y: i32) -> Vec<GlyphDrawCall> {
        let mut calls = Vec::new();
        let mut cur_x = start_x;

        for ch in text.chars() {
            if ch == ' ' {
                cur_x += match font {
                    DukeFont::BigRed => 12,
                    DukeFont::SmallBlue => 6,
                    DukeFont::Mini => 4,
                    DukeFont::DigitalNumbers => 8,
                };
                continue;
            }

            let (tile_id, advance) = match font {
                DukeFont::BigRed => {
                    let upper = ch.to_ascii_uppercase();
                    if upper >= 'A' && upper <= 'Z' {
                        (BIGALPHANUM + (upper as i16 - 'A' as i16), 14)
                    } else if upper >= '0' && upper <= '9' {
                        (BIGALPHANUM + 52 + (upper as i16 - '0' as i16), 12)
                    } else if upper == ':' {
                        (BIGALPHANUM + 62, 8)
                    } else if upper == '/' {
                        (BIGALPHANUM + 63, 10)
                    } else if upper == '-' {
                        (BIGALPHANUM - 11, 10)
                    } else {
                        (BIGALPHANUM, 12)
                    }
                }
                DukeFont::SmallBlue => {
                    let code = ch as i16;
                    if code >= 33 && code <= 126 {
                        (STARTALPHANUM + (code - 33), 8)
                    } else {
                        (STARTALPHANUM, 8)
                    }
                }
                DukeFont::Mini => {
                    let code = ch as i16;
                    if code >= 33 && code <= 126 {
                        (MINIFONT + (code - 33), 5)
                    } else {
                        (MINIFONT, 5)
                    }
                }
                DukeFont::DigitalNumbers => {
                    if ch >= '0' && ch <= '9' {
                        (THREE_DIGIT_BASE + (ch as i16 - '0' as i16), 9)
                    } else if ch == '%' {
                        (THREE_DIGIT_BASE + 10, 9)
                    } else {
                        (THREE_DIGIT_BASE, 9)
                    }
                }
            };

            calls.push(GlyphDrawCall {
                tile_id,
                x: cur_x,
                y: start_y,
            });

            cur_x += advance;
        }

        calls
    }
}
