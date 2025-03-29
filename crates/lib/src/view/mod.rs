use crate::view::font::CharacterSet;

mod font;

pub struct View {   
    _font: Box<dyn CharacterSet<char>>,
}

impl View {
    pub fn render(&self, _time: u32) -> String {
        todo!()
    }   
}
