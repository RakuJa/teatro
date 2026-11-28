use crate::backend::pad_handler::KNOB_INCREMENT;
use crate::gui::theme;
use crate::states::filter_data::FilterKind;
use egui::{
    Align2, Color32, CursorIcon, Event, FontId, Id, Key, Modifiers, MouseWheelUnit, Pos2, Rect,
    Response, Sense, Shape, Stroke, TextEdit, Ui, Vec2,
};

const DRAG_POINTS_PER_TICK: f32 = 1.0;
const FINE_DRAG_FACTOR: f32 = 0.25;
const TICKS_PER_WHEEL_NOTCH: f32 = 2.0;
const POINTS_PER_WHEEL_NOTCH: f32 = 40.0;
const KEY_TICKS: i32 = 1;
const KEY_TICKS_COARSE: i32 = 10;

const START_DEG: f32 = 135.0;
const SWEEP_DEG: f32 = 270.0;

#[derive(Clone, Copy)]
pub enum Channel {
    Music,
    Ambience,
    Effects,
}

impl Channel {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Music => "MUSIC",
            Self::Ambience => "AMBIENCE",
            Self::Effects => "SFX",
        }
    }

    pub const fn color(self) -> Color32 {
        match self {
            Self::Music => theme::MUSIC,
            Self::Ambience => theme::AMBIENCE,
            Self::Effects => theme::EFFECTS,
        }
    }
}

pub struct KnobSpec {
    pub index: u8,
    pub channel: Channel,
    pub function: &'static str,
}

pub const KNOBS: [KnobSpec; 8] = [
    KnobSpec {
        index: 1,
        channel: Channel::Music,
        function: "Volume",
    },
    KnobSpec {
        index: 2,
        channel: Channel::Music,
        function: "Low-pass",
    },
    KnobSpec {
        index: 3,
        channel: Channel::Music,
        function: "High-pass",
    },
    KnobSpec {
        index: 4,
        channel: Channel::Music,
        function: "Soft low-pass",
    },
    KnobSpec {
        index: 5,
        channel: Channel::Ambience,
        function: "Volume",
    },
    KnobSpec {
        index: 6,
        channel: Channel::Ambience,
        function: "Low-pass",
    },
    KnobSpec {
        index: 7,
        channel: Channel::Effects,
        function: "Volume",
    },
    KnobSpec {
        index: 8,
        channel: Channel::Effects,
        function: "High-pass",
    },
];

pub fn ticks_between(current: f32, target: f32) -> i32 {
    ((target.clamp(0.0, 1.0) - current) / KNOB_INCREMENT).round() as i32
}

/// Adds `amount` to a fractional accumulator stored under `id` and returns the
/// whole ticks it now contains, keeping the remainder for the next frame.
fn take_whole_ticks(ui: &Ui, id: Id, amount: f32) -> i32 {
    if amount == 0.0 {
        return 0;
    }
    let acc = ui.data(|d| d.get_temp::<f32>(id).unwrap_or(0.0)) + amount;
    let whole = acc.trunc();
    ui.data_mut(|d| d.insert_temp(id, acc - whole));
    whole as i32
}

fn wheel_notches(ui: &Ui) -> (f32, bool) {
    ui.input(|i| {
        let mut notches = 0.0;
        let mut fine = false;
        for event in &i.events {
            if let Event::MouseWheel {
                unit,
                delta,
                modifiers,
                ..
            } = event
            {
                let d = delta.y + delta.x;
                notches += match unit {
                    MouseWheelUnit::Point => d / POINTS_PER_WHEEL_NOTCH,
                    MouseWheelUnit::Line => d,
                    MouseWheelUnit::Page => d * 10.0,
                };
                fine |= modifiers.shift;
            }
        }
        (notches, fine)
    })
}

fn keyboard_ticks(ui: &Ui) -> i32 {
    ui.input_mut(|i| {
        let mut ticks = 0;
        for (key, dir) in [
            (Key::ArrowUp, 1),
            (Key::ArrowRight, 1),
            (Key::ArrowDown, -1),
            (Key::ArrowLeft, -1),
        ] {
            if i.consume_key(Modifiers::SHIFT, key) {
                ticks += dir * KEY_TICKS_COARSE;
            } else if i.consume_key(Modifiers::NONE, key) {
                ticks += dir * KEY_TICKS;
            }
        }
        ticks
    })
}

const fn angle_for(value: f32) -> f32 {
    SWEEP_DEG
        .mul_add(value.clamp(0.0, 1.0), START_DEG)
        .to_radians()
}

