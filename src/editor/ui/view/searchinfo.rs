use super::location::Location;
use crate::editor::line::Line;

pub struct SearchInfo {
    pub previous_location: Location,
    pub query: Option<Line>,
}
