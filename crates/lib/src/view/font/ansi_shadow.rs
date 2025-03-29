use std::ops::Range;

use super::{CharacterSet, CompositeChar};

const HEIGHT: usize = 6;

const ZERO: CompositeChar<HEIGHT> = CompositeChar::new('0', [
    " ██████╗ ",  
    "██╔═████╗",
    "██║██╔██║",
    "████╔╝██║",
    "╚██████╔╝",
    " ╚═════╝ ",
]);
const ONE: CompositeChar<HEIGHT> = CompositeChar::new('1', [
    " ██╗",
    "███║",
    "╚██║",
    " ██║",
    " ██║",
    " ╚═╝",
]);

const TWO: CompositeChar<HEIGHT> = CompositeChar::new('2', [
    "██████╗ ",
    "╚════██╗",
    " █████╔╝",
    "██╔═══╝ ",
    "███████╗",
    "╚══════╝",
]);

const THREE: CompositeChar<HEIGHT> = CompositeChar::new('3', [
    "██████╗ ",
    "╚════██╗",
    " █████╔╝",
    " ╚═══██╗",
    "██████╔╝",
    "╚═════╝ ",
]);

const FOUR: CompositeChar<HEIGHT> = CompositeChar::new('4', [
    "██╗  ██╗",
    "██║  ██║",
    "███████║",
    "╚════██║",
    "     ██║",
    "     ╚═╝",
]);

const FIVE: CompositeChar<HEIGHT> = CompositeChar::new('5', [
    "███████╗",
    "██╔════╝",
    "███████╗",
    "╚════██║",
    "███████║",
    "╚══════╝",
]);

const SIX: CompositeChar<HEIGHT> = CompositeChar::new('6', [
    " ██████╗ ",
    "██╔════╝ ",
    "███████╗ ",
    "██╔═══██╗",
    "╚██████╔╝",
    " ╚═════╝ ",
]);

const SEVEN: CompositeChar<HEIGHT> = CompositeChar::new('7', [
    "███████╗",
    "╚════██║",
    "    ██╔╝",
    "   ██╔╝ ",
    "   ██║  ",
    "   ╚═╝  ",
]);

const EIGHT: CompositeChar<HEIGHT> = CompositeChar::new('8', [
    " █████╗ ",
    "██╔══██╗",
    "╚█████╔╝",
    "██╔══██╗",
    "╚█████╔╝",
    " ╚════╝ ",
]);

const NINE: CompositeChar<HEIGHT> = CompositeChar::new('9', [
    " █████╗ ",
    "██╔══██╗",
    "╚██████║",
    " ╚═══██║",
    " █████╔╝",
    " ╚════╝ ",
]);
 
const COLON: CompositeChar<HEIGHT> = CompositeChar::new(':', [
    "    ",
    " ██╗",
    " ╚═╝",
    " ██╗",
    " ╚═╝",
    "    ",
]);

pub struct AnsiShadow;

impl CharacterSet<CompositeChar<6>> for AnsiShadow {
    fn height_range(&self) -> Range<usize> {
        (0..HEIGHT).into()
    }

    fn get(&self, index: char) -> Option<&CompositeChar<HEIGHT>> {
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
        let font = AnsiShadow;
        assert_eq!(font.height_range(), (0..6).into());
        assert!(font.get('0').is_some());
        assert!(font.get('1').is_some());
        assert!(font.get('2').is_some());
        assert!(font.get('3').is_some());
        assert!(font.get('4').is_some());
        assert!(font.get('5').is_some());
        assert!(font.get('6').is_some());
        assert!(font.get('7').is_some());
        assert!(font.get('8').is_some());
        assert!(font.get('9').is_some());
        assert!(font.get(':').is_some());
        assert!(font.get('a').is_none());
    }
}