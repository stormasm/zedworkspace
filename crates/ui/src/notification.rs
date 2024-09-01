use std::{any::TypeId, collections::VecDeque, sync::Arc, time::Duration};

use gpui::{
    div, prelude::FluentBuilder, px, Animation, AnimationExt, ClickEvent, DismissEvent, ElementId,
    EventEmitter, InteractiveElement as _, IntoElement, ParentElement as _, Render, SharedString,
    StatefulInteractiveElement, Styled, View, ViewContext, VisualContext, WindowContext,
};
use smol::Timer;

use crate::{
    animation::cubic_bezier, button::Button, h_flex, theme::ActiveTheme as _, v_flex, Icon,
    IconName, Sizable as _, StyledExt,
};

pub enum NotificationType {
    Info,
    Success,
    Warning,
    Error,
}

#[derive(Debug, PartialEq, Clone)]
pub(crate) enum NotificationId {
    Id(TypeId),
    IdAndElementId(TypeId, ElementId),
}

impl From<TypeId> for NotificationId {
    fn from(type_id: TypeId) -> Self {
        Self::Id(type_id)
    }
}

impl From<(TypeId, ElementId)> for NotificationId {
    fn from((type_id, id): (TypeId, ElementId)) -> Self {
        Self::IdAndElementId(type_id, id)
    }
}

pub struct Notification {
    /// The id is used make the notification unique.
    /// Then you push a notification with the same id, the previous notification will be replaced.
    ///
    /// None means the notification will be added to the end of the list.
    id: NotificationId,
    type_: NotificationType,
    title: Option<SharedString>,
    message: SharedString,
    icon: Option<Icon>,
    autohide: bool,
    on_click: Option<Arc<dyn Fn(&ClickEvent, &mut WindowContext)>>,
    closing: bool,
}

impl From<SharedString> for Notification {
    fn from(s: SharedString) -> Self {
        Self::new(s)
    }
}

impl From<&'static str> for Notification {
    fn from(s: &'static str) -> Self {
        Self::new(s)
    }
}

impl From<(NotificationType, &'static str)> for Notification {
    fn from((type_, content): (NotificationType, &'static str)) -> Self {
        Self::new(content).with_type(type_)
    }
}

impl From<(NotificationType, SharedString)> for Notification {
    fn from((type_, content): (NotificationType, SharedString)) -> Self {
        Self::new(content).with_type(type_)
    }
}

struct DefaultIdType;
impl Notification {
    /// Create a new notification with the given content.
    ///
    /// default width is 320px.
    pub fn new(message: impl Into<SharedString>) -> Self {
        let id: SharedString = uuid::Uuid::new_v4().to_string().into();
        let id = (TypeId::of::<DefaultIdType>(), id.into());

        Self {
            id: id.into(),
            title: None,
            message: message.into(),
            type_: NotificationType::Info,
            icon: None,
            autohide: true,
            on_click: None,
            closing: false,
        }
    }

    pub fn info(message: impl Into<SharedString>) -> Self {
        Self::new(message).with_type(NotificationType::Info)
    }

    pub fn success(message: impl Into<SharedString>) -> Self {
        Self::new(message).with_type(NotificationType::Success)
    }

    pub fn warning(message: impl Into<SharedString>) -> Self {
        Self::new(message).with_type(NotificationType::Warning)
    }

    pub fn error(message: impl Into<SharedString>) -> Self {
        Self::new(message).with_type(NotificationType::Error)
    }

    /// Set the type for unique identification of the notification.
    ///
    /// ```rs
    /// struct MyNotificationKind;
    /// let notification = Notification::new("Hello").id::<MyNotificationKind>();
    /// ```
    pub fn id<T: Sized + 'static>(mut self) -> Self {
        self.id = TypeId::of::<T>().into();
        self
    }

    /// Set the type and id of the notification, used to uniquely identify the notification.
    pub fn id1<T: Sized + 'static>(mut self, key: impl Into<ElementId>) -> Self {
        self.id = (TypeId::of::<T>(), key.into()).into();
        self
    }

    /// Set the title of the notification, default is None.
    ///
    /// If tilte is None, the notification will not have a title.
    pub fn title(mut self, title: impl Into<SharedString>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Set the icon of the notification.
    ///
    /// If icon is None, the notification will use the default icon of the type.
    pub fn icon(mut self, icon: impl Into<Icon>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    /// Set the type of the notification, default is NotificationType::Info.
    pub fn with_type(mut self, type_: NotificationType) -> Self {
        self.type_ = type_;
        self
    }

    /// Set the auto hide of the notification, default is true.
    pub fn autohide(mut self, autohide: bool) -> Self {
        self.autohide = autohide;
        self
    }

    /// Set the click callback of the notification.
    pub fn on_click(
        mut self,
        on_click: impl Fn(&ClickEvent, &mut WindowContext) + 'static,
    ) -> Self {
        self.on_click = Some(Arc::new(on_click));
        self
    }

    fn dismiss(&mut self, _: &ClickEvent, cx: &mut ViewContext<Self>) {
        self.closing = true;
        cx.notify();

        // Dismiss the notification after 0.15s to show the animation.
        cx.spawn(|view, mut cx| async move {
            Timer::after(Duration::from_secs_f32(0.15)).await;
            cx.update(|cx| {
                if let Some(view) = view.upgrade() {
                    view.update(cx, |view, cx| {
                        view.closing = false;
                        cx.emit(DismissEvent);
                    });
                }
            })
        })
        .detach()
    }
}
impl EventEmitter<DismissEvent> for Notification {}
impl FluentBuilder for Notification {}
impl Render for Notification {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        let closing = self.closing;
        let icon = match self.icon.clone() {
            Some(icon) => icon,
            None => match self.type_ {
                NotificationType::Info => Icon::new(IconName::Info).text_color(crate::blue_500()),
                NotificationType::Success => {
                    Icon::new(IconName::CircleCheck).text_color(crate::green_500())
                }
                NotificationType::Warning => {
                    Icon::new(IconName::TriangleAlert).text_color(crate::yellow_500())
                }
                NotificationType::Error => {
                    Icon::new(IconName::CircleX).text_color(crate::red_500())
                }
            },
        };

        div()
            .id("notification")
            .group("")
            .occlude()
            .relative()
            .w_96()
            .border_1()
            .border_color(cx.theme().border)
            .bg(cx.theme().popover)
            .rounded_md()
            .shadow_md()
            .py_2()
            .px_4()
            .gap_3()
            .child(div().absolute().top_3().left_4().child(icon))
            .child(
                v_flex()
                    .pl_6()
                    .gap_1()
                    .when_some(self.title.clone(), |this, title| {
                        this.child(div().text_sm().font_semibold().child(title))
                    })
                    .overflow_hidden()
                    .child(div().text_sm().child(self.message.clone())),
            )
            .when_some(self.on_click.clone(), |this, on_click| {
                this.cursor_pointer()
                    .on_click(cx.listener(move |view, event, cx| {
                        view.dismiss(event, cx);
                        on_click(event, cx);
                    }))
            })
            .when(!self.autohide, |this| {
                this.child(
                    h_flex()
                        .absolute()
                        .top_1()
                        .right_1()
                        .invisible()
                        .group_hover("", |this| this.visible())
                        .child(
                            Button::new("close", cx)
                                .icon(IconName::Close)
                                .ghost()
                                .xsmall()
                                .on_click(cx.listener(Self::dismiss)),
                        ),
                )
            })
            .with_animation(
                ElementId::NamedInteger("slide-down".into(), closing as usize),
                Animation::new(Duration::from_secs_f64(0.15))
                    .with_easing(cubic_bezier(0.4, 0., 0.2, 1.)),
                move |this, delta| {
                    if closing {
                        let x_offset = px(0.) + delta * px(45.);
                        this.left(px(0.) + x_offset).opacity(1. - delta)
                    } else {
                        let y_offset = px(-45.) + delta * px(45.);
                        this.top(px(0.) + y_offset)
                    }
                },
            )
    }
}

