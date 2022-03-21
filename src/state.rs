#[derive(Clone, Debug, PartialEq)]
pub enum State {
    Loading,
    Loaded,
    None,
}

impl Default for State {
    fn default() -> Self {
        Self::None
    }
}
