//! Barra superior, coluna de controle e barra inferior.

use super::{
    theme::{
        ACTION_COLOR, ERROR_COLOR, FIELD_GAP, OK_COLOR, OUTPUT_BOTTOM_MARGIN, OUTPUT_BUTTON_RADIUS,
        SETUP_FONT_SIZE, WARN_COLOR,
    },
    widgets::{ReadoutRow, card, info_pair, setup_field, status_dot},
};
use crate::logics::{
    app::{AppState, ConnectionState, SaveStatus},
    format::format_hhmmss,
    memory_setup::{ISET_MAX, SetupField, VSET_MAX},
};
use eframe::egui::{self, Color32};
use std::time::Instant;

pub(crate) fn top_bar(ui: &mut egui::Ui, app: &mut AppState) {
    let is_idle = app.is_idle();

    ui.horizontal(|ui| {
        let (status_color, status_text) = match app.connection() {
            ConnectionState::Disconnected => (Color32::from_gray(110), "Desconectado"),
            ConnectionState::Connecting => (WARN_COLOR, "Conectando..."),
            ConnectionState::Connected => (OK_COLOR, "Conectado"),
            ConnectionState::Error(_) => (ERROR_COLOR, "Erro"),
        };
        status_dot(ui, status_color).on_hover_text(status_text);

        ui.add_enabled_ui(is_idle, |ui| {
            let selected_text = if app.selected_port().is_empty() {
                "-- porta --".to_owned()
            } else {
                app.selected_port().to_owned()
            };
            egui::ComboBox::from_id_salt("serial_port_combo")
                .selected_text(selected_text)
                .show_ui(ui, |ui| {
                    for port in app.available_ports().to_vec() {
                        ui.selectable_value(app.selected_port_mut(), port.clone(), port);
                    }
                });
        });

        let label = match app.connection() {
            ConnectionState::Disconnected | ConnectionState::Error(_) => "Conectar",
            ConnectionState::Connecting => "Cancelar",
            ConnectionState::Connected => "Desconectar",
        };
        if ui.button(label).clicked() {
            if is_idle {
                let ctx = ui.ctx().clone();
                app.connect(move || ctx.request_repaint());
            } else {
                app.disconnect();
            }
        }

        if let ConnectionState::Error(msg) = app.connection() {
            ui.colored_label(ERROR_COLOR, msg);
        }

        if !app.is_connected() {
            return;
        }
        let device = app.device();
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            info_pair(
                ui,
                "Temp",
                &format!("{:.0}\u{b0}C", device.temperature.unwrap_or(0.0)),
            );
            ui.separator();
            info_pair(
                ui,
                "Entrada",
                &format!("{:.2} V", device.input_voltage.unwrap_or(0.0)),
            );
            ui.separator();
            if let Some(fw) = &device.firmware_version {
                info_pair(ui, "fw", fw);
            }
            if let Some(model) = &device.model_name {
                ui.label(egui::RichText::new(model).strong());
            }
        });
    });
}

pub(crate) fn control_column(ui: &mut egui::Ui, app: &mut AppState, colors: &[Color32; 3]) {
    readouts(ui, app, colors);
    ui.add_space(8.0);
    setpoint_card(ui, app, colors);
    ui.add_space(8.0);
    setup_card(ui, app, colors);
    ui.with_layout(egui::Layout::bottom_up(egui::Align::Min), |ui| {
        output_button(ui, app);
    });
}

fn readouts(ui: &mut egui::Ui, app: &AppState, colors: &[Color32; 3]) {
    let device = app.device();
    let mode = device.cc_cv.as_deref();
    ui.add(ReadoutRow {
        unit: "V",
        accent: colors[0],
        value: format!("{:05.2}", device.output_voltage.unwrap_or(0.0)),
        badge: (mode == Some("CV")).then_some("CV"),
    });
    ui.add_space(4.0);
    ui.add(ReadoutRow {
        unit: "A",
        accent: colors[1],
        value: format!("{:05.3}", device.output_current.unwrap_or(0.0)),
        badge: (mode == Some("CC")).then_some("CC"),
    });
    ui.add_space(4.0);
    ui.add(ReadoutRow {
        unit: "W",
        accent: colors[2],
        value: format!("{:05.2}", device.output_power.unwrap_or(0.0)),
        badge: None,
    });
}

