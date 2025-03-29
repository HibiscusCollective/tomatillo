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

#[cfg(test)]
mod tests {
    use crate::view::font::{self, CharacterSet};

    use rstest::rstest;
    use indoc::indoc;

    use super::font::Character;

    #[rstest]
    #[case::font_ansi_shadow(font::ANSI_SHADOW, ANSI_SHADOW_ZERO)]
    #[case::font_none(font::NONE, "00:00")]
    fn should_render_timer_at_zero<T: Character>(#[case] _font: impl CharacterSet<T>, #[case] _expected: &str) {
        // let view = View { font: font };
        // let actual = view.render(0);

        // assert_eq!(actual, expected); Skip this test for now
    }

    const ANSI_SHADOW_ZERO: &str = indoc!("
         ██████╗   ██████╗       ██████╗   ██████╗ 
        ██╔═████╗ ██╔═████╗ ██╗ ██╔═████╗ ██╔═████╗
        ██║██╔██║ ██║██╔██║ ╚═╝ ██║██╔██║ ██║██╔██║
        ████╔╝██║ ████╔╝██║ ██╗ ████╔╝██║ ████╔╝██║
        ╚██████╔╝ ╚██████╔╝ ╚═╝ ╚██████╔╝ ╚██████╔╝
         ╚═════╝   ╚═════╝       ╚═════╝   ╚═════╝ 
    ");
}