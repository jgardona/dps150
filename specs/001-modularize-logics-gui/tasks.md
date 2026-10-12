---

description: "Task list for feature 001-modularize-logics-gui"
---

# Tasks: Separação do controlador em módulos `logics` e `gui`

**Input**: Design documents from `specs/001-modularize-logics-gui/`

**Prerequisites**: [plan.md](./plan.md), [spec.md](./spec.md), [research.md](./research.md),
[data-model.md](./data-model.md), [contracts/logics-api.md](./contracts/logics-api.md),
[quickstart.md](./quickstart.md)

**Tests**: INCLUÍDOS. A spec pede testes automáticos das regras (FR-010, US2, SC-002), e a
constituição exige testes do protocolo (Princípio II).

**Organization**: as tarefas estão agrupadas por user story. Os caminhos são relativos à raiz do
repositório.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: pode rodar em paralelo (arquivos diferentes, sem dependência de tarefa incompleta)
- **[Story]**: US1 (paridade), US2 (regras testáveis), US3 (estrutura previsível)

## Convenções desta feature

- O código de partida é `src/controller.rs` (~1055 linhas) + `src/dps150.rs` + `src/tests.rs`. Os
  nomes citados nas tarefas (ex.: `merge_state`, `serial_thread_main`) são os do código atual.
- **Mover significa copiar o código sem mudar o comportamento.** Só mude o que a tarefa pedir
  (visibilidade `pub`, parâmetros `now`/`notify`, nomes do contrato).
- O projeto real precisa de `libudev` para compilar (`sudo dnf install systemd-devel`).
- Depois de cada tarefa que mexe em código: `cargo build --bin controller --bin tests` precisa
  compilar.

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: branch, referência de avisos e esqueleto da biblioteca.

- [X] T001 Criar o branch `001-modularize-logics-gui` a partir de `master` (`git switch -c 001-modularize-logics-gui`) com a árvore de trabalho limpa — *feito no branch já existente `refactor/break-in-modules`, criado pelo mantenedor a partir de `master`*
- [X] T002 Registrar a referência de avisos antes de qualquer mudança: rodar `cargo clippy --all-targets 2>&1 | grep -E '^warning' | sort | uniq -c` e salvar a saída em `specs/001-modularize-logics-gui/baseline-warnings.txt` (referência esperada: 18 avisos no bin `controller`, 31 no bin `tests`)
- [X] T003 Criar `src/lib.rs` com `pub mod logics;` e `pub mod gui;`, `src/logics/mod.rs` vazio e `src/gui/mod.rs` vazio. Não mudar o `Cargo.toml`: a lib é detectada por `src/lib.rs` com o nome de crate `dps150`

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: levar para `logics` as partes puras e a sessão serial, das quais todas as stories dependem.

**⚠️ CRITICAL**: nenhuma user story começa antes desta fase terminar.

- [X] T004 Mover o protocolo com `git mv src/dps150.rs src/logics/protocol.rs`. No arquivo, apagar a linha `use std::u8;` (obsoleta e sem uso). Em `src/logics/mod.rs`, declarar `pub mod protocol;`. Fora isso, o conteúdo fica idêntico
- [X] T005 Em `src/tests.rs`, trocar `mod dps150;`, `use dps150::DPS150;` e `use crate::dps150::DPSUpdate;` por `use dps150::logics::protocol::{DPS150, DPSUpdate};`. O resto do monitor fica igual (FR-009)
- [X] T006 Em `src/controller.rs`, trocar `mod dps150;` + `use dps150::{DPS150, DPSUpdate, commands};` por `use dps150::logics::protocol::{DPS150, DPSUpdate, commands};` e confirmar que os dois binários compilam
- [X] T007 [P] Criar `src/logics/scale.rs` movendo de `src/controller.rs` as constantes `CHART_Y_DIVS` (4) e `CHART_Y_SUBDIVS` (4), a struct `AxisScale { max, decimals }`, `impl AxisScale::for_peak` e `fn decimals_for`, todos `pub`. Declarar `pub mod scale;` em `src/logics/mod.rs` e trocar os usos em `src/controller.rs` por `dps150::logics::scale::*`
- [X] T008 [P] Criar `src/logics/format.rs` movendo `format_hhmmss` e `format_window` (ambos `pub`) de `src/controller.rs`. Declarar `pub mod format;` e ajustar os usos em `src/controller.rs`
- [X] T009 [P] Criar `src/logics/history.rs` com `pub const HISTORY_LEN: usize = 300;`, `pub struct Sample { pub voltage: f32, pub current: f32, pub power: f32 }` (`Clone, Copy`) e `pub struct History { samples: VecDeque<Sample>, last_sample: Option<Instant> }`, com estes métodos:
  - `new()`;
  - `record_if_due(&mut self, now: Instant, interval: Duration, sample: Sample)`: grava se `last_sample` é `None` ou se `now - last_sample >= interval`; "com a fila cheia, descarta a mais antiga antes de inserir";
  - `clear()`: esvazia a fila e zera `last_sample`;
  - `iter()`, `len()`, `last() -> Option<Sample>`.

  Declarar `pub mod history;`. Em `src/controller.rs`, trocar `history`/`last_sample` do `AppModel` por um `History` e reescrever `record_sample_if_due` para chamar `record_if_due(Instant::now(), self.update_interval(), …)` com a mesma condição de estar `Connected`
