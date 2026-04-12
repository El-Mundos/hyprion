use serde::Deserialize;

mod color;
pub use color::Color;

#[derive(Debug, Deserialize)]
pub struct Theme {
    pub colors: ColorScheme,
    pub spacing: Spacing,
    pub radius: Radius,
}

#[derive(Debug, Deserialize)]
pub struct ColorScheme {
    pub mode: ThemeMode,
    pub background: Color,
    pub surface: Color,
    pub text: Color,
    pub text_muted: Color,
    pub accent: Color,
    pub border: Color,
}

#[derive(Debug, Deserialize)]
pub enum ThemeMode {
    Dark,
    Light,
}

#[derive(Debug, Deserialize)]
pub struct Spacing {
    pub xs: f32,
    pub sm: f32,
    pub md: f32,
    pub lg: f32,
    pub xl: f32,
}

#[derive(Debug, Deserialize)]
pub struct Radius {
    pub sm: f32,
    pub md: f32,
    pub lg: f32,
}