/// Linha do título do card Setpoint: "Setpoint" e, com pendência, o aviso e o botão Desfazer.
/// Devolve a resposta do Desfazer (`Some` só com pendência).
///
/// O aviso e o botão vivem na linha do título, que existe com ou sem pendência. A altura mínima
/// da linha é a de um botão, para o card não crescer quando a pendência aparece e some.
///
/// `align_x` é a posição horizontal (em pontos da tela) onde o Desfazer deve começar: a do botão
/// Aplicar, na linha de baixo, para os dois ficarem alinhados. Se o aviso já passou dessa posição,
/// o botão vem logo depois dele.
fn setpoint_title(ui: &mut egui::Ui, dirty: bool, align_x: Option<f32>) -> Option<egui::Response> {
    ui.horizontal(|ui| {
        let button_height =
            ui.text_style_height(&egui::TextStyle::Button) + 2.0 * ui.spacing().button_padding.y;
        // O botão arredonda a altura do texto para cima; sem isso a linha cresceria uns 0,3 px.
        ui.set_min_height(button_height.ceil().max(ui.spacing().interact_size.y));
        ui.label(egui::RichText::new("Setpoint").strong());
        if !dirty {
            return None;
        }
        ui.colored_label(WARN_COLOR, "\u{2022} não aplicado");
        if let Some(x) = align_x {
            // `next_widget_position` já inclui o espaçamento que o egui põe antes do próximo item,
            // e o `add_space` não o repete: a distância que falta é exatamente esta (conferido
            // pelo teste `desfazer_comeca_na_posicao_do_aplicar`).
            let gap = x - ui.next_widget_position().x;
            if gap > 0.0 {
                ui.add_space(gap);
            }
        }
        let width = action_button_width(ui);
        Some(ui.add(egui::Button::new("Desfazer").min_size(egui::vec2(width, 0.0))))
    })
    .inner
}

/// Largura comum dos botões Aplicar e Desfazer: a do mais largo dos dois textos, com o preenchimento
/// do botão. Os dois usam essa largura mínima para ficarem do mesmo tamanho, qualquer que seja a
/// fonte, em vez de um número fixo.
fn action_button_width(ui: &egui::Ui) -> f32 {
    let font = egui::TextStyle::Button.resolve(ui.style());
    let text_width = |text: &str| {
        ui.painter()
            .layout_no_wrap(text.to_owned(), font.clone(), Color32::WHITE)
            .size()
            .x
    };
    text_width("Aplicar").max(text_width("Desfazer")) + 2.0 * ui.spacing().button_padding.x
}

/// Onde o botão Aplicar começou no quadro anterior (guardado na memória temporária do egui).
fn apply_button_x_id() -> egui::Id {
    egui::Id::new("setpoint_apply_button_x")
}

