//! Cores e dimensões da interface.

use eframe::egui::Color32;

/// Fração da largura ocupada pela coluna de controle (o resto é do gráfico).
pub(crate) const CONTROL_SHARE: f32 = 0.36;
pub(crate) const CONTROL_MIN_WIDTH: f32 = 330.0;
pub(crate) const CONTROL_MAX_WIDTH: f32 = 440.0;
/// Raio dos cantos do botão de saída, em pixels lógicos.
pub(crate) const OUTPUT_BUTTON_RADIUS: u8 = 5;
/// Folga entre a borda de baixo do botão de saída e a barra inferior.
pub(crate) const OUTPUT_BOTTOM_MARGIN: f32 = 10.0;
/// Espaço extra entre os grupos da linha de campos do Setpoint: entre o campo V e o rótulo A, e
/// entre o campo A e o botão Aplicar. Soma-se ao espaçamento padrão entre widgets (de cada lado).
pub(crate) const FIELD_GAP: f32 = 10.0;
/// Tamanho do texto do quadro de setup das memórias (título, rótulos, valores e unidades).
pub(crate) const SETUP_FONT_SIZE: f32 = 16.0;
/// Largura do seletor de cor da legenda.
pub(crate) const COLOR_PICKER_WIDTH: f32 = 200.0;
/// Largura reservada para cada uma das três escalas à esquerda do gráfico.
pub(crate) const CHART_AXIS_COL_W: f32 = 40.0;

// Cores padrão de V, A e W, usadas nas leituras, nos campos e no gráfico.
pub(crate) const DEFAULT_SERIES_COLORS: [Color32; 3] = [
    Color32::from_rgb(230, 60, 60),
    Color32::from_rgb(60, 200, 90),
    Color32::from_rgb(60, 140, 255),
];

pub(crate) const OK_COLOR: Color32 = Color32::from_rgb(60, 200, 90);
pub(crate) const WARN_COLOR: Color32 = Color32::from_rgb(230, 180, 40);
pub(crate) const ERROR_COLOR: Color32 = Color32::from_rgb(255, 90, 90);
pub(crate) const ACTION_COLOR: Color32 = Color32::from_rgb(60, 140, 255);
