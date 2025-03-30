use std::ops::Range;

use super::{Font, CompositeChar};

const HEIGHT: usize = 6;

const ZERO: CompositeChar<HEIGHT> = CompositeChar('0', [
    " ██████╗ ",  
    "██╔═████╗",
    "██║██╔██║",
    "████╔╝██║",
    "╚██████╔╝",
    " ╚═════╝ ",
]);
const ONE: CompositeChar<HEIGHT> = CompositeChar('1', [
    " ██╗",
    "███║",
    "╚██║",
    " ██║",
    " ██║",
    " ╚═╝",
]);

const TWO: CompositeChar<HEIGHT> = CompositeChar('2', [
    "██████╗ ",
    "╚════██╗",
    " █████╔╝",
    "██╔═══╝ ",
    "███████╗",
    "╚══════╝",
]);

const THREE: CompositeChar<HEIGHT> = CompositeChar('3', [
    "██████╗ ",
    "╚════██╗",
    " █████╔╝",
    " ╚═══██╗",
    "██████╔╝",
    "╚═════╝ ",
]);

const FOUR: CompositeChar<HEIGHT> = CompositeChar('4', [
    "██╗  ██╗",
    "██║  ██║",
    "███████║",
    "╚════██║",
    "     ██║",
    "     ╚═╝",
]);

const FIVE: CompositeChar<HEIGHT> = CompositeChar('5', [
    "███████╗",
    "██╔════╝",
    "███████╗",
    "╚════██║",
    "███████║",
    "╚══════╝",
]);

const SIX: CompositeChar<HEIGHT> = CompositeChar('6', [
    " ██████╗ ",
    "██╔════╝ ",
    "███████╗ ",
    "██╔═══██╗",
    "╚██████╔╝",
    " ╚═════╝ ",
]);

const SEVEN: CompositeChar<HEIGHT> = CompositeChar('7', [
    "███████╗",
    "╚════██║",
    "    ██╔╝",
    "   ██╔╝ ",
    "   ██║  ",
    "   ╚═╝  ",
]);

const EIGHT: CompositeChar<HEIGHT> = CompositeChar('8', [
    " █████╗ ",
    "██╔══██╗",
    "╚█████╔╝",
    "██╔══██╗",
    "╚█████╔╝",
    " ╚════╝ ",
]);

const NINE: CompositeChar<HEIGHT> = CompositeChar('9', [
    " █████╗ ",
    "██╔══██╗",
    "╚██████║",
    " ╚═══██║",
    " █████╔╝",
    " ╚════╝ ",
]);
 
const COLON: CompositeChar<HEIGHT> = CompositeChar(':', [
    "    ",
    " ██╗",
    " ╚═╝",
    " ██╗",
    " ╚═╝",
    "    ",
]);

#[derive(Default)]
pub struct AnsiShadow;

impl Font for AnsiShadow {
    type CHAR = CompositeChar<'static, HEIGHT>;

    fn height_range(&self) -> Range<usize> {
        (0..HEIGHT).into()
    }

    fn get(&self, index: char) -> Option<CompositeChar<'static, HEIGHT>> {
        match index {
            '0' => Some(ZERO),
            '1' => Some(ONE),
            '2' => Some(TWO),
            '3' => Some(THREE),
            '4' => Some(FOUR),
            '5' => Some(FIVE),
            '6' => Some(SIX),
            '7' => Some(SEVEN),
            '8' => Some(EIGHT),
            '9' => Some(NINE),
            ':' => Some(COLON),
            _ => None,
        }
    }
}

#[cfg(test)]
pub const TEST_FOUR: CompositeChar<HEIGHT> = FOUR;

#[cfg(test)]
mod tests {
    use super::*;

    use rstest::rstest;

    #[rstest]
    #[case::zero('0', ZERO)]
    #[case::one('1', ONE)]
    #[case::two('2', TWO)]
    #[case::three('3', THREE)]
    #[case::four('4', FOUR)]
    #[case::five('5', FIVE)]
    #[case::six('6', SIX)]
    #[case::seven('7', SEVEN)]
    #[case::eight('8', EIGHT)]
    #[case::nine('9', NINE)]
    #[case::colon(':', COLON)]
    fn test_should_return_correct_character<'a>(#[case] key: char, #[case] expected: CompositeChar<'a, HEIGHT>) {
        let font = AnsiShadow;

        if let Some(actual) = font.get(key) {
            assert_eq!(actual, expected, "expected {key} to map to {expected:?}, but got {actual:?}");
        } else {
            assert!(false, "value not found for {key}");
        }
    }
    
    #[test]
    fn test_should_return_height_range() {
        let font = AnsiShadow;

        assert_eq!(font.height_range(), (0..HEIGHT).into());
    }

    #[test]
    fn test_should_return_none_given_key_that_does_not_exist() {
        let font = AnsiShadow;

        assert_eq!(font.get('a'), None);
    }
}