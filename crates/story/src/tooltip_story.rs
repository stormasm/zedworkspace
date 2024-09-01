use gpui::{
    div, CursorStyle, InteractiveElement, ParentElement, Render, StatefulInteractiveElement,
    Styled, View, VisualContext as _, WindowContext,
};

use ui::{
    button::{Button, ButtonStyle},
    checkbox::Checkbox,
    h_flex,
    label::Label,
    tooltip::Tooltip,
    v_flex,
};

pub struct TooltipStory;

impl TooltipStory {
    pub fn view(cx: &mut WindowContext) -> View<Self> {
        cx.new_view(|cx| Self::new(cx))
    }

    fn new(_: &mut WindowContext) -> Self {
        Self {}
    }
}

impl Render for TooltipStory {
    fn render(&mut self, cx: &mut gpui::ViewContext<Self>) -> impl gpui::IntoElement {
        v_flex()
            .p_4()
            .gap_5()
            .child(
                div()
                    .cursor(CursorStyle::PointingHand)
                    .child(
                        Button::new("button", cx)
                            .label("Hover me")
                            .style(ButtonStyle::Primary),
                    )
                    .id("tooltip-1")
                    .tooltip(|cx| Tooltip::new("This is a Button", cx)),
            )
            .child(
                h_flex()
                    .justify_center()
                    .cursor(CursorStyle::PointingHand)
                    .child(Label::new("Hover me"))
                    .id("tooltip-3")
                    .tooltip(|cx| Tooltip::new("This is a Label", cx)),
            )
            .child(
                div()
                    .cursor(CursorStyle::PointingHand)
                    .child(Checkbox::new("check").label("Remember me").checked(true))
                    .id("tooltip-4")
                    .tooltip(|cx| Tooltip::new("Checked!", cx)),
            )
    }
}
