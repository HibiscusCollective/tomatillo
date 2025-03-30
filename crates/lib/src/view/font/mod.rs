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

pub trait Font {
    type CHAR: Character;

    fn height_range(&self) -> Range<usize>;

    fn get(&self, index: char) -> Option<Self::CHAR>;
}

pub trait Character: Debug + Eq + PartialEq {
    fn width(&self) -> usize;
    fn height(&self) -> usize;
    
    fn draw_line(&self, writer: &mut impl Write, line: usize);
}

pub struct NoopFont;

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct CompositeChar<'a, const HEIGHT: usize>(char, [&'a str; HEIGHT]);

impl Font for NoopFont {
    type CHAR = char;

    fn height_range(&self) -> Range<usize> {
        (0..1).into()
    }

    fn get(&self, index: char) -> Option<char> {
        Some(index)
    }
}

impl<'a, const HEIGHT: usize> Character for CompositeChar<'a, HEIGHT> {
    fn width(&self) -> usize {
        todo!()
    }

    fn height(&self) -> usize {
        HEIGHT
    }

    fn draw_line(&self, writer: &mut impl Write, line: usize) {
        writer.write_str(self.1[line]).unwrap(); // Handle errors
        if line < HEIGHT-1 {
            writer.write_char('\n').unwrap(); // Handle errors
        }
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
    fn test_retrieve_character(#[case] charset: impl Font, #[case] index: char, #[case] expected: impl ToString) {
        let actual = charset.get(index).expect("should have found character");

        let mut writer = String::new();
        for i in charset.height_range() {
            actual.draw_line(&mut writer, i);
        }

        assert_eq!(writer, expected.to_string());
    }

    impl<const HEIGHT: usize> ToString for CompositeChar<'_, HEIGHT> {  
        fn to_string(&self) -> String {
            self.1.join("\n").to_string()
        }
    }
}