fn point_at(center: Pos2, radius: f32, angle: f32) -> Pos2 {
    Pos2::new(
        radius.mul_add(angle.cos(), center.x),
        radius.mul_add(angle.sin(), center.y),
    )
}

fn arc(center: Pos2, radius: f32, from: f32, to: f32) -> Vec<Pos2> {
    const STEPS: usize = 45;
    let steps = STEPS;
    (0..=steps)
        .map(|i| {
            point_at(
                center,
                radius,
                (to - from).mul_add(i as f32 / steps as f32, from),
            )
        })
        .collect()
}

fn paint_knob(
    ui: &Ui,
    response: &Response,
    center: Pos2,
    radius: f32,
    value: f32,
    color: Color32,
    scale: f32,
) {
    let painter = ui.painter();
    let active = response.hovered() || response.dragged();
    let hover_t = ui
        .ctx()
        .animate_bool_with_time(response.id.with("hover"), active, 0.1);

    let ring_r = 6.0f32.mul_add(scale, radius);
    let ring_w = 3.5 * scale;
    let start = angle_for(0.0);
    let end = angle_for(1.0);
    let current = angle_for(value);

    painter.add(Shape::line(
        arc(center, ring_r, start, end),
        Stroke::new(ring_w, theme::BORDER_STRONG),
    ));
    if value > 0.0 {
        painter.add(Shape::line(
            arc(center, ring_r, start, current),
            Stroke::new(ring_w, color),
        ));
    }

    if response.has_focus() {
        painter.circle_stroke(
            center,
            11.0f32.mul_add(scale, radius),
            Stroke::new(1.5 * scale, color.gamma_multiply(0.6)),
        );
    }

    let body = Color32::from_rgb(46, 46, 56).lerp_to_gamma(Color32::from_rgb(60, 60, 74), hover_t);
    painter.circle_filled(center, radius, body);
    painter.circle_filled(
        center,
        radius * 0.8,
        Color32::from_rgb(58, 58, 70).lerp_to_gamma(Color32::from_rgb(72, 72, 88), hover_t),
    );
    painter.circle_stroke(
        center,
        radius,
        Stroke::new(1.0 * scale, Color32::from_rgb(78, 78, 92)),
    );

    painter.line_segment(
        [
            point_at(center, radius * 0.3, current),
            point_at(center, radius * 0.78, current),
        ],
        Stroke::new(3.0 * scale, theme::TEXT),
    );
}

pub fn knob(
    ui: &Ui,
    id: Id,
    center: Pos2,
    radius: f32,
    value: f32,
    color: Color32,
    scale: f32,
) -> (Response, i32) {
    let hit = Rect::from_center_size(center, Vec2::splat(radius.mul_add(2.0, 12.0 * scale)));
    let response = ui
        .interact(hit, id, Sense::click_and_drag())
        .on_hover_cursor(CursorIcon::ResizeVertical);
    if response.clicked() || response.drag_started() {
        response.request_focus();
    }

    let fine = ui.input(|i| i.modifiers.shift);
    let mut ticks = 0;

    if response.dragged() {
        let d = response.drag_delta();
        let per_tick = DRAG_POINTS_PER_TICK * scale / if fine { FINE_DRAG_FACTOR } else { 1.0 };
        // Up or right turns clockwise.
        ticks += take_whole_ticks(ui, id.with("drag_acc"), (d.x - d.y) / per_tick);
    } else if response.drag_stopped() {
        ui.data_mut(|d| d.remove::<f32>(id.with("drag_acc")));
    }

    if response.hovered() {
        let (notches, fine_wheel) = wheel_notches(ui);
        let per_notch = if fine_wheel {
            1.0
        } else {
            TICKS_PER_WHEEL_NOTCH
        };
        ticks += take_whole_ticks(ui, id.with("wheel_acc"), notches * per_notch);
    }

    if response.has_focus() {
        // Keep the arrow keys for the knob instead of egui's focus navigation.
        ui.memory_mut(|m| {
            m.set_focus_lock_filter(
                id,
                egui::EventFilter {
                    horizontal_arrows: true,
                    vertical_arrows: true,
                    ..Default::default()
                },
            );
        });
        ticks += keyboard_ticks(ui);
    }

    paint_knob(ui, &response, center, radius, value, color, scale);
    (response, ticks)
}

