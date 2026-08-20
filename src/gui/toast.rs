use std::time::{Duration, Instant};

use iced::{
    Alignment, Length, Point, Radians,
    widget::canvas::{self, Path, Stroke, stroke},
};

use crate::gui::{
    icon::Icon,
    style,
    widget::{Canvas, Column, Container, Element, Row, Space, text},
};

pub const PINK: iced::Color = iced::Color::from_rgb(1.0, 0.427, 0.686); // #ff6daf
pub const GREEN: iced::Color = iced::Color::from_rgb(0.110, 0.875, 0.420); // #1cdf6b
pub const RED: iced::Color = iced::Color::from_rgb(0.788, 0.302, 0.302); // #c94d4d
pub const DARK: iced::Color = iced::Color::from_rgb(0.039, 0.039, 0.051); // #0a0a0d

#[derive(Debug, Clone)]
pub enum ToastKind {
    Progress { title: String, subtitle: String, percent: u8 },
    Done { title: String, subtitle: String },
    NoChanges { title: String, subtitle: String },
    Error { title: String, subtitle: String },
}

impl ToastKind {
    /// A rough measure of how wide the window should be, based on the longer line of text.
    pub fn width_hint(&self) -> usize {
        let (title, subtitle) = match self {
            Self::Progress { title, subtitle, .. } => (title, subtitle),
            Self::Done { title, subtitle } => (title, subtitle),
            Self::NoChanges { title, subtitle } => (title, subtitle),
            Self::Error { title, subtitle } => (title, subtitle),
        };
        title.chars().count().max(subtitle.chars().count())
    }
}

pub struct Toast {
    pub kind: ToastKind,
    pub created: Instant,
}

impl Toast {
    pub fn new(kind: ToastKind) -> Self {
        Self {
            kind,
            created: Instant::now(),
        }
    }
}

fn ease_out_cubic(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    1.0 - (1.0 - t).powi(3)
}

/// Fraction (0..1) of `duration_ms` elapsed since `delay_ms`, eased.
fn eased_progress(elapsed: Duration, delay_ms: u64, duration_ms: u64) -> f32 {
    let elapsed_ms = elapsed.as_millis() as i64 - delay_ms as i64;
    if elapsed_ms <= 0 {
        return 0.0;
    }
    ease_out_cubic(elapsed_ms as f32 / duration_ms as f32)
}

pub fn view(toast: &Toast) -> Element<'static> {
    let elapsed = toast.created.elapsed();

    let (chip, title, subtitle, accessory, bar): (Element, String, String, Option<Element>, Option<Element>) =
        match &toast.kind {
            ToastKind::Progress { title, subtitle, percent } => {
                let angle = Radians((elapsed.as_millis() as f32 / 900.0) * std::f32::consts::TAU);
                let chip = chip_container(style::Container::ToastChipTransparent, spinner(angle));
                let fill = eased_progress(elapsed, 400, 1200) * (*percent as f32 / 100.0);
                (chip, title.clone(), subtitle.clone(), Some(percent_label(*percent)), Some(progress_bar(fill)))
            }
            ToastKind::Done { title, subtitle } => {
                let chip = chip_container(style::Container::ToastChipDone, app_icon(24.0));
                let pop = eased_progress(elapsed, 550, 350);
                let fill = eased_progress(elapsed, 500, 1000);
                (chip, title.clone(), subtitle.clone(), Some(check_circle(pop)), Some(progress_bar(fill)))
            }
            ToastKind::NoChanges { title, subtitle } => {
                let chip = chip_container(style::Container::ToastChipNoChanges, app_icon(22.0));
                (chip, title.clone(), subtitle.clone(), None, None)
            }
            ToastKind::Error { title, subtitle } => {
                let chip = chip_container(style::Container::ToastChipError, error_mark());
                (chip, title.clone(), subtitle.clone(), Some(error_badge()), None)
            }
        };

    let mut text_col = Column::new()
        .spacing(2)
        .push(text(title).size(14).class(style::Text::ToastTitle))
        .push(text(subtitle).size(12).class(style::Text::ToastSubtitle));
    if let Some(bar) = bar {
        text_col = text_col.push(Space::new().width(Length::Fill).height(9)).push(bar);
    }

    let mut row = Row::new()
        .spacing(14)
        .align_y(Alignment::Center)
        .push(chip)
        .push(Container::new(text_col).width(Length::Fill));
    if let Some(accessory) = accessory {
        row = row.push(accessory);
    }

    let card = Container::new(row)
        .padding([14, 16])
        .width(Length::Fill)
        .height(Length::Fill)
        .class(style::Container::ToastCard);

    // Auto-dismiss timer line for the "no changes" state.
    let with_timer: Element = if matches!(toast.kind, ToastKind::NoChanges { .. }) {
        let remaining = (1.0 - eased_progress(elapsed, 500, 2500)).max(0.0);
        let timer_bar = portion_bar(remaining, 3, style::Container::Wrapper, style::Container::ToastTimerLine);
        let timer = Container::new(timer_bar)
            .width(Length::Fill)
            .height(Length::Fill)
            .align_y(Alignment::End);
        iced::widget::stack![card, timer].into()
    } else {
        card.into()
    };

    Container::new(with_timer)
        .padding(8)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

fn chip_container<'a>(class: style::Container, content: Element<'a>) -> Element<'a> {
    Container::new(content)
        .width(42)
        .height(42)
        .align_x(Alignment::Center)
        .align_y(Alignment::Center)
        .class(class)
        .into()
}

