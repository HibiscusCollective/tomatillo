use crate::view::font::CompositeChar;
use crate::view::font::CharacterSet;
use std::ops::Range;

const HEIGHT: usize = 3;

const ZERO: CompositeChar<HEIGHT> = CompositeChar::new('0', [
    "┏┓",
    "┃┫",
    "┗┛",
]);
const ONE: CompositeChar<HEIGHT> = CompositeChar::new('1', [
    "┓ ",
    "┃ ",
    "┻ ",
]);

const TWO: CompositeChar<HEIGHT> = CompositeChar::new('2', [
    "┏┓",
    "┏┛",
    "┗━",
]);

const THREE: CompositeChar<HEIGHT> = CompositeChar::new('3', [
    "┏┓",
    " ┫",
    "┗┛",
]);

const FOUR: CompositeChar<HEIGHT> = CompositeChar::new('4', [
    "┏┓",
    "┃┃",
    "┗╋",
]);

const FIVE: CompositeChar<HEIGHT> = CompositeChar::new('5', [
    "┏━",
    "┗┓",
    "┗┛",
]);

const SIX: CompositeChar<HEIGHT> = CompositeChar::new('6', [
    "┏┓",
    "┣┓",
    "┗┛",
]);

const SEVEN: CompositeChar<HEIGHT> = CompositeChar::new('7', [
    "━┓",
    " ┃",
    " ╹",
]);

const EIGHT: CompositeChar<HEIGHT> = CompositeChar::new('8', [
    "┏┓",
    "┣┫",
    "┗┛",
]);

const NINE: CompositeChar<HEIGHT> = CompositeChar::new('9', [
    "┏┓",
    "┗┫",
    "┗┛",
]);

const COLON: CompositeChar<HEIGHT> = CompositeChar::new(':', [
    " ",
    "•",
    "•",
]);

pub struct Templar;

#[cfg(test)]
pub const TEST_COLON: CompositeChar<HEIGHT> = COLON;

impl CharacterSet<&'static CompositeChar<HEIGHT>> for Templar {
    fn height_range(&self) -> Range<usize> {
        (0..HEIGHT).into()
    }

    fn get(&self, index: char) -> Option<&'static CompositeChar<HEIGHT>> {
        match index {
            '0' => Some(&ZERO),
            '1' => Some(&ONE),
            '2' => Some(&TWO),
            '3' => Some(&THREE),
            '4' => Some(&FOUR),
            '5' => Some(&FIVE),
            '6' => Some(&SIX),
            '7' => Some(&SEVEN),
            '8' => Some(&EIGHT),
            '9' => Some(&NINE),
            ':' => Some(&COLON),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_character_set() {
        let font = Templar;
        assert_eq!(font.height_range(), (0..HEIGHT).into());
        assert_eq!(font.get('0'), Some(&ZERO));
        assert_eq!(font.get('1'), Some(&ONE));
        assert_eq!(font.get('2'), Some(&TWO));
        assert_eq!(font.get('3'), Some(&THREE));
        assert_eq!(font.get('4'), Some(&FOUR));
        assert_eq!(font.get('5'), Some(&FIVE));
        assert_eq!(font.get('6'), Some(&SIX));
        assert_eq!(font.get('7'), Some(&SEVEN));
        assert_eq!(font.get('8'), Some(&EIGHT));
        assert_eq!(font.get('9'), Some(&NINE));
        assert_eq!(font.get(':'), Some(&COLON));
        assert_eq!(font.get('a'), None);
    }
}