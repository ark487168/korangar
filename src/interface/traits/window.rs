use types::maths::Vector2;
use graphics::Renderer;
use interface::types::*;
use interface::traits::Element;

pub trait Window {

    fn window_class_matches(&self, other_window_class: &str) -> bool;

    fn resolve(&mut self, interface_settings: &InterfaceSettings, theme: &Theme, avalible_space: Size) -> (Option<&str>, Vector2<f32>, Size);

    fn update(&mut self) -> Option<ChangeEvent>;

    fn hovered_element(&self, mouse_position: Vector2<f32>) -> HoverInformation;

    fn get_area(&self) -> (Position, Size);

    fn hovers_area(&self, position: Position, size: Size) -> bool;

    fn offset(&mut self, avalible_space: Size, offset: Vector2<f32>) -> Option<(&str, Vector2<f32>)>;

    fn validate_position(&mut self, avalible_space: Size);

    fn resize(&mut self, interface_settings: &InterfaceSettings, theme: &Theme, avalible_space: Size, growth: Vector2<f32>) -> (Option<&str>, Size);

    fn validate_size(&mut self, interface_settings: &InterfaceSettings, avalible_space: Size);

    fn render(&self, renderer: &mut Renderer, state_provider: &StateProvider, interface_settings: &InterfaceSettings, theme: &Theme, hovered_element: Option<&dyn Element>);
}
