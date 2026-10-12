# Contrato: API pública de `logics` (usada por `gui` e pelos binários)

**Feature**: [spec.md](../spec.md) | **Data model**: [data-model.md](../data-model.md)

Este é o contrato entre os módulos. `gui` e os binários só podem usar o que está listado aqui. As
regras de cada operação estão no data model, e este documento fixa as assinaturas e as garantias.
Atualizado após a implementação para refletir os nomes finais do código (T032).

**Invariante geral**: nenhum item abaixo usa tipos de `eframe`/`egui` (FR-004, SC-003).

---

## Árvore de módulos

```text
dps150 (lib: src/lib.rs)
├── logics
│   ├── protocol   — DPS150, DPSUpdate, commands  (antes src/dps150.rs)
│   ├── session    — AppCommand, SerialEvent, spawn_session, available_ports
│   ├── app        — AppState, ConnectionState, SetpointEditor
│   ├── history    — Sample, History, HISTORY_LEN
│   ├── scale      — AxisScale, CHART_Y_DIVS, CHART_Y_SUBDIVS
│   └── format     — format_hhmmss, format_window
└── gui            — GuiApp (impl eframe::App) e widgets; depende só de `logics`
```

## `logics::protocol`

Sem mudança de API em relação ao `src/dps150.rs` atual: `DPS150::{new, init_command, get_all,
get_protection, lock_session, enable_output, set_float_value, group_voltage_id, group_current_id,
push_serial_data}`, `DPSUpdate` e `commands::*`.

## `logics::session`

```rust
pub enum AppCommand { SetVoltage(f32), SetCurrent(f32), EnableOutput(bool),
                      SaveProfile(u8, f32, f32), SetPollInterval(Duration), Disconnect }
pub enum SerialEvent { Update(Box<DPSUpdate>), Connected, Error(String) }

/// Inicia a sessão numa thread própria e devolve os canais. `notify` é chamado
/// sempre que houver um evento novo para a interface (sem dependência de UI).
pub fn spawn_session(
    port_name: String,
    poll_interval: Duration,
    notify: impl Fn() + Send + 'static,
) -> (Sender<AppCommand>, Receiver<SerialEvent>);

/// Nomes das portas seriais disponíveis (vazio em caso de erro).
pub fn available_ports() -> Vec<String>;
```

**Garantias**: nenhuma função bloqueia quem a chama (a espera de 2 s e a E/S acontecem na thread da
sessão). `notify` é chamado depois de `Connected`, de `Error` e de cada lote de `Update`.

## `logics::app`

```rust
pub enum ConnectionState { Disconnected, Connecting, Connected, Error(String) }

pub struct SetpointEditor { pub vset_input: f32, pub cset_input: f32, /* dirty, editing_* */ }
impl SetpointEditor { pub fn is_dirty(&self) -> bool; }

pub const UPDATE_MS_RANGE: RangeInclusive<u64> = 100..=1000;
pub const PORT_REFRESH_INTERVAL: Duration = Duration::from_secs(1);

impl AppState {
    pub fn new() -> Self;

    // Leitura do estado (só leitura para a gui)
    pub fn device(&self) -> &DPSUpdate;
    pub fn connection(&self) -> &ConnectionState;
    pub fn is_connected(&self) -> bool;
    pub fn is_idle(&self) -> bool;                 // Disconnected | Error
    pub fn output_on(&self) -> bool;               // output_opened == Some(false)
    pub fn available_ports(&self) -> &[String];
    pub fn selected_port(&self) -> &str;
    pub fn selected_port_mut(&mut self) -> &mut String;
    pub fn setpoint(&self) -> &SetpointEditor;
    pub fn setpoint_mut(&mut self) -> &mut SetpointEditor; // a gui edita vset/cset e chama mark_edited
    pub fn selected_profile(&self) -> u8;
    pub fn group_values(&self, n: u8) -> Option<(f32, f32)>;
    pub fn history(&self) -> &History;
    pub fn update_ms(&self) -> u64;                // a gui edita uma cópia e chama set_update_ms
    pub fn update_interval(&self) -> Duration;
    pub fn current_elapsed(&self, now: Instant) -> Duration;

    // Ciclo por quadro
    pub fn refresh_ports_if_due(&mut self, now: Instant);
    pub fn poll_events(&mut self);                 // drena SerialEvent, faz merge e sincroniza
    pub fn tick(&mut self, now: Instant);          // relógio de saída e amostragem do histórico

    // Ações do usuário
    pub fn connect(&mut self, notify: impl Fn() + Send + 'static);
    pub fn disconnect(&mut self);
    pub fn mark_setpoint_edited(&mut self);        // campo alterado → dirty
    pub fn set_editing(&mut self, v: bool, c: bool);
    pub fn apply_setpoints(&mut self);
    pub fn discard_setpoint_edits(&mut self);      // "Desfazer"
    pub fn load_profile(&mut self, n: u8);         // não envia comandos
    pub fn save_profile(&mut self, n: u8);
    pub fn toggle_output(&mut self);
    pub fn set_update_ms(&mut self, ms: u64);      // envia SetPollInterval
    pub fn clear_history(&mut self);
}
```

**Garantias**:
- Só `apply_setpoints`, `save_profile`, `toggle_output`, `set_update_ms` e `disconnect` enviam
  comandos à fonte (Princípio III: nenhum envio implícito).
- Sem sessão ativa, os comandos são descartados em silêncio, como hoje no `send_cmd`. Durante
  `Connecting`, os comandos ficam no canal e são aplicados depois da inicialização. É assim que um
  `set_update_ms` feito durante a conexão não se perde. Fora de `Connected`, a `gui` desabilita os
  botões de Aplicar, Salvar e saída. O `disconnect`, durante `Connecting`, cancela a conexão.
- `tick` e `refresh_ports_if_due` usam só o `now` recebido, sem relógio interno.

## `logics::history`, `logics::scale`, `logics::format`

```rust
pub const HISTORY_LEN: usize = 300;
pub struct Sample { pub voltage: f32, pub current: f32, pub power: f32 }
impl History { pub fn new() -> Self; pub fn record_if_due(&mut self, now: Instant, interval: Duration, sample: Sample);
               pub fn clear(&mut self); pub fn iter(&self) -> impl Iterator<Item = &Sample>;
               pub fn len(&self) -> usize; pub fn is_empty(&self) -> bool; pub fn last(&self) -> Option<Sample>; }

pub const CHART_Y_DIVS: usize = 4;
pub const CHART_Y_SUBDIVS: usize = 4;
pub struct AxisScale { pub max: f32, pub decimals: usize }
impl AxisScale { pub fn for_peak(peak: f32, min_step: f32) -> Self; }

pub fn format_hhmmss(d: Duration) -> String;   // "01:02:03"
pub fn format_window(d: Duration) -> String;   // "30 s", "2.5 min", "5 min"
```

## Ponto de entrada — `src/controller.rs`

```rust
fn main() -> eframe::Result  // configura a janela (1000×540, título "DPS150 Controller")
                             // e chama eframe::run_native com dps150::gui::GuiApp::new()
```

Nenhuma regra de negócio e nenhum desenho de interface nesse arquivo (FR-003, SC-004).
