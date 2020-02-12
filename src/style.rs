use iced::{button, Background, Color, Vector};

pub enum Button {
    Primary,
    Secondary,
    Destructive,
}

impl button::StyleSheet for Button {
    fn active(&self) -> button::Style {
        button::Style {
            background: Some(Background::Color(match self {
                Button::Primary => Color::from_rgb(0.11, 0.42, 0.87),
                Button::Secondary => Color::from_rgb(0.5, 0.5, 0.5),
                Button::Destructive => Color::from_rgb(0.8, 0.2, 0.2),
            })),
            border_width: 1,
            shadow_offset: Vector::new(0.0, 0.0),
            text_color: Color::WHITE,
            ..button::Style::default()
        }
    }

    fn hovered(&self) -> button::Style {
        let active = self.active();

        button::Style {
            background: Some(Background::Color(match self {
                Button::Primary => Color::from_rgb(0.08, 0.38, 0.83),
                Button::Secondary => Color::from_rgb(0.46, 0.46, 0.46),
                Button::Destructive => Color::from_rgb(0.76, 0.16, 0.16),
            })),
            ..active
        }
    }
}
