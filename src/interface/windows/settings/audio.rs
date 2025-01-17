use procedural::*;

use crate::interface::{InterfaceSettings, PrototypeWindow, Size, Window, WindowBuilder, WindowCache};

#[derive(Default)]
pub struct AudioSettingsWindow {}

impl AudioSettingsWindow {
    pub const WINDOW_CLASS: &'static str = "audio_settings";
}

impl PrototypeWindow for AudioSettingsWindow {
    fn window_class(&self) -> Option<&str> {
        Self::WINDOW_CLASS.into()
    }

    fn to_window(&self, window_cache: &WindowCache, interface_settings: &InterfaceSettings, available_space: Size) -> Window {
        let elements = vec![];

        WindowBuilder::default()
            .with_title("Audio Settings".to_string())
            .with_class(Self::WINDOW_CLASS.to_string())
            .with_size(constraint!(200 > 250 < 300, ?))
            .with_elements(elements)
            .closable()
            .build(window_cache, interface_settings, available_space)
    }
}