- [X] T010 Criar `src/logics/session.rs`:
  - mover de `src/controller.rs` os enums `AppCommand` e `SerialEvent` (agora `pub`);
  - transformar `serial_thread_main` numa função privada que recebe `notify: impl Fn() + Send + 'static` no lugar de `ctx: egui::Context`, trocando cada `ctx.request_repaint()` por `notify()`;
  - expor `pub fn spawn_session(port_name: String, poll_interval: Duration, notify: impl Fn() + Send + 'static) -> (Sender<AppCommand>, Receiver<SerialEvent>)`, que cria os dois canais `mpsc` e faz o `thread::spawn`;
  - expor `pub fn available_ports() -> Vec<String>`, com a lógica de `serialport::available_ports()` que hoje está em `refresh_available_ports`, devolvendo vazio em caso de erro.

  Declarar `pub mod session;`. Em `src/controller.rs`, o `connect` passa a chamar `spawn_session(port, interval, move || ctx.request_repaint())`. O arquivo NÃO DEVE importar `egui`

**Checkpoint**: `logics` tem `protocol`, `scale`, `format`, `history` e `session`; `controller.rs` ainda tem `AppModel` e a interface; o app funciona como antes.

---

## Phase 3: User Story 1 - Operar a fonte exatamente como antes (Priority: P1) 🎯 MVP

**Goal**: toda a lógica em `logics`, toda a interface em `gui`, `src/controller.rs` só com `main`, e
o comportamento visível idêntico (FR-001, FR-002, FR-003, FR-005, FR-006, FR-007).