fn app_icon(size: f32) -> Element<'static> {
    Icon::Upload.text_narrow().size(size).class(style::Text::ToastTitle).into()
}

fn percent_label(percent: u8) -> Element<'static> {
    Row::new()
        .align_y(Alignment::Start)
        .push(text(format!("{percent}")).size(17).class(style::Text::ToastPercent))
        .push(text("%").size(11).class(style::Text::ToastPercentUnit))
        .into()
}

/// Splits a track into a filled and empty portion by relative flex weight.
/// `Length::FillPortion` only distributes space among *siblings* in a Row/Column -
/// alone inside a plain Container it has no effect, so the split must be two Row children.
fn portion_bar(fraction: f32, height: u32, track_class: style::Container, fill_class: style::Container) -> Element<'static> {
    let fraction = fraction.clamp(0.0, 1.0);
    let filled = (fraction * 1000.0).round() as u16;
    let empty = 1000u16.saturating_sub(filled);

    let mut row = Row::new();
    if filled > 0 {
        row = row.push(
            Container::new(Space::new())
                .width(Length::FillPortion(filled))
                .height(Length::Fill)
                .class(fill_class),
        );
    }
    if empty > 0 {
        row = row.push(Space::new().width(Length::FillPortion(empty)));
    }

    Container::new(row)
        .width(Length::Fill)
        .height(height)
        .class(track_class)
        .into()
}

fn progress_bar(fraction: f32) -> Element<'static> {
    portion_bar(fraction, 4, style::Container::ToastBarTrack, style::Container::ToastBarFill)
}

struct CheckProgram {
    /// 0..1 pop-in progress.
    pop: f32,
}

impl<Message> canvas::Program<Message, style::Theme> for CheckProgram {
    type State = ();

    fn draw(
        &self,
        _state: &(),
        renderer: &iced::Renderer,
        _theme: &style::Theme,
        bounds: iced::Rectangle,
        _cursor: iced::mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        let mut frame = canvas::Frame::new(renderer, bounds.size());
        let center = Point::new(bounds.width / 2.0, bounds.height / 2.0);
        let radius = (bounds.width.min(bounds.height) / 2.0) * self.pop;

        frame.fill(&Path::circle(center, radius.max(0.0)), GREEN);

        if self.pop > 0.4 {
            // Checkmark path, scaled/centered to fit the circle (design ref: M4 12l5 5L20 6 in a 24x24 box).
            let scale = (radius * 2.0) / 24.0;
            let to_point = |x: f32, y: f32| Point::new(center.x + (x - 12.0) * scale, center.y + (y - 12.0) * scale);
            let check = Path::new(|builder| {
                builder.move_to(to_point(4.0, 12.0));
                builder.line_to(to_point(9.0, 17.0));
                builder.line_to(to_point(20.0, 6.0));
            });
            frame.stroke(
                &check,
                Stroke {
                    style: stroke::Style::Solid(DARK),
                    width: 3.4 * scale,
                    line_cap: canvas::LineCap::Round,
                    line_join: canvas::LineJoin::Round,
                    ..Stroke::default()
                },
            );
        }

        vec![frame.into_geometry()]
    }
}

fn check_circle(pop: f32) -> Element<'static> {
    Canvas::new(CheckProgram { pop: pop.clamp(0.0, 1.0) }).width(26).height(26).into()
}

fn error_mark() -> Element<'static> {
    text("!").size(18).class(style::Text::ToastError).into()
}

fn error_badge() -> Element<'static> {
    Container::new(text("!").size(14).class(style::Text::ToastCheck))
        .width(22)
        .height(22)
        .align_x(Alignment::Center)
        .align_y(Alignment::Center)
        .class(style::Container::ToastErrorBadge)
        .into()
}

struct SpinnerProgram {
    angle: Radians,
}

impl<Message> canvas::Program<Message, style::Theme> for SpinnerProgram {
    type State = ();

    fn draw(
        &self,
        _state: &(),
        renderer: &iced::Renderer,
        _theme: &style::Theme,
        bounds: iced::Rectangle,
        _cursor: iced::mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        let mut frame = canvas::Frame::new(renderer, bounds.size());
        let center = Point::new(bounds.width / 2.0, bounds.height / 2.0);
        let radius = (bounds.width.min(bounds.height) / 2.0) - 1.0;

        let track = Path::circle(center, radius);
        frame.stroke(
            &track,
            Stroke {
                style: stroke::Style::Solid(PINK.scale_alpha(0.2)),
                width: 2.0,
                ..Stroke::default()
            },
        );

        let sweep = std::f32::consts::TAU * 0.28;
        let arc = Path::new(|builder| {
            builder.arc(canvas::path::Arc {
                center,
                radius,
                start_angle: self.angle,
                end_angle: Radians(self.angle.0 + sweep),
            });
        });
        frame.stroke(
            &arc,
            Stroke {
                style: stroke::Style::Solid(PINK),
                width: 2.0,
                line_cap: canvas::LineCap::Round,
                ..Stroke::default()
            },
        );

        vec![frame.into_geometry()]
    }
}

fn spinner(angle: Radians) -> Element<'static> {
    iced::widget::stack![
        Canvas::new(SpinnerProgram { angle }).width(42).height(42),
        Container::new(app_icon(22.0))
            .width(42)
            .height(42)
            .align_x(Alignment::Center)
            .align_y(Alignment::Center),
    ]
    .into()
}
