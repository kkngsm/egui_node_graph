use std::{fmt::Display, rc::Rc};

use yew::{function_component, html, Callback, Html, Properties};

use crate::state::{InputParam, NodeId, OutputParam, WidgetValueTrait};

#[derive(Properties)]
pub struct InputWidgetProps<NodeData, DataType, ValueType, UserState, UserResponse> {
    pub name: Rc<String>,
    pub param: Rc<InputParam<DataType, ValueType>>,
    pub node_data: Rc<NodeData>,
    pub node_id: NodeId,
    pub is_connected: bool,
    pub user_state: UserState,
    pub user_callback: Callback<UserResponse>,
}

#[function_component(InputWidget)]
pub fn input_widget<NodeData, DataType, ValueType, UserState, UserResponse>(
    InputWidgetProps {
        is_connected,
        param,
        name,
        node_data,
        node_id,
        user_state,
        user_callback,
    }: &InputWidgetProps<NodeData, DataType, ValueType, UserState, UserResponse>,
) -> Html
where
    DataType: Display,
    ValueType:
        WidgetValueTrait<NodeData = NodeData, UserState = UserState, Response = UserResponse>,
{
    let widget = if *is_connected {
        html! {name.as_str()}
    } else {
        param.value.value_widget(
            name.as_str(),
            *node_id,
            user_state,
            node_data,
            user_callback.clone(),
        )
    };
    html! {
        <div
            class={"widget"}
            data-type={param.typ.to_string()}
        >
            {widget}
        </div>
    }
}

#[derive(Properties)]
pub struct OutputWidgetProps<DataType> {
    pub name: Rc<String>,
    pub param: Rc<OutputParam<DataType>>,
}

#[function_component(OutputWidget)]
pub fn output_widget<DataType>(
    OutputWidgetProps { param, name }: &OutputWidgetProps<DataType>,
) -> Html
where
    DataType: Display,
{
    html! {
        <div
            key={param.id.to_string()}
            class={"widget"}
            data-type={param.typ.to_string()}
        >
            {name.as_str()}
        </div>
    }
}

impl<NodeData, DataType, ValueType, UserState, UserResponse> PartialEq
    for InputWidgetProps<NodeData, DataType, ValueType, UserState, UserResponse>
{
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.name, &other.name)
            && Rc::ptr_eq(&self.node_data, &other.node_data)
            && Rc::ptr_eq(&self.param, &other.param)
            && self.node_id == other.node_id
            && self.is_connected == other.is_connected
            && self.user_callback == other.user_callback
        // The following always return True, because RefCell is used.
        // && Rc::ptr_eq(&self.user_state, &other.user_state)
    }
}

impl<DataType> PartialEq for OutputWidgetProps<DataType> {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.name, &other.name) && Rc::ptr_eq(&self.param, &other.param)
    }
}
