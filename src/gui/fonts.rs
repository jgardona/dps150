//! Fonte da interface: a Noto Sans, embutida no executável.

use eframe::egui::{self, FontData, FontDefinitions, FontFamily};
use std::sync::Arc;

/// Nome da fonte nas definições do egui.
const NOTO_SANS_NAME: &str = "noto-sans";
/// A Noto Sans (arquivo variável, licença SIL OFL 1.1 em `assets/fonts/OFL.txt`). Fica dentro do
/// executável, então compilar e rodar nunca dependem de fonte instalada no sistema. O eixo `wght`
/// tem padrão 400 (peso normal), que é o que o egui usa sem pedir outro.
const NOTO_SANS_BYTES: &[u8] = include_bytes!("../../assets/fonts/NotoSans-VariableFont_wght.ttf");

/// Definições de fonte do egui com a Noto Sans na frente de `Proportional`.
///
/// As fontes do egui continuam atrás dela: um caractere que a Noto Sans não tem (por exemplo, o
/// ✓) cai na fonte de reserva, em vez de virar quadradinho. `Monospace` não muda.
fn definitions() -> FontDefinitions {
    let mut fonts = FontDefinitions::default();
    fonts.font_data.insert(
        NOTO_SANS_NAME.to_owned(),
        Arc::new(FontData::from_static(NOTO_SANS_BYTES)),
    );
    if let Some(proportional) = fonts.families.get_mut(&FontFamily::Proportional) {
        proportional.insert(0, NOTO_SANS_NAME.to_owned());
    }
    fonts
}

/// Instala a Noto Sans como fonte padrão. Deve ser chamada antes do primeiro quadro.
pub fn install_fonts(ctx: &egui::Context) {
    ctx.set_fonts(definitions());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn noto_sans_e_a_primeira_da_familia_proporcional() {
        let fonts = definitions();
        assert!(fonts.font_data.contains_key(NOTO_SANS_NAME));
        let proportional = &fonts.families[&FontFamily::Proportional];
        assert_eq!(
            proportional.first().map(String::as_str),
            Some(NOTO_SANS_NAME)
        );
    }

    #[test]
    fn fontes_de_reserva_do_egui_continuam_atras() {
        let padrao = FontDefinitions::default().families[&FontFamily::Proportional].clone();
        let atual = &definitions().families[&FontFamily::Proportional];
        assert_eq!(&atual[1..], padrao.as_slice());
    }

    #[test]
    fn monospace_nao_muda() {
        assert_eq!(
            definitions().families[&FontFamily::Monospace],
            FontDefinitions::default().families[&FontFamily::Monospace]
        );
    }

    #[test]
    fn bytes_embutidos_sao_uma_fonte_truetype() {
        assert_eq!(&NOTO_SANS_BYTES[..4], &[0x00, 0x01, 0x00, 0x00]);
        assert!(NOTO_SANS_BYTES.len() > 100 * 1024);
    }

    #[test]
    fn bytes_embutidos_sao_os_do_arquivo_em_assets() {
        let caminho = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/assets/fonts/NotoSans-VariableFont_wght.ttf"
        );
        let arquivo = std::fs::read(caminho).expect("fonte em assets/fonts");
        assert_eq!(arquivo.as_slice(), NOTO_SANS_BYTES);
    }

    /// Largura de "Setup M1 não" em 16 px, medida num contexto com as fontes dadas.
    fn text_width(fonts: Option<FontDefinitions>) -> f32 {
        let ctx = egui::Context::default();
        if let Some(fonts) = fonts {
            ctx.set_fonts(fonts);
        }
        let mut width = 0.0;
        // Duas passagens: a fonte nova só vale a partir do quadro seguinte ao `set_fonts`.
        for _ in 0..2 {
            let _ = ctx.run_ui(egui::RawInput::default(), |ui| {
                let galley = ui.painter().layout_no_wrap(
                    "Setup M1 n\u{e3}o \u{b0}C".to_owned(),
                    egui::FontId::proportional(16.0),
                    egui::Color32::WHITE,
                );
                width = galley.size().x;
            });
        }
        width
    }

    #[test]
    fn a_noto_sans_e_de_fato_usada_para_desenhar() {
        let padrao = text_width(None);
        let noto = text_width(Some(definitions()));
        assert!(padrao > 0.0 && noto > 0.0);
        assert!((padrao - noto).abs() > 1.0, "{padrao} == {noto}");
    }

    /// Largura, em 16 px na Noto Sans, de cada texto dado.
    fn widths(texts: &[String]) -> Vec<f32> {
        let ctx = egui::Context::default();
        ctx.set_fonts(definitions());
        let mut out = Vec::new();
        for _ in 0..2 {
            out.clear();
            let _ = ctx.run_ui(egui::RawInput::default(), |ui| {
                for text in texts {
                    let galley = ui.painter().layout_no_wrap(
                        text.clone(),
                        egui::FontId::proportional(crate::gui::theme::SETUP_FONT_SIZE),
                        egui::Color32::WHITE,
                    );
                    out.push(galley.size().x);
                }
            });
        }
        out
    }

    #[test]
    fn os_seis_maximos_cabem_na_largura_minima_da_coluna() {
        use crate::gui::theme::CONTROL_MIN_WIDTH;
        use crate::logics::memory_setup::SetupField;

        let texts: Vec<String> = SetupField::ALL
            .iter()
            .map(|f| format!("{:.*} {}", f.decimals(), f.max(), f.unit()))
            .collect();
        let w = widths(&texts);
        // Três colunas; cada uma tem a largura do maior valor das duas linhas.
        let colunas: f32 = (0..3).map(|c| w[c].max(w[c + 3])).sum();
        // Folga: preenchimento de cada campo (2 × 4 px), espaço entre colunas (2 × 14 px) e as
        // margens do card (2 × 8 px).
        let total = colunas + 3.0 * 8.0 + 2.0 * 14.0 + 2.0 * 8.0;
        assert!(total <= CONTROL_MIN_WIDTH, "{total} > {CONTROL_MIN_WIDTH}");
    }
}