fn setpoint_card(ui: &mut egui::Ui, app: &mut AppState, colors: &[Color32; 3]) {
    card(ui, |ui| {
        // O Desfazer se alinha ao Aplicar, que fica na linha de baixo: usa a posição dele no
        // quadro anterior (o Aplicar é desenhado a cada quadro, então ela sempre existe).
        let apply_x_known = ui.data(|d| d.get_temp::<f32>(apply_button_x_id()));
        let undo = setpoint_title(ui, app.setpoint().is_dirty(), apply_x_known);
        if undo.is_some_and(|r| r.clicked()) {
            app.discard_setpoint_edits();
        }

        let mut enter_pressed = false;
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("V").color(colors[0]).strong());
            let v_resp = ui.add(
                egui::DragValue::new(&mut app.setpoint_mut().vset_input)
                    .speed(0.05)
                    .range(0.0..=VSET_MAX)
                    .fixed_decimals(2)
                    .suffix(" V"),
            );
            ui.add_space(FIELD_GAP);
            ui.label(egui::RichText::new("A").color(colors[1]).strong());
            let c_resp = ui.add(
                egui::DragValue::new(&mut app.setpoint_mut().cset_input)
                    .speed(0.01)
                    .range(0.0..=ISET_MAX)
                    .fixed_decimals(3)
                    .suffix(" A"),
            );

            if v_resp.changed() || c_resp.changed() {
                app.mark_setpoint_edited();
            }
            app.set_editing(
                v_resp.dragged() || v_resp.has_focus(),
                c_resp.dragged() || c_resp.has_focus(),
            );
            enter_pressed = (v_resp.lost_focus() || c_resp.lost_focus())
                && ui.input(|i| i.key_pressed(egui::Key::Enter));

            // Mesmo espaço que separa o campo V do rótulo A.
            ui.add_space(FIELD_GAP);
            let dirty = app.setpoint().is_dirty();
            let apply = egui::Button::new(
                egui::RichText::new("Aplicar")
                    .color(Color32::BLACK)
                    .strong(),
            )
            .min_size(egui::vec2(action_button_width(ui), 0.0))
            .fill(if dirty {
                ACTION_COLOR
            } else {
                Color32::from_gray(90)
            });
            let apply_resp = ui
                .add_enabled(app.is_connected(), apply)
                .on_hover_text("Envia Vset/Iset ao aparelho (ou Enter)");
            let apply_x = apply_resp.rect.left();
            // Se o Aplicar andou (a largura dos campos mudou), o Desfazer usou a posição velha neste
            // quadro: pede mais um quadro para ele se alinhar mesmo sem nenhum evento do usuário.
            if apply_x_known.is_none_or(|old| (old - apply_x).abs() > 0.5) {
                ui.ctx().request_repaint();
            }
            ui.data_mut(|d| d.insert_temp(apply_button_x_id(), apply_x));
            let clicked = apply_resp.clicked();
            if clicked || (enter_pressed && app.is_connected()) {
                app.apply_setpoints();
            }
        });

        ui.separator();

        ui.horizontal(|ui| {
            ui.label("Memória");
            for n in 1..=6u8 {
                let hover = match app.group_values(n) {
                    Some((v, c)) => format!("M{n}: {v:.2} V / {c:.3} A — clique para ver no setup"),
                    None => format!("M{n}: sem dados"),
                };
                if ui
                    .selectable_label(app.selected_profile() == n, format!("M{n}"))
                    .on_hover_text(hover)
                    .clicked()
                {
                    app.request_select_memory(n);
                }
            }
            let target = app.selected_profile();
            let problem = app
                .memory_setup()
                .validate(target)
                .err()
                .and_then(|errors| errors.first().map(|e| e.message()));
            let hover = problem.clone().unwrap_or_else(|| {
                format!("Grava os seis valores do setup em M{target} (OVP, OCP, OPP e OTP valem para o aparelho todo)")
            });
            if ui
                .add_enabled(
                    app.is_connected() && problem.is_none(),
                    egui::Button::new("Salvar"),
                )
                .on_hover_text(hover)
                .clicked()
            {
                // O botão só fica habilitado com valores válidos; qualquer erro já está na tela.
                let _ = app.save_memory_setup();
            }
        });
    });
}

