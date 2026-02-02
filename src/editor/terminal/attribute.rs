use crossterm::style::Color;

use crate::editor::annotated_string::AnnotationType;

pub struct Attribute {
    pub foreground: Option<Color>,
    pub background: Option<Color>,
}

// use proper color for annotation types
impl From<AnnotationType> for Attribute {
    fn from(annotation_type: AnnotationType) -> Self {
        match annotation_type {
            AnnotationType::Match => Self {
                foreground: Some(Color::Black),
                background: Some(Color::Yellow),
            },

            AnnotationType::SelectedMatch => Self {
                foreground: Some(Color::Black),
                background: Some(Color::Green),
            },

            AnnotationType::Digit => Self {
                foreground: Some(Color::Red),
                background: None,
            },
        }
    }
}
