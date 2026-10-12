# Implementation Plan: Separação do controlador em módulos `logics` e `gui`

**Branch**: `001-modularize-logics-gui` | **Date**: 2026-10-12 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `specs/001-modularize-logics-gui/spec.md`

## Summary

Hoje `src/controller.rs` (~1050 linhas) mistura estado, regras, sessão serial e desenho da interface.
O protocolo (`src/dps150.rs`) é incluído separadamente pelos dois binários. A refatoração cria uma
biblioteca no próprio pacote (`src/lib.rs`) com dois módulos:

- **`logics`**: protocolo, sessão serial, estado e regras, histórico, escala do gráfico e formatação.
  Sem dependência de egui. Recebe o relógio (`now`) e o notificador de redesenho por parâmetro.
- **`gui`**: `GuiApp` (`impl eframe::App`), layout, widgets, gráfico, cores e dimensões. Lê o estado
  de `logics` e chama as ações dele.

`src/controller.rs` fica só com `main`. O comportamento visível não muda. Os dois ganhos são testes
automáticos das regras (FR-010) e o fim dos avisos de código morto causados pela inclusão duplicada
do protocolo (FR-011).

## Technical Context

**Language/Version**: Rust, edição 2024, rustc 1.88 ou mais novo (o código usa let chains)

**Primary Dependencies**: `eframe`/`egui` 0.34 (só em `gui` e `controller.rs`), `serialport` 4.9
(só em `logics::session`)

**Storage**: N/A (nada é persistido; as cores voltam ao padrão ao reiniciar, como hoje)

**Testing**: `cargo test --lib`, com testes unitários `#[cfg(test)]` nos submódulos de `logics`, mais
o roteiro manual de paridade da spec

**Target Platform**: Linux (desktop, Wayland/X11), aparelhos FNIRSI IPS3608/DPS150 a 115200 baud

**Project Type**: desktop-app (GUI nativa) + ferramenta CLI auxiliar (`tests`), num único pacote
Cargo

**Performance Goals**: sem regressão. Redesenho ocioso a cada 100 ms ou menos, comando enviado à
fonte em até `update_ms` (100–1000 ms), interface sem travar durante a conexão.

**Constraints**: nenhum tipo de egui em `logics`; nenhuma E/S serial na thread da GUI; memória
limitada (`HISTORY_LEN` = 300, `BUFFER_MAX_SIZE` = 1024); comportamento visível idêntico (FR-007).

**Scale/Scope**: ~1300 linhas de Rust em 3 arquivos, reorganizadas em cerca de 12 arquivos; 2
binários.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Princípio | Gate | Pré-pesquisa | Pós-design |
|---|---|---|---|
| I. Qualidade | `fmt`/`clippy` sem avisos novos | ✅ Referência registrada (controller 18, tests 31) | ✅ A lib remove os avisos de código morto (R1); `use std::u8` removido (R2); o aviso `type_complexity` do gráfico é resolvido com um alias de tipo em `gui::chart` |
| I. Qualidade | Protocolo puro, sem E/S nem egui | ✅ Já é puro | ✅ Movido intacto para `logics::protocol` |
| I. Qualidade | Sem `unwrap`/`panic!` em caminhos da serial | ✅ Código atual usa `unwrap_or`/`let _ =` | ✅ Mantido; o contrato não introduz `unwrap` |
| II. Testes | Protocolo com testes de fixtures | ⚠️ Não existem hoje (débito) | ✅ Testes de checksum, parsing, ressincronização e fragmentação incluídos (R8) |
| II. Testes | Lógica da GUI testável quando alterada | ⚠️ Hoje só testável com a janela | ✅ Regras em `logics`, com `now` injetado (R4) |
| II. Testes | Roteiro manual para hardware | ✅ Na spec | ✅ Referenciado no quickstart |
| III. UX | Sem envio implícito; edição pendente preservada | ✅ | ✅ Garantias explícitas no contrato |
| III. UX | README atualizado | — | ✅ FR-012, tarefa de fechamento |
| IV. Desempenho | Nenhuma E/S serial na thread da GUI | ⚠️ A enumeração de portas roda a cada quadro quando ocioso | ✅ Limitada a 1 Hz (R7) |
| IV. Desempenho | Memória limitada, redesenho ≤ 100 ms | ✅ | ✅ Inalterado |
| Restrições | Dependência nova justificada | ✅ Nenhuma dependência nova | ✅ |