/// Quadro de setup da memória selecionada: Vset, Iset e as quatro proteções, em duas linhas de três.
fn setup_card(ui: &mut egui::Ui, app: &mut AppState, colors: &[Color32; 3]) {
    let memory = app.selected_profile();
    let dirty = app.memory_setup().is_dirty();
    let connected = app.is_connected();
    let errors = if dirty {
        app.memory_setup()
            .validate(memory)
            .err()
            .unwrap_or_default()
    } else {
        Vec::new()
    };
    let mut edit = None;

    card(ui, |ui| {
        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new(format!("Setup M{memory}"))
                    .size(SETUP_FONT_SIZE)
                    .strong(),
            );
            if dirty {
                ui.colored_label(WARN_COLOR, "\u{2022} não salvo");
            }
        });

        egui::Grid::new("setup_grid")
            .num_columns(3)
            .spacing([14.0, 4.0])
            .show(ui, |ui| {
                for (i, field) in SetupField::ALL.into_iter().enumerate() {
                    let accent = match field {
                        SetupField::Vset | SetupField::Ovp => Some(colors[0]),
                        SetupField::Iset | SetupField::Ocp => Some(colors[1]),
                        SetupField::Opp => Some(colors[2]),
                        SetupField::Otp => None,
                    };
                    let invalid = errors.iter().any(|e| e.field() == Some(field));
                    let value = app.memory_setup().get(field);
                    if let Some(v) = setup_field(ui, field, value, accent, connected, invalid) {
                        edit = Some((field, v));
                    }
                    if i % 3 == 2 {
                        ui.end_row();
                    }
                }
            });

        for error in errors.iter().take(3) {
            ui.colored_label(ERROR_COLOR, egui::RichText::new(error.message()).small());
        }

        if let Some(next) = app.pending_switch() {
            ui.horizontal(|ui| {
                ui.colored_label(WARN_COLOR, format!("Descartar as alterações de M{memory}?"));
                if ui.button("Descartar").clicked() {
                    app.confirm_switch();
                }
                if ui.button("Continuar editando").clicked() {
                    app.cancel_switch();
                }
            })
            .response
            .on_hover_text(format!("Você pediu M{next}"));
        }

        ui.horizontal(|ui| {
            if ui
                .add_enabled(
                    connected && app.group_values(memory).is_some(),
                    egui::Button::new("Usar no setpoint"),
                )
                .on_hover_text(
                    "Carrega Vset/Iset da memória no card Setpoint; confirme com Aplicar",
                )
                .clicked()
            {
                app.use_memory_in_setpoint(memory);
            }
            if dirty && ui.button("Desfazer").clicked() {
                app.discard_memory_setup_edits();
            }
            match app.save_status() {
                SaveStatus::Idle => {}
                SaveStatus::Verifying { .. } => {
                    ui.weak("verificando\u{2026}");
                }
                SaveStatus::Confirmed => {
                    ui.colored_label(OK_COLOR, "gravado");
                }
                SaveStatus::NotConfirmed => {
                    ui.colored_label(ERROR_COLOR, "não confirmado")
                        .on_hover_text("A fonte não devolveu os valores enviados");
                }
            }
        });

        ui.label(
            egui::RichText::new("* vale para o aparelho todo")
                .small()
                .weak(),
        );
    });

    if let Some((field, value)) = edit {
        app.set_setup_field(field, value);
    }
}

fn output_button(ui: &mut egui::Ui, app: &mut AppState) -> egui::Response {
    // A função é chamada num layout de baixo para cima, em que o primeiro item fica na base da área:
    // um espaço antes do botão o sobe. O egui não acrescenta o espaçamento entre itens depois de um
    // espaço, então a folga é exatamente a pedida (conferido pelo teste da folga de baixo).
    ui.add_space(OUTPUT_BOTTOM_MARGIN);
    let (label, fill, text_color, hover) = if app.output_on() {
        (
            "SAÍDA LIGADA",
            OK_COLOR,
            Color32::BLACK,
            "Clique para desligar a saída",
        )
    } else {
        (
            "SAÍDA DESLIGADA",
            Color32::from_gray(55),
            Color32::WHITE,
            "Clique para ligar a saída",
        )
    };
    let button = egui::Button::new(
        egui::RichText::new(label)
            .color(text_color)
            .strong()
            .size(20.0),
    )
    .fill(fill)
    // O raio vai no próprio botão (e não no estilo global): só este botão muda, e vale em todos os
    // estados, porque o egui o repassa ao quadro que desenha o botão.
    .corner_radius(OUTPUT_BUTTON_RADIUS);
    let w = ui.available_width();
    let resp = ui
        .add_enabled_ui(app.is_connected(), |ui| ui.add_sized([w, 56.0], button))
        .inner;
    let resp = resp.on_hover_text(hover);
    if resp.clicked() {
        app.toggle_output();
    }
    resp
}