pub fn value_field(
    ui: &mut Ui,
    id: Id,
    rect: Rect,
    value: f32,
    filter: Option<FilterKind>,
    scale: f32,
) -> i32 {
    let edit_id = id.with("edit_text");
    let editing: Option<String> = ui.data(|d| d.get_temp(edit_id));

    if let Some(mut text) = editing {
        let response = ui.put(
            rect,
            TextEdit::singleline(&mut text)
                .hint_text(format_knob_value(value, filter))
                .font(FontId::proportional(12.0 * scale))
                .horizontal_align(egui::Align::Center)
                .background_color(theme::INPUT_BG)
                .margin(Vec2::new(4.0, 2.0)),
        );
        if !response.has_focus() && !response.lost_focus() {
            response.request_focus();
        }
        let (enter, escape) = ui.input(|i| (i.key_pressed(Key::Enter), i.key_pressed(Key::Escape)));
        if escape {
            ui.data_mut(|d| d.remove::<String>(edit_id));
            return 0;
        }
        if enter || response.lost_focus() {
            ui.data_mut(|d| d.remove::<String>(edit_id));
            return parse_knob_input(&text, filter)
                .map_or(0, |target| ticks_between(value, target));
        }
        ui.data_mut(|d| d.insert_temp(edit_id, text));
        return 0;
    }

    let response = ui
        .interact(rect, id.with("value"), Sense::click())
        .on_hover_cursor(CursorIcon::Text)
        .on_hover_text(if filter.is_some() {
            "Click to type a value: 40%, 800 Hz, 2.5k or off"
        } else {
            "Click to type an exact value"
        });
    let painter = ui.painter();
    if response.hovered() {
        painter.rect_filled(rect, 4.0 * scale, theme::INPUT_BG);
        painter.rect_stroke(
            rect,
            4.0 * scale,
            Stroke::new(1.0, theme::BORDER_STRONG),
            egui::StrokeKind::Inside,
        );
    }
    painter.text(
        rect.center(),
        Align2::CENTER_CENTER,
        format_knob_value(value, filter),
        FontId::proportional(13.0 * scale),
        theme::TEXT,
    );
    if response.clicked() {
        ui.data_mut(|d| d.insert_temp(edit_id, String::new()));
    }
    0
}

pub fn format_knob_value(value: f32, filter: Option<FilterKind>) -> String {
    match filter.map(|f| f.cutoff_hz(value)) {
        None => format_percent(value),
        Some(None) => "Off".to_owned(),
        Some(Some(hz)) if hz >= 1_000.0 => format!("{:.1} kHz", hz / 1_000.0),
        Some(Some(hz)) => format!("{hz:.0} Hz"),
    }
}

pub fn parse_knob_input(text: &str, filter: Option<FilterKind>) -> Option<f32> {
    let t = text.trim().to_lowercase().replace(',', ".");
    if let Some(kind) = filter {
        if t == "off" {
            return Some(0.0);
        }
        let frequency = t
            .strip_suffix("khz")
            .or_else(|| t.strip_suffix('k'))
            .map(|n| (n, 1_000.0))
            .or_else(|| t.strip_suffix("hz").map(|n| (n, 1.0)));
        if let Some((number, multiplier)) = frequency {
            return number
                .trim()
                .parse::<f32>()
                .ok()
                .map(|hz| kind.knob_for_cutoff(hz * multiplier).max(KNOB_INCREMENT));
        }
    }
    t.trim_end_matches('%')
        .trim()
        .parse::<f32>()
        .ok()
        .filter(|p| p.is_finite())
        .map(|pct| (pct / 100.0).clamp(0.0, 1.0))
}