/// A list of notifications.
pub struct NotificationList {
    /// Notifications that will be auto hidden.
    pub(crate) notifications: VecDeque<View<Notification>>,
    expanded: bool,
}

impl NotificationList {
    pub fn new(_cx: &mut ViewContext<Self>) -> Self {
        Self {
            notifications: VecDeque::new(),
            expanded: false,
        }
    }

    pub fn push(&mut self, notification: impl Into<Notification>, cx: &mut ViewContext<Self>) {
        let notification = notification.into();
        let id = notification.id.clone();
        let autohide = notification.autohide;

        // Remove the notification by id, for keep unique.
        self.notifications.retain(|note| note.read(cx).id != id);

        let notification = cx.new_view(|_| notification);
        cx.subscribe(&notification, move |view, _, _: &DismissEvent, cx| {
            view.notifications.retain(|note| id != note.read(cx).id);
        })
        .detach();

        self.notifications.push_back(notification);
        if autohide {
            // Sleep for 5 seconds to autohide the notification
            cx.spawn(|view, mut cx| async move {
                Timer::after(Duration::from_secs(5)).await;
                let _ = view.update(&mut cx, |view, cx| {
                    if let Some(ix) = view
                        .notifications
                        .iter()
                        .position(|note| note.read(cx).autohide)
                    {
                        if let Some(note) = view.notifications.get(ix) {
                            note.update(cx, |note, cx| note.dismiss(&ClickEvent::default(), cx));
                        }
                    }
                    cx.notify()
                });
            })
            .detach();
        }
        cx.notify();
    }

    pub fn clear(&mut self, cx: &mut ViewContext<Self>) {
        self.notifications.clear();
        cx.notify();
    }

    pub fn notifications(&self) -> Vec<View<Notification>> {
        self.notifications.iter().cloned().collect()
    }
}

impl Render for NotificationList {
    fn render(&mut self, cx: &mut gpui::ViewContext<Self>) -> impl IntoElement {
        let size = cx.viewport_size();
        let items = self.notifications.iter().rev().take(10).rev().cloned();

        div()
            .absolute()
            .flex()
            .top_4()
            .bottom_4()
            .right_4()
            .justify_end()
            .child(
                v_flex()
                    .id("notification-list")
                    .absolute()
                    .relative()
                    .right_0()
                    .h(size.height - px(8.))
                    .on_hover(cx.listener(|view, hovered, cx| {
                        view.expanded = *hovered;
                        cx.notify()
                    }))
                    .gap_3()
                    .children(items),
            )
    }
}
