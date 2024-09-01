use gpui::{
    div, ClickEvent, Div, ElementId, InteractiveElement, IntoElement, MouseButton, ParentElement,
    RenderOnce, SharedString, Stateful, StatefulInteractiveElement, Styled,
};

use crate::theme::{ActiveTheme as _, Colorize};

#[derive(IntoElement)]
pub struct Link {
    base: Stateful<Div>,
    href: Option<SharedString>,
    on_click: Option<Box<dyn Fn(&ClickEvent, &mut gpui::WindowContext) + 'static>>,
}

impl Link {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            base: div().id(id),
            href: None,
            on_click: None,
        }
    }

    pub fn href(mut self, href: impl Into<SharedString>) -> Self {
        self.href = Some(href.into());
        self
    }

    pub fn on_click(
        mut self,
        handler: impl Fn(&ClickEvent, &mut gpui::WindowContext) + 'static,
    ) -> Self {
        self.on_click = Some(Box::new(handler));
        self
    }
}

impl Styled for Link {
    fn style(&mut self) -> &mut gpui::StyleRefinement {
        self.base.style()
    }
}

impl ParentElement for Link {
    fn extend(&mut self, elements: impl IntoIterator<Item = gpui::AnyElement>) {
        self.base.extend(elements)
    }
}

impl RenderOnce for Link {
    fn render(self, cx: &mut gpui::WindowContext) -> impl IntoElement {
        let href = self.href.clone();
        let on_click = self.on_click;

        div()
            .text_color(cx.theme().link)
            .text_decoration_1()
            .text_decoration_color(cx.theme().link)
            .hover(|this| {
                this.text_color(cx.theme().link.opacity(0.8))
                    .text_decoration_1()
            })
            .cursor_pointer()
            .child(
                self.base
                    .active(|this| {
                        this.text_color(cx.theme().link.opacity(0.6))
                            .text_decoration_1()
                    })
                    .on_mouse_down(MouseButton::Left, |_, cx| {
                        cx.prevent_default();
                        cx.stop_propagation();
                    })
                    .on_click({
                        move |e, cx| {
                            if let Some(href) = &href {
                                cx.open_url(&href.clone());
                            }
                            if let Some(on_click) = &on_click {
                                on_click(e, cx);
                            }
                        }
                    }),
            )
    }
}
