use std::{fmt::{Debug, Write}, ops::Range};

use ansi_shadow::AnsiShadow;
use electronic::Electronic;
use templar::Templar;

mod ansi_shadow;
mod electronic;
mod templar;

pub const NONE: NoopFont = NoopFont;
pub const ANSI_SHADOW: AnsiShadow = AnsiShadow;
pub const ELECTRONIC: Electronic = Electronic;
pub const TEMPLAR: Templar = Templar;

pub trait CharacterSet<T: Character> {
    fn height_range(&self) -> Range<usize>;
    fn get(&self, index: char) -> Option<T>;
}

pub trait Character: Debug + Eq + PartialEq + ToString {
    fn width(&self) -> usize;
    fn height(&self) -> usize;
    
    fn draw_line(&self, writer: &mut impl Write, line: usize);
}

pub struct NoopFont;

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct CompositeChar<const HEIGHT: usize> {
    id: char,
    lines: [&'static str; HEIGHT],
}

impl<const HEIGHT: usize> CompositeChar<HEIGHT> {
    const fn new(id: char, lines: [&'static str; HEIGHT]) -> Self {
        Self { id, lines }
    }
}

impl CharacterSet<char> for NoopFont {
    fn height_range(&self) -> Range<usize> {
        (0..1).into()
    }

    fn get(&self, index: char) -> Option<char> {
        Some(index)
    }
}

impl<const HEIGHT: usize> Character for &'static CompositeChar<HEIGHT> {
    fn width(&self) -> usize {
        todo!()
    }

    fn height(&self) -> usize {
        HEIGHT
    }

    fn draw_line(&self, writer: &mut impl Write, line: usize) {
        write!(writer, "{}", self.lines[line]).unwrap();
        if line < HEIGHT-1 {
            writer.write_char('\n').unwrap();
        }
    }
}

impl<const HEIGHT: usize> ToString for CompositeChar<HEIGHT> {
    fn to_string(&self) -> String {
        self.lines.join("\n").to_string()
    }
}

impl<const HEIGHT: usize> ToString for &'static CompositeChar<HEIGHT> {
    fn to_string(&self) -> String {
        (*self).to_string()
    }
}

impl Character for char {
    fn width(&self) -> usize {
        todo!()
    }

    fn height(&self) -> usize { 1 }

    fn draw_line(&self, writer: &mut impl Write, _: usize) {
        writer.write_char(*self).unwrap(); // TODO: handle errors
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;
    
    use super::*;

    #[rstest]
    #[case::none(NONE, '7', "7")]
    #[case::ansi_shadow(ANSI_SHADOW, '4', ansi_shadow::TEST_FOUR)]
    #[case::electronic(ELECTRONIC, '9', electronic::TEST_NINE)]
    #[case::templar(TEMPLAR, ':', templar::TEST_COLON)]
    fn test_retrieve_character<T: Character>(#[case] charset: impl CharacterSet<T>, #[case] index: char, #[case] expected: impl ToString) {
        let actual = charset.get(index).expect("should have found character");

        let mut writer = String::new();
        for i in charset.height_range() {
            actual.draw_line(&mut writer, i);
        }

        assert_eq!(writer, expected.to_string());
    }
}