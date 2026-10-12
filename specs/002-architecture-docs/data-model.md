# Data Model: Documentação de arquitetura do software

**Feature**: [spec.md](./spec.md) | **Plan**: [plan.md](./plan.md) | **Research**: [research.md](./research.md)

Esta feature não tem dados de execução. As "entidades" são os documentos e as regras que os ligam ao
código. O formato exato das entradas está em [contracts/doc-format.md](./contracts/doc-format.md).

---

## Entidades

### Documento de entrada — `docs/README.md`

| Campo | Regra |
|---|---|
| Lista de documentos | Um item por documento de `docs/`, com link relativo e uma linha de descrição (FR-003) |
| Ordem de leitura | Sugestão por perfil: entender → alterar um módulo → manter a documentação |

### Visão geral — `docs/arquitetura.md`

| Seção | Regra |
|---|---|
| Módulos | Lista de **todos** os módulos de `src/`, com papel e "o que não faz" (FR-004, FR-006) |
| Diagramas | Os 6 diagramas de [contracts/doc-format.md](./contracts/doc-format.md#diagramas-obrigatórios) (FR-005, FR-011, FR-012) |
| Tipos centrais | `AppState`, `ConnectionState`, `SetpointEditor`, `OutputClock`, `History`/`Sample`, `AxisScale`, `AppCommand`, `SerialEvent`, `DPSUpdate` (FR-011) |
| Regras de projeto | O que `logics` e `gui` podem e não podem usar (FR-004 de 001) |

### Documento de módulo — `docs/modulos/<slug>.md`

| Campo | Regra |
|---|---|
| `slug` | Caminho do arquivo sem `src/` e sem `.rs`, com `/` trocado por `-` (`src/logics/app.rs` → `logics-app`; `src/gui/mod.rs` → `gui-mod`). Existe um documento para **todo arquivo com pelo menos uma função fora de testes** |
| Papel | O que o módulo faz e o que **não** faz (FR-006) |
| Tipos | Os tipos do módulo e o papel de cada um (parte do FR-011) |
| Funções | Uma entrada por função, na ordem do arquivo (FR-007) |
| Testes | Uma frase sobre o que os testes do módulo cobrem; os testes não têm entrada própria |

### Entrada de função

| Campo | Regra |
|---|---|
| Título | `### \`Nome qualificado\`` — única no documento; é a chave da conferência |
| Marcação | `*(interna)*` quando a função é privada |
| Propósito | Uma frase |
| Entradas | Cada parâmetro e seu significado (ou "nenhuma") |
| Saída | O valor devolvido (ou "nada") |
| Efeitos | Comandos enviados, estado alterado, E/S, bloqueio (ou "nenhum") |
| Como funciona | Passos ou parágrafo; **com exemplo** para as funções da lista da pesquisa R3 (FR-009) |

**Nome qualificado**: `Tipo::método` (método inerente ou de trait; vale o tipo que implementa) ou `nome`
(função livre). Genéricos, lifetimes e caminhos do tipo são descartados.

### Diagrama

| Campo | Regra |
|---|---|
| Tipo | `flowchart`, `sequenceDiagram` ou `stateDiagram-v2` (Mermaid), ou ASCII de uma linha |
| Título | Uma linha de texto logo acima do bloco |
| Tamanho | No máximo ~12 nós |
| Legenda | Os rótulos usam os nomes reais do código |

### Conferência de cobertura — `tests/docs_coverage.rs`

| Verificação | Falha quando |
|---|---|
| Cobertura de funções | Uma função do código não tem título no documento do módulo (**faltando**), ou um título não corresponde a nenhuma função (**sobrando**) |
| Documento de módulo | Um arquivo com funções não tem `docs/modulos/<slug>.md` |
| Links | Um link relativo de `README.md` ou `docs/**/*.md` aponta para um arquivo inexistente |
| Sanidade do scanner | A profundidade de chaves de algum `.rs` não volta a 0 no fim do arquivo |

## Mapa módulo → documento → funções esperadas

É a lista que a conferência vai cobrar (88 funções). Os nomes seguem a regra de nome qualificado.

| Arquivo | Documento | Funções |
|---|---|---|
| `src/controller.rs` | `controller` | `main` (1) |
| `src/gui/mod.rs` | `gui-mod` | `GuiApp::default`, `GuiApp::new`, `GuiApp::ui` (3) |
| `src/gui/panels.rs` | `gui-panels` | `top_bar`, `control_column`, `readouts`*, `setpoint_card`*, `output_button`*, `bottom_bar` (6) |
| `src/gui/chart.rs` | `gui-chart` | `chart_panel`, `draw_history_chart`* (2) |
| `src/gui/widgets.rs` | `gui-widgets` | `ReadoutRow::ui`, `card`, `info_pair`, `status_dot`, `column_ui` (5) |
| `src/logics/protocol.rs` | `logics-protocol` | `DPS150::default`, `new`, `init_command`, `get_protection`, `get_all`, `lock_session`, `group_voltage_id`, `group_current_id`, `enable_output`, `set_float_value`, `build_command`*, `push_serial_data`, `process_buffer`*, `parse_data`*, `read_float`* (15) |
| `src/logics/session.rs` | `logics-session` | `spawn_session`, `available_ports`, `session_main`* (3) |
| `src/logics/app.rs` | `logics-app` | `SetpointEditor::is_dirty`; `OutputClock::tick`*, `OutputClock::elapsed`*; `AppState::` `default`, `new`, `device`, `connection`, `is_connected`, `is_idle`, `output_on`, `available_ports`, `selected_port`, `selected_port_mut`, `setpoint`, `setpoint_mut`, `selected_profile`, `group_values`, `history`, `update_ms`, `update_interval`, `current_elapsed`, `refresh_ports_if_due`, `poll_events`, `tick`, `connect`, `disconnect`, `mark_setpoint_edited`, `set_editing`, `apply_setpoints`, `discard_setpoint_edits`, `load_profile`, `save_profile`, `toggle_output`, `set_update_ms`, `clear_history`, `send`*, `end_session`*, `reset_telemetry`*, `set_available_ports`*, `merge`*, `sync_edit_buffers`* (41) |
| `src/logics/history.rs` | `logics-history` | `History::default`, `new`, `record_if_due`, `clear`, `iter`, `len`, `is_empty`, `last` (8) |
| `src/logics/scale.rs` | `logics-scale` | `decimals_for`, `AxisScale::for_peak` (2) |
| `src/logics/format.rs` | `logics-format` | `format_hhmmss`, `format_window` (2) |
| `src/lib.rs`, `src/logics/mod.rs`, `src/gui/theme.rs` | — (descritos em `arquitetura.md`) | nenhuma |

`*` = função privada (recebe a marcação `*(interna)*`). Inclui funções de `impl` de traits
(`Default::default`, `eframe::App::ui`, `egui::Widget::ui`). A função `with_test_session` de
`AppState` (marcada `#[cfg(test)]`) e todo o bloco `mod tests` ficam de fora.

## Ciclo de vida de uma entrada

```text
função nova no código ──▶ conferência falha ("faltando")
        │                          │
        │                 autor escreve a entrada
        ▼                          ▼
função renomeada ──▶ falha ("faltando" + "sobrando") ──▶ autor renomeia o título
função removida  ──▶ falha ("sobrando")              ──▶ autor apaga a entrada
```
