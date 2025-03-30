use crate::view::font::{Character, Font};

mod font;

pub struct View<'a, C: Character> {   
    _font: &'a dyn Font<CHAR = C>,
}

impl<'a, C: Character> View<'a, C> {
    pub fn render(&self, _time: u32) -> String {
        todo!()
    }   
}

#[cfg(test)]
mod tests {
    use super::font::{self, Character, Font};

    use rstest::rstest;
    use indoc::indoc;

    #[rstest]
    #[case::font_ansi_shadow(font::ANSI_SHADOW, ANSI_SHADOW_ZERO)]
    #[case::font_electronic(font::ELECTRONIC, ELECTRONIC_ZERO)]
    #[case::font_templar(font::TEMPLAR, TEMPLAR_ZERO)]
    #[case::font_none(font::NONE, "00:00")]
    fn should_render_timer_at_zero<'a, C: Character>(#[case] _font: impl Font<CHAR = C>, #[case] _expected: &str) {
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

    const ELECTRONIC_ZERO: &str = indoc!("
          ▄▄▄▄▄▄▄▄▄      ▄▄▄▄▄▄▄▄▄            ▄▄▄▄▄▄▄▄▄      ▄▄▄▄▄▄▄▄▄     
         ▐░░░░░░░░░▌    ▐░░░░░░░░░▌          ▐░░░░░░░░░▌    ▐░░░░░░░░░▌  
        ▐░█░█▀▀▀▀▀█░▌  ▐░█░█▀▀▀▀▀█░▌   ▄▄   ▐░█░█▀▀▀▀▀█░▌  ▐░█░█▀▀▀▀▀█░▌
        ▐░▌▐░▌    ▐░▌  ▐░▌▐░▌    ▐░▌  ▐░░▌  ▐░▌▐░▌    ▐░▌  ▐░▌▐░▌    ▐░▌  
        ▐░▌ ▐░▌   ▐░▌  ▐░▌ ▐░▌   ▐░▌   ▀▀   ▐░▌ ▐░▌   ▐░▌  ▐░▌ ▐░▌   ▐░▌
        ▐░▌  ▐░▌  ▐░▌  ▐░▌  ▐░▌  ▐░▌        ▐░▌  ▐░▌  ▐░▌  ▐░▌  ▐░▌  ▐░▌
        ▐░▌   ▐░▌ ▐░▌  ▐░▌   ▐░▌ ▐░▌   ▄▄   ▐░▌   ▐░▌ ▐░▌  ▐░▌   ▐░▌ ▐░▌
        ▐░▌    ▐░▌▐░▌  ▐░▌    ▐░▌▐░▌  ▐░░▌  ▐░▌    ▐░▌▐░▌  ▐░▌    ▐░▌▐░▌
        ▐░█▄▄▄▄▄█░█░▌  ▐░█▄▄▄▄▄█░█░▌   ▀▀   ▐░█▄▄▄▄▄█░█░▌  ▐░█▄▄▄▄▄█░█░▌ 
         ▐░░░░░░░░░▌    ▐░░░░░░░░░▌          ▐░░░░░░░░░▌    ▐░░░░░░░░░▌ 
          ▀▀▀▀▀▀▀▀▀      ▀▀▀▀▀▀▀▀▀            ▀▀▀▀▀▀▀▀▀      ▀▀▀▀▀▀▀▀▀  
    ");

    const TEMPLAR_ZERO: &str = indoc!("
        ┏┓ ┏┓   ┏┓ ┏┓
        ┃┫ ┃┫ • ┃┫ ┃┫
        ┗┛ ┗┛ • ┗┛ ┗┛
    ");
}