pub(crate) fn bottom_bar(ui: &mut egui::Ui, app: &AppState, now: Instant) {
    let device = app.device();
    ui.horizontal(|ui| {
        info_pair(
            ui,
            "Capacidade",
            &format!("{:.3} Ah", device.output_capacity.unwrap_or(0.0)),
        );
        ui.separator();
        info_pair(
            ui,
            "Energia",
            &format!("{:.2} Wh", device.output_energy.unwrap_or(0.0)),
        );
        ui.separator();
        info_pair(ui, "Tempo", &format_hhmmss(app.current_elapsed(now)));
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gui::theme::{CONTROL_MIN_WIDTH, FIELD_GAP};
    use eframe::egui::TextStyle;

    /// Larguras dos textos no corpo padrão da interface, com a Noto Sans instalada, mais o
    /// espaçamento entre widgets e o preenchimento dos botões e campos do estilo padrão.
    struct Metrics {
        widths: Vec<f32>,
        item_spacing: f32,
        button_padding: f32,
    }

    fn measure(texts: &[&str]) -> Metrics {
        let ctx = egui::Context::default();
        crate::gui::install_fonts(&ctx);
        let mut widths = Vec::new();
        let (mut item_spacing, mut button_padding) = (0.0, 0.0);
        // Duas passagens: a fonte nova só vale a partir do quadro seguinte ao `set_fonts`.
        for _ in 0..2 {
            widths.clear();
            let _ = ctx.run_ui(egui::RawInput::default(), |ui| {
                let font = TextStyle::Body.resolve(ui.style());
                item_spacing = ui.spacing().item_spacing.x;
                button_padding = ui.spacing().button_padding.x;
                for text in texts {
                    let galley = ui.painter().layout_no_wrap(
                        (*text).to_owned(),
                        font.clone(),
                        Color32::WHITE,
                    );
                    widths.push(galley.size().x);
                }
            });
        }
        Metrics {
            widths,
            item_spacing,
            button_padding,
        }
    }

    /// Margens internas do card (de cada lado).
    const CARD_MARGIN: f32 = 8.0;

    #[test]
    fn linha_v_a_aplicar_cabe_na_coluna_minima() {
        // Valores máximos: 36,00 V e 8,200 A.
        let m = measure(&["V", "36.00 V", "A", "8.200 A", "Aplicar", "Desfazer"]);
        let w = &m.widths;
        let (sp, bp) = (m.item_spacing, m.button_padding);
        let gap = sp + FIELD_GAP + sp;
        let total = w[0] + sp + (w[1] + 2.0 * bp) // V e o campo
            + gap
            + w[2] + sp + (w[3] + 2.0 * bp) // A e o campo
            + gap
            + (w[4].max(w[5]) + 2.0 * bp) // botão Aplicar, do mesmo tamanho do Desfazer
            + 2.0 * CARD_MARGIN;
        assert!(total <= CONTROL_MIN_WIDTH, "{total} > {CONTROL_MIN_WIDTH}");
    }

    /// Altura da linha do título do Setpoint, com ou sem pendência.
    fn title_height(dirty: bool) -> f32 {
        let ctx = egui::Context::default();
        crate::gui::install_fonts(&ctx);
        let mut height = 0.0;
        for _ in 0..2 {
            let _ = ctx.run_ui(egui::RawInput::default(), |ui| {
                let top = ui.cursor().top();
                setpoint_title(ui, dirty, None);
                height = ui.cursor().top() - top;
            });
        }
        height
    }

    #[test]
    fn linha_do_titulo_tem_a_mesma_altura_com_e_sem_pendencia() {
        let (sem, com) = (title_height(false), title_height(true));
        assert!(sem > 0.0);
        assert!((sem - com).abs() < 0.01, "{sem} != {com}");
    }

    #[test]
    fn titulo_com_aviso_e_desfazer_cabe_na_coluna_minima() {
        let m = measure(&["Setpoint", "\u{2022} não aplicado", "Desfazer"]);
        let w = &m.widths;
        let (sp, bp) = (m.item_spacing, m.button_padding);
        let total = w[0] + sp + w[1] + sp + (w[2] + 2.0 * bp) + 2.0 * CARD_MARGIN;
        assert!(total <= CONTROL_MIN_WIDTH, "{total} > {CONTROL_MIN_WIDTH}");
    }

    /// Posição horizontal do Desfazer, pedindo para ele começar em `align_x`.
    fn undo_left(align_x: Option<f32>) -> f32 {
        let ctx = egui::Context::default();
        crate::gui::install_fonts(&ctx);
        let mut left = 0.0;
        for _ in 0..2 {
            let _ = ctx.run_ui(egui::RawInput::default(), |ui| {
                left = setpoint_title(ui, true, align_x).map_or(0.0, |button| button.rect.left());
            });
        }
        left
    }

    #[test]
    fn desfazer_comeca_na_posicao_do_aplicar() {
        // Uma posição bem à direita do aviso: o Desfazer deve começar exatamente nela.
        let alvo = 250.0;
        assert!((undo_left(Some(alvo)) - alvo).abs() < 0.01);
    }

    #[test]
    fn desfazer_vem_depois_do_aviso_se_o_aplicar_estiver_mais_a_esquerda() {
        let natural = undo_left(None);
        assert!((undo_left(Some(natural - 40.0)) - natural).abs() < 0.01);
    }

    /// Largura do botão Desfazer e a largura comum dos dois botões.
    fn undo_and_common_width() -> (f32, f32) {
        let ctx = egui::Context::default();
        crate::gui::install_fonts(&ctx);
        let (mut undo, mut common) = (0.0, 0.0);
        for _ in 0..2 {
            let _ = ctx.run_ui(egui::RawInput::default(), |ui| {
                common = action_button_width(ui);
                undo = setpoint_title(ui, true, None).map_or(0.0, |b| b.rect.width());
            });
        }
        (undo, common)
    }

    #[test]
    fn desfazer_tem_a_largura_comum_dos_dois_botoes() {
        let (undo, common) = undo_and_common_width();
        assert!(common > 0.0);
        assert!((undo - common).abs() < 0.01, "{undo} != {common}");
    }

    #[test]
    fn largura_comum_comporta_o_texto_mais_largo() {
        let m = measure(&["Aplicar", "Desfazer"]);
        let (_, common) = undo_and_common_width();
        let mais_largo = m.widths[0].max(m.widths[1]) + 2.0 * m.button_padding;
        assert!(
            (common - mais_largo).abs() < 0.01,
            "{common} != {mais_largo}"
        );
    }

    /// Retângulos desenhados (em qualquer nível de aninhamento) com a largura mínima dada.
    fn wide_rects(shape: &egui::epaint::Shape, min_width: f32, out: &mut Vec<egui::CornerRadius>) {
        match shape {
            egui::epaint::Shape::Rect(r) if r.rect.width() > min_width => out.push(r.corner_radius),
            egui::epaint::Shape::Vec(shapes) => {
                for s in shapes {
                    wide_rects(s, min_width, out);
                }
            }
            _ => {}
        }
    }

    /// Roda `output_button` desconectado (botão desabilitado) num layout de baixo para cima e devolve
    /// os raios dos retângulos largos desenhados, a resposta do botão e a base da área.
    fn output_button_run() -> (Vec<egui::CornerRadius>, Option<egui::Rect>, f32) {
        let ctx = egui::Context::default();
        crate::gui::install_fonts(&ctx);
        let mut app = AppState::new();
        let (mut radii, mut rect, mut bottom) = (Vec::new(), None, 0.0);
        for _ in 0..2 {
            let out = ctx.run_ui(egui::RawInput::default(), |ui| {
                bottom = ui.max_rect().bottom();
                ui.with_layout(egui::Layout::bottom_up(egui::Align::Min), |ui| {
                    rect = Some(output_button(ui, &mut app).rect);
                });
            });
            radii.clear();
            for clipped in &out.shapes {
                wide_rects(&clipped.shape, 100.0, &mut radii);
            }
        }
        (radii, rect, bottom)
    }

    #[test]
    fn botao_de_saida_tem_cantos_com_raio_de_5() {
        let (radii, _, _) = output_button_run();
        let esperado = egui::CornerRadius::same(OUTPUT_BUTTON_RADIUS);
        assert!(radii.contains(&esperado), "raios desenhados: {radii:?}");
    }

    #[test]
    fn botao_de_saida_tem_folga_de_10_px_embaixo_e_tamanho_de_sempre() {
        let (_, rect, bottom) = output_button_run();
        let rect = rect.expect("botão desenhado");
        let folga = bottom - rect.bottom();
        assert!(
            (folga - OUTPUT_BOTTOM_MARGIN).abs() < 0.5,
            "folga {folga} != {OUTPUT_BOTTOM_MARGIN}"
        );
        // A altura (56 px) e a largura (toda a área) não mudam.
        assert!(
            (rect.height() - 56.0).abs() < 0.5,
            "altura {}",
            rect.height()
        );
        assert!(rect.width() > 100.0);
    }
}
