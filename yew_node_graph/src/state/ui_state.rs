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

use std::ops::Index;

use glam::{Vec2};
use slotmap::SecondaryMap;

use super::{AnyParameterId, InputId, NodeId, OutputId};

// Information needed when dragging or selecting a node
#[derive(Debug, Clone)]
pub struct MousePosOnNode {
    /// Id of mouse-on node
    pub id: NodeId,
    /// Position from top left of node
    pub gap: Vec2,
}

#[derive(Debug, Clone, Default)]
pub struct PortsData<T> {
    pub input: SecondaryMap<InputId, T>,
    pub output: SecondaryMap<OutputId, T>,
}

impl<T> PortsData<T> {
    pub fn insert(&mut self, key: AnyParameterId, value: T) -> Option<T> {
        match key {
            AnyParameterId::Input(id) => self.input.insert(id, value),
            AnyParameterId::Output(id) => self.output.insert(id, value),
        }
    }
    pub fn remove(&mut self, key: AnyParameterId) -> Option<T> {
        match key {
            AnyParameterId::Input(id) => self.input.remove(id),
            AnyParameterId::Output(id) => self.output.remove(id),
        }
    }
    pub fn get(&self, key: AnyParameterId) -> Option<&T> {
        match key {
            AnyParameterId::Input(id) => self.input.get(id),
            AnyParameterId::Output(id) => self.output.get(id),
        }
    }
    pub fn get_mut(&mut self, key: AnyParameterId) -> Option<&mut T> {
        match key {
            AnyParameterId::Input(id) => self.input.get_mut(id),
            AnyParameterId::Output(id) => self.output.get_mut(id),
        }
    }

    // /// Return ports that are within the threshold
    // /// # Warning
    // /// - It is not the closest port of all, since it returns when a port is found with a distance less than or equal to the threshold value.
    // /// - For optimization, the threshold value is the square of the distance
    // pub fn get_near_input(&self, pos: T, th: f32) -> Option<(InputId, &T)> {
    //     self.input
    //         .iter()
    //         .find(|(_, port_pos)| port_pos.distance_squared(pos) < th)
    // }
    // /// Output version of [`get_near_input`]
    // pub fn get_near_output(&self, pos: T, th: f32) -> Option<(OutputId, &T)> {
    //     self.output
    //         .iter()
    //         .find(|(_, port_pos)| port_pos.distance_squared(pos) < th)
    // }
}

impl<T> Index<InputId> for PortsData<T> {
    type Output = T;
    fn index(&self, index: InputId) -> &Self::Output {
        &self.input[index]
    }
}
impl<T> Index<OutputId> for PortsData<T> {
    type Output = T;
    fn index(&self, index: OutputId) -> &Self::Output {
        &self.output[index]
    }
}

impl<T> Index<AnyParameterId> for PortsData<T> {
    type Output = T;
    fn index(&self, index: AnyParameterId) -> &Self::Output {
        self.get(index).unwrap_or_else(|| {
            panic!(
                "{} index error for {:?}. Has the value been deleted?",
                stringify!(AnyParameterId),
                index
            )
        })
    }
}
