#[derive(Default, Copy, Clone)]
#[cfg_attr(feature = "persistence", derive(serde::Serialize, serde::Deserialize))]
pub struct PanZoom {
    pub pan: crate::Vec2,
    pub zoom: f32,
}

impl PanZoom {
    pub fn adjust_zoom(
        &mut self,
        zoom_delta: f32,
        point: crate::Vec2,
        zoom_min: f32,
        zoom_max: f32,
    ) {
        let zoom_clamped = (self.zoom + zoom_delta).clamp(zoom_min, zoom_max);
        let zoom_delta = zoom_clamped - self.zoom;

        self.zoom += zoom_delta;
        self.pan += point * zoom_delta;
    }
}

/// NodeFinder Status
#[derive(Debug, Default, Clone)]
#[cfg_attr(feature = "persistence", derive(serde::Serialize, serde::Deserialize))]
pub struct NodeFinder {
    pub pos: crate::Vec2,
    pub is_showing: bool,
}

use glam::Vec2;

use super::NodeId;

// Information needed when dragging or selecting a node
#[derive(Debug, Clone)]
pub struct MousePosOnNode {
    /// Id of mouse-on node
    pub id: NodeId,
    /// Position from top left of node
    pub gap: Vec2,
}