**Resultado**: aprovado. Os ⚠️ pré-pesquisa são débitos existentes, e o desenho corrige todos (a
constituição manda corrigir um débito quando a área é tocada). Nenhuma violação precisa entrar em
Complexity Tracking.

## Project Structure

### Documentation (this feature)

```text
specs/001-modularize-logics-gui/
├── spec.md
├── plan.md              # Este arquivo
├── research.md          # Fase 0 — decisões R1–R8
├── data-model.md        # Fase 1 — entidades e regras
├── quickstart.md        # Fase 1 — roteiro de validação
├── contracts/
│   └── logics-api.md    # Fase 1 — API pública de logics
├── checklists/
│   └── requirements.md
└── tasks.md             # Fase 2 (/speckit-tasks — ainda não criado)
```

### Source Code (repository root)

```text
src/
├── controller.rs        # bin "controller": só main() — janela e run_native(GuiApp)
├── tests.rs             # bin "tests": monitor CLI, usa dps150::logics::protocol
├── lib.rs               # crate dps150: pub mod logics; pub mod gui;
├── logics/
│   ├── mod.rs
│   ├── protocol.rs      # ← src/dps150.rs (movido; sem `use std::u8`)
│   ├── session.rs       # AppCommand, SerialEvent, spawn_session, available_ports
│   ├── app.rs           # AppState, ConnectionState, SetpointEditor, OutputClock, merge
│   ├── history.rs       # Sample, History, HISTORY_LEN
│   ├── scale.rs         # AxisScale, decimals_for, CHART_Y_DIVS/SUBDIVS
│   └── format.rs        # format_hhmmss, format_window
└── gui/
    ├── mod.rs           # GuiApp + impl eframe::App (laço por quadro e layout de 2 colunas)
    ├── theme.rs         # DEFAULT_SERIES_COLORS, OK/WARN/ERROR/ACTION, dimensões
    ├── widgets.rs       # ReadoutRow, card, info_pair, status_dot, column_ui
    ├── panels.rs        # top_bar, control_column (readouts, setpoint, memória, saída), bottom_bar
    └── chart.rs         # chart_panel (legenda, seletor de cor, slider) e draw_history_chart
```

Os testes ficam nos próprios arquivos de `logics` (`#[cfg(test)] mod tests`). Não há pasta
`tests/` de integração nesta feature.

**Structure Decision**: um único pacote Cargo com uma biblioteca (`src/lib.rs`) e dois binários. O
`Cargo.toml` mantém as seções `[[bin]]` atuais, e a biblioteca é detectada automaticamente por
`src/lib.rs`. A dependência entre módulos é sempre `controller.rs → gui → logics` e
`tests.rs → logics`.

## Ordem de implementação recomendada

Cada passo deve compilar e manter o app funcionando:

1. Criar `src/lib.rs` + `logics/mod.rs`. Mover `dps150.rs` → `logics/protocol.rs`. Trocar `mod dps150`
   nos dois binários por `use dps150::logics::protocol`.
2. Extrair `scale`, `format` e `history` (funções puras) com os testes delas.
3. Extrair `session` (thread serial, com `notify` no lugar de `egui::Context`) e `available_ports`.
4. Extrair `app` (`AppState` e regras, com `now` injetado e enumeração de portas a 1 Hz) com os testes.
5. Mover a interface para `gui/` (`GuiApp` com `AppState` e `series_colors`). Reduzir
   `controller.rs` a `main`.
6. Adicionar os testes de protocolo. Rodar os gates do quickstart e o roteiro de paridade.
7. Atualizar o README. Fazer a emenda PATCH da constituição (caminho do protocolo).

## Complexity Tracking

Nenhuma violação da constituição precisa de justificativa.

## Melhorias futuras (fora do escopo)

- Fixtures de protocolo capturadas de um aparelho real, identificadas por modelo e firmware
  (Princípio II). Esta feature começa com bytes montados pelas próprias funções de comando.
- Débitos de UX já registrados na constituição: faixa de setpoint de 30 V / 5 A e o texto "Update" em
  inglês.
