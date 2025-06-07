#[derive(Debug)]
pub struct Border {
    pub corner_up_left: &'static str,
    pub corner_up_right: &'static str,
    pub corner_bottom_left: &'static str,
    pub corner_bottom_right: &'static str,
    pub division_up: &'static str,
    pub division_bottom: &'static str,
    pub horizontal: &'static str,
    pub vertical: &'static str,
    pub intersection: &'static str,
    pub vertical_left: &'static str,
    pub vertical_right: &'static str,
}

#[derive(Debug,Default)]
pub enum BorderStyle {
    #[default]
    Default,
    Simple,
    Rounded,
    Double,
    SimpleDouble,
}

impl From<String> for BorderStyle {
    fn from(value: String) -> Self {
        match value.as_str() {
            "1" => BorderStyle::Default,
            "2" => BorderStyle::Simple,
            "3" => BorderStyle::Rounded,
            "4" => BorderStyle::Double,
            "5" => BorderStyle::SimpleDouble,
            _ => unreachable!()
        }
    }
}

impl BorderStyle {
    pub fn get(&self) -> Border {
        match self {
            BorderStyle::Default => Border {
                corner_up_left: "┏",
                corner_up_right: "┓",
                corner_bottom_left: "┗",
                corner_bottom_right: "┛",
                division_up: "┳",
                division_bottom: "┻",
                horizontal: "━",
                vertical: "┃",
                intersection: "╋",
                vertical_left: "┣",
                vertical_right: "┫",
            },
            BorderStyle::Simple => Border {
                corner_up_left: "┌",
                corner_up_right: "┐",
                corner_bottom_left: "└",
                corner_bottom_right: "┘",
                division_up: "┬",
                division_bottom: "┴",
                horizontal: "─",
                vertical: "│",
                intersection: "┼",
                vertical_left: "├",
                vertical_right: "┤",
            },
            BorderStyle::Rounded => Border {
                corner_up_left: "╭",
                corner_up_right: "╮",
                corner_bottom_left: "╰",
                corner_bottom_right: "╯",
                division_up: "┬",
                division_bottom: "┴",
                horizontal: "─",
                vertical: "│",
                intersection: "┼",
                vertical_left: "├",
                vertical_right: "┤",
            },
            BorderStyle::Double => Border {
                corner_up_left: "╔",
                corner_up_right: "╗",
                corner_bottom_left: "╚",
                corner_bottom_right: "╝",
                division_up: "╦",
                division_bottom: "╩",
                horizontal: "═",
                vertical: "║",
                intersection: "╬",
                vertical_left: "╠",
                vertical_right: "╣",
            },
            BorderStyle::SimpleDouble => Border {
                corner_up_left: "╒",
                corner_up_right: "╕",
                corner_bottom_left: "╘",
                corner_bottom_right: "╛",
                division_up: "╤",
                division_bottom: "╧",
                horizontal: "═",
                vertical: "│",
                intersection: "╪",
                vertical_left: "╞",
                vertical_right: "╡",
            },
        }
    }
}
