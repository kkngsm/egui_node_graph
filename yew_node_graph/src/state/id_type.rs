use std::fmt::Display;

slotmap::new_key_type! { pub struct NodeId; }
slotmap::new_key_type! { pub struct InputId; }
slotmap::new_key_type! { pub struct OutputId; }

#[cfg_attr(feature = "persistence", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum AnyParameterId {
    Input(InputId),
    Output(OutputId),
}

impl AnyParameterId {
    pub fn assume_input(&self) -> InputId {
        match self {
            AnyParameterId::Input(input) => *input,
            AnyParameterId::Output(output) => panic!("{output:?} is not an InputId"),
        }
    }
    pub fn assume_output(&self) -> OutputId {
        match self {
            AnyParameterId::Output(output) => *output,
            AnyParameterId::Input(input) => panic!("{input:?} is not an OutputId"),
        }
    }
}

impl From<OutputId> for AnyParameterId {
    fn from(output: OutputId) -> Self {
        Self::Output(output)
    }
}

impl From<InputId> for AnyParameterId {
    fn from(input: InputId) -> Self {
        Self::Input(input)
    }
}

impl Display for NodeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl Display for InputId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl Display for OutputId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