pub fn format_percent(value: f32) -> String {
    let pct = value * 100.0;
    if (pct - pct.round()).abs() < 0.05 {
        format!("{pct:.0}%")
    } else {
        format!("{pct:.1}%")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ticks_between_matches_encoder_steps() {
        assert_eq!(ticks_between(0.0, 1.0), 200);
        assert_eq!(ticks_between(0.5, 0.25), -50);
        assert_eq!(ticks_between(0.1, 0.1), 0);
        assert_eq!(ticks_between(0.9, 2.0), 20);
    }

    #[test]
    fn filter_values_show_cutoff() {
        let lp = Some(FilterKind::LowPass);
        assert_eq!(format_knob_value(0.0, lp), "Off");
        assert_eq!(format_knob_value(0.5, lp), "2.0 kHz");
        assert_eq!(format_knob_value(1.0, lp), "200 Hz");
        assert_eq!(format_knob_value(0.1, None), "10%");
    }

    #[test]
    fn typed_values_accept_percent_and_frequency() {
        let lp = Some(FilterKind::LowPass);
        assert_eq!(parse_knob_input("50", lp), Some(0.5));
        assert_eq!(parse_knob_input(" 25 % ", None), Some(0.25));
        assert_eq!(parse_knob_input("off", lp), Some(0.0));
        let k = parse_knob_input("2k", lp).unwrap();
        assert!((k - 0.5).abs() < 1e-4);
        let hz = parse_knob_input("2000 Hz", lp).unwrap();
        assert!((hz - 0.5).abs() < 1e-4);
        assert!(parse_knob_input("2.0 khz", lp).is_some());
        assert!(parse_knob_input("20 kHz", lp).unwrap() > 0.0);
        assert_eq!(parse_knob_input("800 Hz", None), None);
        assert_eq!(parse_knob_input("abc", None), None);
    }

    #[test]
    fn percent_formatting() {
        assert_eq!(format_percent(0.1), "10%");
        assert_eq!(format_percent(0.105), "10.5%");
        assert_eq!(format_percent(0.0), "0%");
    }

    struct Harness {
        ctx: egui::Context,
        time: f64,
    }

    const CENTER: Pos2 = Pos2::new(100.0, 100.0);

    impl Harness {
        fn new() -> Self {
            let mut h = Self {
                ctx: egui::Context::default(),
                time: 0.0,
            };
            h.frame(vec![]);
            h
        }

        fn frame(&mut self, events: Vec<Event>) -> i32 {
            self.time += 1.0 / 60.0;
            let input = egui::RawInput {
                screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::splat(400.0))),
                time: Some(self.time),
                events,
                ..Default::default()
            };
            let mut ticks = 0;
            self.ctx
                .run_ui(input, |ui| {
                    let (_, t) = knob(ui, Id::new("k"), CENTER, 24.0, 0.5, Color32::WHITE, 1.0);
                    ticks += t;
                })
                .drop_without_applying_deltas();
            ticks
        }

        fn button(&mut self, pos: Pos2, pressed: bool) -> i32 {
            self.frame(vec![Event::PointerButton {
                pos,
                button: egui::PointerButton::Primary,
                pressed,
                modifiers: Modifiers::NONE,
            }])
        }

        fn key(&mut self, key: Key, modifiers: Modifiers) -> i32 {
            self.frame(vec![
                Event::ModifiersChanged(modifiers),
                Event::Key {
                    key,
                    physical_key: None,
                    pressed: true,
                    repeat: false,
                    modifiers,
                },
            ])
        }

        fn wheel(&mut self, lines: f32, modifiers: Modifiers) -> i32 {
            self.frame(vec![
                Event::ModifiersChanged(modifiers),
                Event::MouseWheel {
                    unit: MouseWheelUnit::Line,
                    delta: Vec2::new(0.0, lines),
                    phase: egui::TouchPhase::Move,
                    modifiers,
                },
            ])
        }
    }

    #[test]
    fn dragging_up_turns_clockwise_one_tick_per_point() {
        let mut h = Harness::new();
        let mut ticks = h.frame(vec![Event::PointerMoved(CENTER)]);
        ticks += h.button(CENTER, true);
        for step in 1..=10 {
            ticks += h.frame(vec![Event::PointerMoved(
                CENTER - Vec2::new(0.0, 8.0 * step as f32),
            )]);
        }
        ticks += h.button(CENTER - Vec2::new(0.0, 80.0), false);
        assert!((70..=80).contains(&ticks), "got {ticks}");

        let mut down = h.button(CENTER, true);
        for step in 1..=10 {
            down += h.frame(vec![Event::PointerMoved(
                CENTER + Vec2::new(0.0, 8.0 * step as f32),
            )]);
        }
        assert!(down < -60, "got {down}");
    }

    #[test]
    fn wheel_turns_two_ticks_per_notch_and_one_with_shift() {
        let mut h = Harness::new();
        h.frame(vec![Event::PointerMoved(CENTER)]);
        assert_eq!(h.wheel(3.0, Modifiers::NONE), 6);
        assert_eq!(h.wheel(-2.0, Modifiers::SHIFT), -2);
    }

    #[test]
    fn arrow_keys_turn_a_focused_knob() {
        let mut h = Harness::new();
        h.frame(vec![Event::PointerMoved(CENTER)]);
        h.button(CENTER, true);
        h.button(CENTER, false);
        h.frame(vec![]);
        assert_eq!(h.key(Key::ArrowUp, Modifiers::NONE), 1);
        assert_eq!(h.key(Key::ArrowLeft, Modifiers::NONE), -1);
        assert_eq!(h.key(Key::ArrowRight, Modifiers::SHIFT), 10);
    }

    #[test]
    fn knob_indices_are_sequential() {
        for (i, k) in KNOBS.iter().enumerate() {
            assert_eq!(usize::from(k.index), i + 1);
        }
    }
}
