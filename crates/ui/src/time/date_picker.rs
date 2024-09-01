use gpui::{
    anchored, deferred, div, prelude::FluentBuilder as _, px, AppContext, ElementId, EventEmitter,
    FocusHandle, FocusableView, InteractiveElement as _, KeyBinding, Length, MouseButton,
    ParentElement as _, Render, SharedString, StatefulInteractiveElement as _, Styled as _, View,
    ViewContext, VisualContext as _,
};
use rust_i18n::t;

use crate::{
    dropdown::Escape, h_flex, input::ClearButton, theme::ActiveTheme as _, Icon, IconName, Sizable,
    Size, StyleSized as _, StyledExt as _,
};

use super::calendar::{Calendar, CalendarEvent, Date};

pub fn init(cx: &mut AppContext) {
    let context = Some("DatePicker");
    cx.bind_keys([KeyBinding::new("escape", Escape, context)])
}

#[derive(Clone)]
pub enum DatePickerEvent {
    Change(Date),
}

pub struct DatePicker {
    id: ElementId,
    focus_handle: FocusHandle,
    date: Date,
    cleanable: bool,
    placeholder: Option<SharedString>,
    open: bool,
    size: Size,
    width: Length,
    date_format: SharedString,
    calendar: View<Calendar>,
    number_of_months: usize,
}

impl DatePicker {
    pub fn new(id: impl Into<ElementId>, cx: &mut ViewContext<Self>) -> Self {
        let calendar = cx.new_view(Calendar::new);

        cx.subscribe(&calendar, |this, _, ev: &CalendarEvent, cx| match ev {
            CalendarEvent::Selected(date) => {
                this.update_date(*date, cx);
            }
        })
        .detach();

        Self {
            id: id.into(),
            focus_handle: cx.focus_handle(),
            date: Date::Single(None),
            calendar,
            open: false,
            size: Size::default(),
            width: Length::Auto,
            date_format: "%Y/%m/%d".into(),
            cleanable: false,
            number_of_months: 1,
            placeholder: None,
        }
    }

    /// Set the date format of the date picker to display in Input, default: "%Y/%m/%d".
    pub fn date_format(mut self, format: impl Into<SharedString>) -> Self {
        self.date_format = format.into();
        self
    }

    /// Set the placeholder of the date picker, default: "".
    pub fn placeholder(mut self, placeholder: impl Into<SharedString>) -> Self {
        self.placeholder = Some(placeholder.into());
        self
    }

    /// Set true to show the clear button when the input field is not empty.
    pub fn cleanable(mut self) -> Self {
        self.cleanable = true;
        self
    }

    /// Set width of the date picker input field, default is `Length::Auto`.
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    /// Set the number of months calendar view to display, default is 1.
    pub fn number_of_months(mut self, number_of_months: usize) -> Self {
        self.number_of_months = number_of_months;
        self
    }

    /// Get the date of the date picker.
    pub fn date(&self) -> Date {
        self.date
    }

    /// Set the date of the date picker.
    pub fn set_date(&mut self, date: impl Into<Date>, cx: &mut ViewContext<Self>) {
        self.update_date(date.into(), cx);
    }

    fn update_date(&mut self, date: Date, cx: &mut ViewContext<Self>) {
        self.date = date;
        self.calendar.update(cx, |view, cx| {
            view.set_date(date, cx);
        });
        self.open = false;
        cx.emit(DatePickerEvent::Change(date));
        cx.notify();
    }

    fn escape(&mut self, _: &Escape, cx: &mut ViewContext<Self>) {
        self.open = false;
        cx.notify();
    }

    fn clean(&mut self, _: &gpui::ClickEvent, cx: &mut ViewContext<Self>) {
        self.update_date(Date::Single(None), cx);
    }

    fn toggle_calendar(&mut self, _: &gpui::ClickEvent, cx: &mut ViewContext<Self>) {
        self.open = !self.open;
        cx.notify();
    }
}

impl EventEmitter<DatePickerEvent> for DatePicker {}
impl Sizable for DatePicker {
    fn with_size(mut self, size: impl Into<Size>) -> Self {
        self.size = size.into();
        self
    }
}
impl FocusableView for DatePicker {
    fn focus_handle(&self, _: &AppContext) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for DatePicker {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl gpui::IntoElement {
        let is_focused = self.focus_handle.is_focused(cx);
        let show_clean = self.cleanable && self.date.is_some();
        let placeholder = self
            .placeholder
            .clone()
            .unwrap_or_else(|| t!("DatePicker.placeholder").into());
        let display_title = self
            .date
            .format(&self.date_format)
            .unwrap_or(placeholder.clone());

        self.calendar.update(cx, |view, cx| {
            view.set_number_of_months(self.number_of_months, cx);
        });

        let popover_width =
            285.0 * self.number_of_months as f32 + (self.number_of_months - 1) as f32 * 16.0;

        div()
            .id(self.id.clone())
            .key_context("DatePicker")
            .track_focus(&self.focus_handle)
            .on_action(cx.listener(Self::escape))
            .w_full()
            .relative()
            .map(|this| match self.width {
                Length::Definite(l) => this.flex_none().w(l),
                Length::Auto => this.w_full(),
            })
            .input_text_size(self.size)
            .child(
                div()
                    .id("date-picker-input")
                    .relative()
                    .flex()
                    .items_center()
                    .justify_between()
                    .bg(cx.theme().background)
                    .border_1()
                    .border_color(cx.theme().input)
                    .rounded(px(cx.theme().radius))
                    .shadow_sm()
                    .cursor_pointer()
                    .overflow_hidden()
                    .input_text_size(self.size)
                    .when(is_focused, |this| this.outline(cx))
                    .input_size(self.size)
                    .when(!self.open, |this| {
                        this.on_click(cx.listener(Self::toggle_calendar))
                    })
                    .child(
                        h_flex()
                            .w_full()
                            .items_center()
                            .justify_between()
                            .gap_1()
                            .child(div().w_full().overflow_hidden().child(display_title))
                            .when(show_clean, |this| {
                                this.child(ClearButton::new(cx).on_click(cx.listener(Self::clean)))
                            })
                            .when(!show_clean, |this| {
                                this.child(
                                    Icon::new(IconName::Calendar)
                                        .text_color(cx.theme().muted_foreground),
                                )
                            }),
                    ),
            )
            .when(self.open, |this| {
                this.child(
                    deferred(
                        anchored().snap_to_window().child(
                            div()
                                .track_focus(&self.focus_handle)
                                .occlude()
                                .absolute()
                                .mt_1p5()
                                .overflow_hidden()
                                .rounded_lg()
                                .p_3()
                                .w(px(popover_width))
                                .border_1()
                                .border_color(cx.theme().border)
                                .shadow_lg()
                                .rounded_lg()
                                .bg(cx.theme().background)
                                .on_mouse_up_out(
                                    MouseButton::Left,
                                    cx.listener(|view, _, cx| view.escape(&Escape, cx)),
                                )
                                .child(self.calendar.clone()),
                        ),
                    )
                    .with_priority(2),
                )
            })
    }
}