**Independent Test**: rodar `cargo run --release --bin controller` com a fonte conectada e executar os
9 passos da "Verificação manual de paridade" da [spec](./spec.md#verificação-manual-de-paridade).
Todos precisam dar o mesmo resultado de antes.

### Implementation for User Story 1

- [X] T011 [US1] Criar `src/logics/app.rs` (e declarar `pub mod app;`) com os tipos e campos de [data-model.md](./data-model.md):
  - `pub enum ConnectionState { Disconnected, Connecting, Connected, Error(String) }`;
  - `pub struct SetpointEditor { pub vset_input: f32, pub cset_input: f32, dirty: bool, editing_v: bool, editing_c: bool }`, com `pub fn is_dirty(&self) -> bool`;
  - `struct OutputClock { run_start: Option<Instant>, frozen: Duration }`;
  - `pub struct AppState` com: `state: DPSUpdate`, `connection`, `commands: Option<Sender<AppCommand>>`, `events: Option<Receiver<SerialEvent>>`, `available_ports: Vec<String>`, `selected_port: String`, `last_port_refresh: Option<Instant>`, `setpoint`, `selected_profile: u8` ("começa em 1"), `clock`, `update_ms: u64` ("começa em 100", "dentro de `UPDATE_MS_RANGE` (100..=1000)"), `history: History`;
  - `pub const UPDATE_MS_RANGE: RangeInclusive<u64> = 100..=1000;` e `pub const PORT_REFRESH_INTERVAL: Duration = Duration::from_secs(1);`.
- [X] T012 [US1] Em `src/logics/app.rs`, implementar os métodos de `AppState` de [contracts/logics-api.md](./contracts/logics-api.md), movendo a lógica de `AppModel` em `src/controller.rs`:
  - **Leitura**: `new`, `device`, `connection`, `is_connected`, `is_idle` (Disconnected | Error), `output_on` (`output_opened == Some(false)`), `available_ports`, `selected_port_mut`, `setpoint`, `setpoint_mut`, `selected_profile`, `group_values` (de `group_values`), `history`, `update_ms_mut`, `update_interval`, `current_elapsed(now)`.
  - **Ciclo por quadro**:
    - `refresh_ports_if_due(now)`: só quando `is_idle()` e `last_port_refresh` é `None` ou tem ≥ `PORT_REFRESH_INTERVAL`; chama `session::available_ports()` e mantém a regra "se sumir da lista, passa a ser a primeira porta disponível, ou vazio".
    - `poll_events()`: drena o receptor; em `Update`, faz o merge e sincroniza; em `Connected`, muda o estado; em `Error(msg)`, vai para `Error`, larga os canais e chama `reset_telemetry`.
    - `tick(now)`: as transições do `OutputClock` e, se `Connected`, `history.record_if_due(now, update_interval, sample)` com os campos ausentes valendo 0.
  - **Ações**:
    - `connect(notify)`: com a porta vazia, vai para `Error("Nenhum dispositivo/porta serial disponível para conexão.")`; senão chama `spawn_session` e vai para `Connecting` + `reset_telemetry`.
    - `disconnect()`: envia `Disconnect`, larga os canais, vai para `Disconnected` + `reset_telemetry`, e faz `last_port_refresh = None` para a lista atualizar na hora.
    - `mark_setpoint_edited`, `set_editing(v, c)`, `apply_setpoints` (envia `SetVoltage` e `SetCurrent` e zera `dirty`), `discard_setpoint_edits` (zera `dirty` e sincroniza), `load_profile(n)` ("Nenhum comando é enviado"; com os dois valores, preenche e marca `dirty`; sempre faz `selected_profile = n`), `save_profile(n)`, `toggle_output` (envia `EnableOutput(!output_on)`), `set_update_ms(ms)` (limita a `UPDATE_MS_RANGE` e envia `SetPollInterval`), `clear_history`.
  - **Privados**: `send(cmd)` (descarta em silêncio sem canal, como o `send_cmd` atual), `merge` (a macro `merge_opt!` de `merge_state`, com todos os 37 campos), `sync_edit_buffers` (regra de `sync_edit_buffers_if_idle`) e `reset_telemetry` (zera `state`, o relógio e o histórico).

  Não usar nenhum tipo de `egui`/`eframe`.
- [X] T013 [P] [US1] Criar `src/gui/theme.rs` com as constantes visuais movidas de `src/controller.rs`:
  - `DEFAULT_SERIES_COLORS`;
  - `OK_COLOR`, `WARN_COLOR`, `ERROR_COLOR`, `ACTION_COLOR`;
  - `CONTROL_SHARE` (0.36), `CONTROL_MIN_WIDTH` (330), `CONTROL_MAX_WIDTH` (440);
  - `COLOR_PICKER_WIDTH`, **mantendo o valor atual de 200.0**;
  - `CHART_AXIS_COL_W` (40).
- [X] T014 [P] [US1] Criar `src/gui/widgets.rs` movendo `ReadoutRow` (+ `impl egui::Widget`), `card`, `info_pair`, `status_dot` e `column_ui` de `src/controller.rs` (`pub(crate)`)
- [X] T015 [US1] Criar `src/gui/chart.rs`:
  - mover `draw_history_chart`, que passa a receber `&History` e usar `logics::scale::{AxisScale, CHART_Y_DIVS, CHART_Y_SUBDIVS}` e `logics::history::HISTORY_LEN`;
  - trocar o tipo do array `series` por um alias, ex.: `type SeriesSpec = (fn(&Sample) -> f32, Color32, &'static str, f32);` (resolve o aviso `clippy::type_complexity`);
  - criar `pub(crate) fn chart_panel(ui, app: &mut AppState, colors: &mut [Color32; 3])` com o conteúdo do `chart_panel` atual: legenda clicável com `Popup` + `color_picker_color32` + "Padrão", rótulo "janela" via `format_window`, slider ligado a `app.update_ms_mut()` com `UPDATE_MS_RANGE` (ao mudar, chama `app.set_update_ms`), e "Limpar" chamando `app.clear_history()`.
- [X] T016 [US1] Criar `src/gui/panels.rs` movendo de `src/controller.rs`, como funções `pub(crate)` que recebem `&mut AppState` (e as cores quando preciso):
  - `top_bar`: usa `is_idle`, `available_ports`, `selected_port_mut`, `connect(move || ctx.request_repaint())` com `ctx = ui.ctx().clone()`, `disconnect`, mensagem de erro e as informações do aparelho. **Não** chama mais `refresh_available_ports`.
  - `control_column` (`readouts`, `setpoint_card`, `output_button`): o setpoint edita `app.setpoint_mut().vset_input`/`cset_input`, chama `mark_setpoint_edited` quando o campo muda, `set_editing(...)` com o foco/arrasto, e `apply_setpoints` / `discard_setpoint_edits` / `load_profile` / `save_profile`. O botão de saída usa `output_on` e `toggle_output`.
  - `bottom_bar`: usa `current_elapsed(Instant::now())`.

  Manter textos, tamanhos e faixas idênticos (V `0.0..=30.0` com 2 casas, A `0.0..=5.0` com 3 casas)
- [X] T017 [US1] Em `src/gui/mod.rs`:
  - declarar `mod theme; mod widgets; mod chart; mod panels;`;
  - criar `pub struct GuiApp { app: AppState, series_colors: [Color32; 3] }` com `pub fn new() -> Self` (cores = `DEFAULT_SERIES_COLORS`);
  - implementar `eframe::App::ui`: `let now = Instant::now();`, `app.refresh_ports_if_due(now)`, `app.poll_events()`, `app.tick(now)`, depois o mesmo layout do `ui` atual (painéis superior e inferior, coluna de controle proporcional limitada, gráfico com o resto), e por fim `request_repaint_after(app.update_interval().min(100 ms))`.
- [X] T018 [US1] Reduzir `src/controller.rs` a: `use dps150::gui::GuiApp;`, as constantes `WIDTH` (1000.0) e `HEIGHT` (540.0) e `fn main() -> eframe::Result`, que monta `NativeOptions` e chama `eframe::run_native("DPS150 Controller", options, Box::new(|_cc| Ok(Box::new(GuiApp::new()))))`. Apagar todo o resto (SC-004: ~40 linhas ou menos)
- [X] T019 [US1] Rodar `cargo fmt`, `cargo build --bin controller --bin tests` e `cargo clippy --all-targets`, e comparar com `specs/001-modularize-logics-gui/baseline-warnings.txt`. Corrigir todo aviso novo (FR-011)
- [ ] T020 [US1] Com a fonte conectada, executar os 9 passos da "Verificação manual de paridade" de `specs/001-modularize-logics-gui/spec.md` e anotar o resultado de cada passo no fim de `specs/001-modularize-logics-gui/quickstart.md`. A única diferença aceita é a porta recém-plugada levar até 1 s para aparecer (research R7)

**Checkpoint**: o app funciona como antes com a nova estrutura. Este é o MVP.

---

## Phase 4: User Story 2 - Regras do programa verificáveis sem interface (Priority: P2)

**Goal**: testes automáticos das regras de `logics`, executáveis sem janela e sem porta serial
(FR-010, SC-002, Princípio II).

**Independent Test**: `env -u DISPLAY -u WAYLAND_DISPLAY cargo test --lib` passa sem a fonte conectada.

### Tests for User Story 2

- [X] T021 [P] [US2] Em `src/logics/scale.rs`, adicionar `#[cfg(test)] mod tests` com:
  - `AxisScale::for_peak(3.3, 0.1)` → `max == 4.0`, `decimals == 2`;
  - `for_peak(0.0, 0.01)` → `max == 0.04`, `decimals == 4`;
  - `for_peak(0.0, 0.1)` → `max == 0.4`, `decimals == 3`;
  - `for_peak(15.0, 0.1)` → `max == 20.0`;
  - `decimals_for(0.25) == 2` e `decimals_for(1.0) == 0`.

  Comparar floats com tolerância de `1e-6`
- [X] T022 [P] [US2] Em `src/logics/format.rs`, adicionar testes:
  - `format_hhmmss(Duration::from_secs(3723)) == "01:02:03"` e `format_hhmmss(Duration::ZERO) == "00:00:00"`;
  - `format_window` de 30 s → `"30 s"`, de 150 s → `"2.5 min"`, de 300 s → `"5 min"`.
- [X] T023 [P] [US2] Em `src/logics/history.rs`, adicionar testes com um `Instant` base `t0` e deslocamentos explícitos:
  - (a) a primeira amostra é sempre gravada;
  - (b) uma amostra antes de `interval` é ignorada;
  - (c) ao inserir a amostra 301, `len() == 300` e a mais antiga sai (a primeira em `iter()` é a segunda inserida);
  - (d) `clear()` zera a fila, e a próxima amostra é gravada mesmo dentro do intervalo.
- [X] T024 [US2] Em `src/logics/app.rs`, adicionar o construtor só de teste `#[cfg(test)] fn with_test_session() -> (AppState, Receiver<AppCommand>, Sender<SerialEvent>)`. Ele cria os dois canais, guarda o lado da aplicação no `AppState` e deixa `connection = Connected`
- [X] T025 [US2] Em `src/logics/app.rs`, usando `with_test_session`, testar:
  - (a) edição pendente preservada: `vset_input = 5.0` + `mark_setpoint_edited`, depois um `Update` com `vset = Some(3.3)` + `poll_events` → `vset_input` continua 5.0;
  - (b) `discard_setpoint_edits` → `vset_input == 3.3` e `!is_dirty()`;
  - (c) `load_profile(2)` com `g2_vset/g2_cset` conhecidos → campos preenchidos, `is_dirty()`, `selected_profile() == 2` e **nenhum comando no canal** (`try_recv` é `Err`);
  - (d) `apply_setpoints` → recebe `SetVoltage` e `SetCurrent` com os valores dos campos, e `!is_dirty()`;
  - (e) `save_profile(3)` → recebe `SaveProfile(3, v, c)`;
  - (f) merge parcial: um `Update` só com `output_voltage` mantém o `temperature` anterior;
  - (g) relógio: `tick(t0)` com saída ligada, `current_elapsed(t0 + 5s) == 5s`; desligar e `tick(t0 + 5s)` → congela em 5s; `disconnect` zera;
  - (h) `tick` com `connection = Disconnected` não grava no histórico;
  - (i) `Error("x")` via canal + `poll_events` → `ConnectionState::Error`, `!is_connected()` e histórico vazio.
- [X] T026 [P] [US2] Em `src/logics/protocol.rs`, adicionar `#[cfg(test)] mod tests`, com um helper de teste que monta um pacote de resposta `[0xF0, 0xA1, type_id, len, payload…, checksum]` (checksum = `(type_id + len + Σpayload) % 256`). Testar:
  - (a) `enable_output(true) == [0xF1, 0xB1, 219, 1, 1, 221]`;
  - (b) `set_float_value(commands::VOLTAGE_SET, 5.0)` tem cabeçalho `[0xF1, 0xB1, 193, 4]`, payload `5.0f32.to_le_bytes()` e checksum correto;
  - (c) um pacote type_id 195 com 3 floats gera um `DPSUpdate` com tensão, corrente e potência;
  - (d) lixo antes do cabeçalho é ignorado e o pacote é lido;
  - (e) o pacote dividido em dois `push_serial_data` só gera a atualização na segunda chamada;
  - (f) checksum errado não gera atualização.
- [X] T027 [US2] Rodar `env -u DISPLAY -u WAYLAND_DISPLAY cargo test --lib` e confirmar que todos os testes passam sem interface gráfica e sem porta serial (SC-002)

**Checkpoint**: US1 e US2 completas; regras cobertas por testes.

---

## Phase 5: User Story 3 - Estrutura de código previsível (Priority: P3)

**Goal**: dependências e responsabilidades verificáveis e documentadas (FR-004, FR-012, SC-003, SC-004).

**Independent Test**: as verificações da seção 2 de [quickstart.md](./quickstart.md) passam.

### Implementation for User Story 3

- [X] T028 [P] [US3] Adicionar uma doc de módulo `//!` em `src/logics/mod.rs` (responsabilidade: regras, protocolo e sessão; regra "não depende de `gui` nem de egui") e em `src/gui/mod.rs` (responsabilidade: apresentação; regra "usa `logics`, não contém regras de negócio nem acessa a serial")
- [X] T029 [P] [US3] Atualizar a seção "Estrutura do código" de `README.md`: a tabela de arquivos passa a listar `src/controller.rs` (entrada), `src/lib.rs`, `src/logics/{protocol,session,app,history,scale,format}.rs`, `src/gui/{mod,theme,widgets,panels,chart}.rs` e `src/tests.rs`. Descrever a regra de dependência `controller → gui → logics`, trocar o link de `src/dps150.rs` por `src/logics/protocol.rs` na seção "Protocolo", e citar `cargo test --lib` (FR-012)
- [X] T030 [US3] Rodar as verificações estruturais da seção 2 de `specs/001-modularize-logics-gui/quickstart.md`:
  - `grep -rnE 'egui|eframe' src/logics/` sem resultados;
  - `grep -rnE 'serialport|logics::protocol::DPS150' src/gui/` sem resultados;
  - `wc -l src/controller.rs` ≤ ~40;
  - `src/tests.rs` sem `mod dps150`.

  Corrigir qualquer falha

**Checkpoint**: as três stories estão completas.

---

## Phase 6: Polish & Cross-Cutting Concerns

- [X] T031 Emendar a constituição com `/speckit-constitution` (PATCH 1.0.0 → 1.0.1): trocar o caminho `src/dps150.rs` por `src/logics/protocol.rs` no Princípio I e atualizar o "Last Amended" em `.specify/memory/constitution.md`
- [X] T032 [P] Se algum nome divergiu durante a implementação, atualizar `specs/001-modularize-logics-gui/contracts/logics-api.md` e `specs/001-modularize-logics-gui/data-model.md` para refletir o código final
- [ ] T033 Executar todo o `specs/001-modularize-logics-gui/quickstart.md` (gates automáticos, estrutura, paridade, monitor `tests`) e confirmar que `cargo build --release --bin controller --bin tests` passa
- [ ] T034 Apagar `specs/001-modularize-logics-gui/baseline-warnings.txt` (ou mantê-lo como registro, a critério do mantenedor) e fazer o commit da feature no branch `001-modularize-logics-gui`

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: sem dependências. T002 precisa vir antes de qualquer mudança de código.
- **Foundational (Phase 2)**: depende do Setup. T004 → T005/T006 → (T007, T008, T009 em paralelo) → T010.
- **US1 (Phase 3)**: depende da Foundational. T011 → T012 → (T013, T014 em paralelo) → T015 → T016 → T017 → T018 → T019 → T020.
- **US2 (Phase 4)**: T021, T022, T023 e T026 dependem só da Foundational e podem começar junto com a US1. T024/T025 dependem de T012. T027 fecha a fase.
- **US3 (Phase 5)**: depende da US1 (T018), porque as verificações olham a estrutura final.
- **Polish (Phase 6)**: depende de US1, US2 e US3.

### User Story Dependencies

- **US1 (P1)**: só depende da Foundational. É o MVP.
- **US2 (P2)**: os testes de funções puras e do protocolo não dependem da US1; os testes de `app` dependem de T011–T012.
- **US3 (P3)**: verifica e documenta o resultado da US1.

### Parallel Opportunities

- Phase 2: T007, T008 e T009 (arquivos novos diferentes; cada um mexe em trechos distintos de `src/controller.rs`, mas convém integrar um de cada vez).
- Phase 3: T013 e T014.
- Phase 4: T021, T022, T023 e T026 (arquivos diferentes), inclusive em paralelo com a Phase 3.
- Phase 5: T028 e T029.

---

## Parallel Example: User Story 2

```bash
# Testes de funções puras e do protocolo, logo após a Phase 2:
Task: "T021 testes de AxisScale em src/logics/scale.rs"
Task: "T022 testes de formatação em src/logics/format.rs"
Task: "T023 testes do histórico em src/logics/history.rs"
Task: "T026 testes do protocolo em src/logics/protocol.rs"
```

## Parallel Example: User Story 1

```bash
# Depois de T012 (AppState pronto):
Task: "T013 constantes visuais em src/gui/theme.rs"
Task: "T014 widgets em src/gui/widgets.rs"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Phase 1 (Setup) e Phase 2 (Foundational): o app continua funcionando a cada passo.
2. Phase 3 (US1) até o T020: **pare e valide** com a fonte conectada.
3. Neste ponto a estrutura pedida (`logics`, `gui`, `controller.rs` com `main`) já está entregue.

### Incremental Delivery

1. Setup + Foundational: `logics` com as partes puras e a sessão.
2. US1: estrutura completa e paridade validada (MVP).
3. US2: testes automáticos; parte deles já pode ter sido feita durante a US1.
4. US3: verificações e documentação.
5. Polish: emenda da constituição, quickstart completo e commit.

---

## Notes

- Commit sugerido depois de cada fase (o app compila e funciona em todos os checkpoints).
- Em caso de dúvida sobre uma regra, [data-model.md](./data-model.md) é a fonte. Para nomes e
  assinaturas, [contracts/logics-api.md](./contracts/logics-api.md).
- Nada de feature nova: os débitos de UX (faixa de 30 V / 5 A, texto "Update") ficam fora do escopo.
