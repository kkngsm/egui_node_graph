/// NodeFinder Status
#[derive(Debug, Default, Clone)]
#[cfg_attr(feature = "persistence", derive(serde::Serialize, serde::Deserialize))]
pub struct NodeFinder {
    pub pos: crate::Vec2,
    pub is_showing: bool,
}
