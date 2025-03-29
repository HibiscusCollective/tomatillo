use std::{collections::HashMap, fmt::{Debug, Write}, ops::Range};

pub const NONE: NoopFont = NoopFont;

pub trait CharacterSet<T: Character> {
    fn height_range(&self) -> Range<usize>;
    fn get(&self, index: char) -> Option<&T>;
}

pub trait Character: Debug + Eq + PartialEq {
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
        (0..0).into()
    }

    fn get(&self, _index: char) -> Option<&char> {
        None
    }
}

impl<const HEIGHT: usize> CharacterSet<CompositeChar<HEIGHT>> for HashMap<char, CompositeChar<HEIGHT>> {
    fn height_range(&self) -> Range<usize> {
        (0..HEIGHT).into()
    }

    fn get(&self, index: char) -> Option<&CompositeChar<HEIGHT>> {
        self.get(&index)
    }
}

impl<const HEIGHT: usize> Character for CompositeChar<HEIGHT> {
    fn width(&self) -> usize {
        3
    }

    fn height(&self) -> usize {
        HEIGHT
    }

    fn draw_line(&self, writer: &mut impl Write, line: usize) {
        write!(writer, "{}\n", self.lines[line]).unwrap()
    }
}

impl Character for char {
    fn width(&self) -> usize {
        todo!()
    }

    fn height(&self) -> usize {
        todo!()
    }

    fn draw_line(&self, _writer: &mut impl Write, line: usize) {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;

    use indoc::indoc;
    use rstest::rstest;

    #[rstest]
    fn test_retrieve_character() {
        let test_font: Box<dyn CharacterSet<CompositeChar<2>>> = Box::new(new_test_character_set());

        let actual = test_font.get('A').expect("should have found character");

        let mut writer = String::new();
        for i in test_font.height_range() {
            actual.draw_line(&mut writer, i);
        }

        assert_eq!(
            writer, 
            indoc!(" _ 
                    |-|
            ")
        );
    }

    fn new_test_character_set() -> HashMap<char, CompositeChar<2>> {  
        let mut hm = HashMap::new();
        hm.insert('A', CompositeChar::new('A', [
            " _ ",
            "|-|",
        ]));
        hm
    }
}