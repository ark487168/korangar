use std::time::{Duration, Instant};

use cgmath::{Vector4, Zero};
use procedural::*;

use crate::debug::*;
use crate::graphics::{InterfaceRenderer, Renderer};
use crate::input::MouseInputMode;
use crate::interface::*;

const VISIBILITY_THRESHHOLD: f32 = 0.01;

#[derive(new)]
pub struct FrameInspectorView {
    #[new(default)]
    state: ElementState,
    measurement: Measurement,
    #[new(default)]
    start_offset: Duration,
    #[new(default)]
    end_offset: Duration,
}

impl FrameInspectorView {
    fn interpolate_alpha_linear(sloap: f32, start: f32, value: f32) -> f32 {
        f32::min(1.0, sloap * (value - start))
    }

    fn interpolate_alpha_smoothed(max: f32, min: f32, value: f32) -> f32 {
        if value < min {
            return 0.0;
        }

        // Clamp the value between 0 and 1 to ensure the interpolation is within bounds
        let normalized_value = num::clamp((value - min) / ((max - min) * 20.0), 0.0, 1.0);

        // Apply a smoothing function for smooth transitions
        1.0 - normalized_value.powf(0.4)
    }

    fn render_lines(
        renderer: &mut ElementRenderer<'_>,
        theme: &Theme,
        text: &str,
        text_width: f32,
        mut x_position: f32,
        distance: f32,
        size: Size,
        alpha: f32,
        render_numbers: bool,
    ) {
        let color = theme.profiler.line_color.multiply_alpha_f32(alpha);

        while x_position < size.x {
            renderer.render_rectangle(
                Position::new(x_position, 0.0),
                Size::new(*theme.profiler.line_width * *renderer.interface_settings.scaling, size.y),
                Vector4::zero(),
                color,
            );

            if render_numbers {
                let offset = Position::new(
                    x_position + (distance - text_width) / 2.0,
                    size.y - *theme.profiler.distance_text_offset * *renderer.interface_settings.scaling,
                );

                renderer.renderer.render_text(
                    renderer.render_target,
                    text,
                    renderer.position + offset,
                    renderer.clip_size,
                    *theme.profiler.line_color,
                    *theme.profiler.distance_text_size * *renderer.interface_settings.scaling,
                );
            }

            x_position += distance;
        }
    }

    fn render_measurement(
        renderer: &mut ElementRenderer<'_>,
        color_lookup: &mut super::ColorLookup,
        theme: &Theme,
        measurement: &Measurement,
        start_time: Instant,
        total_width: f32,
        unit: f32,
        y_position: f32,
    ) {
        const BAR_FADE_SPEED: f32 = 0.05;
        const TEXT_FADE_SPEED: f32 = 0.01;
        // Size in scaled pixels at which the text starts fading in
        const TEXT_DISPLAY_SIZE: f32 = 50.0;

        let scaled_bar_gap = theme.profiler.bar_gap.x * *renderer.interface_settings.scaling;
        let color = color_lookup.get_color(measurement.name);
        let text_offset = *theme.profiler.bar_text_offset * *renderer.interface_settings.scaling;
        let x_position = measurement.start_time.saturating_duration_since(start_time).as_secs_f32() * unit + scaled_bar_gap;
        let x_size = measurement.end_time.saturating_duration_since(start_time).as_secs_f32() * unit - x_position - scaled_bar_gap;
        let x_size = x_size.min(total_width - x_position - scaled_bar_gap);
        let y_size = *theme.profiler.bar_height * *renderer.interface_settings.scaling;

        let alpha = Self::interpolate_alpha_linear(BAR_FADE_SPEED * *renderer.interface_settings.scaling, 0.0, x_size);
        if alpha < VISIBILITY_THRESHHOLD {
            return;
        }

        renderer.render_rectangle(
            Position::new(x_position, y_position),
            Size::new(x_size, y_size),
            *theme.profiler.bar_border_radius,
            color.multiply_alpha_f32(alpha),
        );

        let alpha = Self::interpolate_alpha_linear(
            TEXT_FADE_SPEED * *renderer.interface_settings.scaling,
            TEXT_DISPLAY_SIZE * *renderer.interface_settings.scaling,
            x_size,
        );

        if alpha > VISIBILITY_THRESHHOLD {
            let text = format!("{} ({:?})", measurement.name, measurement.end_time - measurement.start_time);
            let clip_size = ClipSize::new(
                renderer.clip_size.x + x_position,
                renderer.clip_size.y + y_position,
                renderer.clip_size.x + x_position + x_size,
                renderer.clip_size.y + y_position + y_size,
            );

            renderer.renderer.render_text(
                renderer.render_target,
                &text,
                renderer.position + Position::new(x_position, y_position) + text_offset,
                clip_size,
                theme.profiler.bar_text_color.multiply_alpha_f32(alpha),
                *theme.profiler.bar_text_size * *renderer.interface_settings.scaling,
            );
        }

        let y_position = y_position + (theme.profiler.bar_gap.y + *theme.profiler.bar_height) * *renderer.interface_settings.scaling;
        measurement.indices.iter().for_each(|measurement| {
            Self::render_measurement(
                renderer,
                color_lookup,
                theme,
                measurement,
                start_time,
                total_width,
                unit,
                y_position,
            )
        });
    }
}

impl Element for FrameInspectorView {
    fn get_state(&self) -> &ElementState {
        &self.state
    }

