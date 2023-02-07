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