    fn get_state_mut(&mut self) -> &mut ElementState {
        &mut self.state
    }

    fn is_focusable(&self) -> bool {
        false
    }

    fn resolve(&mut self, placement_resolver: &mut PlacementResolver, _interface_settings: &InterfaceSettings, _theme: &Theme) {
        let size_constraint = &constraint!(100%, 300);
        self.state.resolve(placement_resolver, size_constraint);
    }

    fn hovered_element(&self, mouse_position: Position, mouse_mode: &MouseInputMode) -> HoverInformation {
        match mouse_mode {
            MouseInputMode::None => self.state.hovered_element(mouse_position),
            _ => HoverInformation::Missed,
        }
    }

    fn scroll(&mut self, delta: f32) -> Option<ChangeEvent> {
        const ZOOM_SPEED: f32 = 0.004;

        let viewed_duration = self.measurement.total_time_taken() - (self.start_offset + self.end_offset);
        let side_bias = (1.0 / self.state.cached_size.x) * self.state.mouse_position.get().x;
        let total_offset = viewed_duration.mul_f32(delta.abs() * ZOOM_SPEED);

        if delta.is_sign_negative() {
            self.end_offset = self.end_offset.saturating_sub(total_offset.mul_f32(side_bias));
            self.start_offset = self.start_offset.saturating_sub(total_offset.mul_f32(1.0 - side_bias));
        } else {
            self.end_offset += total_offset.mul_f32(1.0 - side_bias);
            self.start_offset += total_offset.mul_f32(side_bias);
        }

        Some(ChangeEvent::RerenderWindow)
    }

    fn render(
        &self,
        render_target: &mut <InterfaceRenderer as Renderer>::Target,
        renderer: &InterfaceRenderer,
        _state_provider: &StateProvider,
        interface_settings: &InterfaceSettings,
        theme: &Theme,
        parent_position: Position,
        clip_size: ClipSize,
        _hovered_element: Option<&dyn Element>,
        _focused_element: Option<&dyn Element>,
        _mouse_mode: &MouseInputMode,
        _second_theme: bool,
    ) {
        // Multiple of the size of the distance number that needs to be available to
        // display the number
        const DISTANCE_NUMBER_SIZE: f32 = 1.5;

        let mut renderer = self
            .state
            .element_renderer(render_target, renderer, interface_settings, parent_position, clip_size);

        renderer.render_background(*theme.profiler.border_radius, *theme.profiler.background_color);

        let mut colors = super::ColorLookup::default();

        let start_time = self.measurement.start_time + self.start_offset;
        let end_time = self.measurement.end_time - self.end_offset;
        let viewed_duration = end_time - start_time;

        // We only ever want to display a single unit, so we render from smallest to
        // biggest and keep track when a unit has been rendered.
        let mut numbers_shown = false;

        // Technically we could make this macro simpler by unifying the units used for
        // calculating visibility and offset, but that might introduce more
        // floating point inaccuracy as the values get very small or very large.
        macro_rules! render_lines_size {
            ($alpha_function:ident, $max:expr, $min:expr, $duration:expr, $div_function:ident, $divider:expr, $text:expr) => {
                let visibility = Self::interpolate_alpha_smoothed($max, $min, viewed_duration.$alpha_function() as f32);

                if visibility > VISIBILITY_THRESHHOLD {
                    let distance = $duration.div_duration_f32(viewed_duration) * self.state.cached_size.x;
                    let offset = ((self.start_offset.$div_function() as f32 / $divider) * -distance) % distance;

                    let text_width = renderer
                        .get_text_dimensions($text, *theme.profiler.distance_text_size, f32::MAX)
                        .x;
                    let show_numbers = !numbers_shown && distance > text_width * DISTANCE_NUMBER_SIZE;

                    #[allow(unused_assignments)]
                    {
                        numbers_shown |= show_numbers;
                    }

                    Self::render_lines(
                        &mut renderer,
                        theme,
                        $text,
                        text_width,
                        offset,
                        distance,
                        self.state.cached_size,
                        visibility,
                        show_numbers,
                    );
                }
            };
        }

        render_lines_size!(as_nanos, 20.0, 0.0, Duration::from_nanos(10), as_nanos, 10.0, "10ns");
        render_lines_size!(as_nanos, 200.0, 20.0, Duration::from_nanos(100), as_nanos, 100.0, "100ns");
        render_lines_size!(as_nanos, 2000.0, 200.0, Duration::from_micros(1), as_nanos, 1000.0, "1μs");
        render_lines_size!(as_micros, 20.0, 2.0, Duration::from_micros(10), as_nanos, 10000.0, "10μs");
        render_lines_size!(as_micros, 200.0, 20.0, Duration::from_micros(100), as_nanos, 100000.0, "100μs");
        render_lines_size!(as_micros, 2000.0, 200.0, Duration::from_millis(1), as_micros, 1000.0, "1ms");
        render_lines_size!(as_millis, 20.0, 2.0, Duration::from_millis(10), as_micros, 10000.0, "10ms");
        render_lines_size!(as_millis, 200.0, 20.0, Duration::from_millis(100), as_micros, 100000.0, "100ms");

        Self::render_measurement(
            &mut renderer,
            &mut colors,
            theme,
            &self.measurement,
            start_time,
            self.state.cached_size.x,
            self.state.cached_size.x / viewed_duration.as_secs_f32(),
            theme.profiler.bar_gap.y * *interface_settings.scaling,
        );
    }
}
