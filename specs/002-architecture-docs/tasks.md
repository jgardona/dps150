---

description: "Task list for feature 002-architecture-docs"
---

# Tasks: Documentação de arquitetura do software

**Input**: Design documents from `specs/002-architecture-docs/`

**Prerequisites**: [plan.md](./plan.md), [spec.md](./spec.md), [research.md](./research.md),
[data-model.md](./data-model.md), [contracts/doc-format.md](./contracts/doc-format.md),
[quickstart.md](./quickstart.md)

**Tests**: INCLUÍDOS. A conferência de cobertura (FR-015) é um teste de integração em Rust, e o scanner
dela tem testes próprios (Princípio II da constituição).

**Organization**: tarefas agrupadas por user story. Os caminhos são relativos à raiz do repositório.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: pode rodar em paralelo (arquivos diferentes, sem dependência de tarefa incompleta)
- **[Story]**: US1 (visão geral e README), US2 (funções por módulo), US3 (manutenção e conferência)

## Convenções desta feature

- **Nenhum arquivo de `src/` pode ser alterado** (FR-017, SC-009). Só mudam `Cargo.toml` (T001),
  `README.md`, `docs/`, `tests/docs_coverage.rs` e `specs/002-architecture-docs/`.
- **Toda documentação é escrita lendo o código no estado atual**, não o plano da feature 001 (pesquisa
  R5). Em caso de divergência, vale o código.
- **Idioma**: pt-BR, no tom do README. Nomes de tipos e funções em crases. Explicar o *porquê* das
  decisões relevantes (pesquisa R6).
- **Entrada de função**: título `` ### `Nome qualificado` `` (uma linha, um par de crases, único no
  documento) + `*(interna)*` se for privada + tabela com **Propósito**, **Entradas**, **Saída**,
  **Efeitos** + **Como funciona**. Formato completo em [contracts/doc-format.md](./contracts/doc-format.md#2-entrada-de-função-o-que-a-conferência-lê).
- **Funções com regra não óbvia** (FR-009) têm **Exemplo:** com valores de entrada e resultado:
  `AxisScale::for_peak`, `decimals_for`, `DPS150::process_buffer`, `DPS150::parse_data`,
  `DPS150::build_command`, `History::record_if_due`, `OutputClock::tick`,
  `AppState::sync_edit_buffers`, `AppState::merge`, `AppState::refresh_ports_if_due`,
  `AppState::load_profile`, `session_main`, `format_window`, `draw_history_chart`, `GuiApp::ui`.
- **Diagramas**: Mermaid em bloco ` ```mermaid `, com título de uma linha acima, "no máximo ~12 nós",
  rótulos com `()`, `:`, `/` ou `>` entre aspas e nomes reais do código.
- **Compilar neste ambiente**: o projeto real precisa de `libudev` (`sudo dnf install systemd-devel`).
  Sem ela, trabalhar numa cópia temporária com
  `serialport = { version = "4.9.0", default-features = false }` no `Cargo.toml` e rodar `cargo` lá.
- **Branch**: continuar no branch atual (`doc/documentacao-componentes`); não criar outro.

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: destravar o `cargo test` e criar a estrutura de pastas.

- [X] T001 Remover de `Cargo.toml` o bloco `[[bin]]` com `name = "tests"` e `path = "src/tests.rs"` (arquivo apagado no commit `8ecd93f`; pesquisa R9) e confirmar que `cargo check --all-targets` volta a compilar. Não mexer em mais nada do arquivo
- [X] T002 [P] Criar as pastas `docs/` e `docs/modulos/` (com `mkdir -p docs/modulos`)

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: a conferência de cobertura em `tests/docs_coverage.rs`. Ela guia a escrita das fases
seguintes (lista as funções que faltam) e prova o SC-002, o SC-005 e o SC-006.

**⚠️ CRITICAL**: nenhuma user story começa antes desta fase terminar. Tudo fica num só arquivo, então as
tarefas são sequenciais. Usar **só a biblioteca padrão** (nenhuma dependência nova).

- [X] T003 Criar `tests/docs_coverage.rs` com um comentário de módulo `//!` em pt-BR explicando o objetivo e, primeiro, os **testes do scanner** com a assinatura `fn scan_functions(source: &str) -> Result<Vec<String>, String>` ainda como `todo!()`. Casos fixos, um `#[test]` cada, conforme [contracts/doc-format.md](./contracts/doc-format.md#4-conferência-de-cobertura--testsdocs_coveragers):
  - `impl AppState { pub fn new() … }` → `AppState::new`;
  - `impl Default for AppState { fn default() … }` → `AppState::default`;
  - `impl<'a> egui::Widget for ReadoutRow<'a> { fn ui(…) … }` → `ReadoutRow::ui`;
  - `pub(crate) fn card(…)` no nível do módulo → `card`;
  - `fn dentro_de_outra()` aninhada em uma função → ignorada;
  - `#[cfg(test)] mod tests { … }` e `#[cfg(test)] fn …` dentro de um `impl` → ignorados;
  - `pub mod commands { pub const X: u8 = 1; }` (módulo aninhado só com constantes) → nenhuma função;
  - `{` sem `}` correspondente → `Err` mencionando chaves desbalanceadas.
- [X] T004 Em `tests/docs_coverage.rs`, implementar `scan_functions`: varredura por linha mantendo uma pilha de contextos por profundidade de chaves; `impl [<…>] [Trait for] Tipo` define o `Tipo` (sem genéricos, lifetimes nem caminho); reconhecer `fn nome` (com `pub` ou `pub(crate)`) só no nível direto do `impl` ou do módulo; ao ver `#[cfg(test)]`, descartar o item seguinte até a chave dele fechar; devolver `Err` se a profundidade de chaves não voltar a 0 no fim do texto. Fazer todos os testes de T003 passarem
- [X] T005 Em `tests/docs_coverage.rs`, implementar `fn doc_titles(markdown: &str) -> Vec<String>`, que extrai os títulos de função e **ignora blocos cercados** (linhas entre ` ``` `). O título válido é uma linha `` ### `…` `` com exatamente um par de crases, sem texto antes nem depois (expressão `^### \`([^\`]+)\`\s*$` do contrato). Testes: título válido; título com texto extra é ignorado; título dentro de bloco ` ```markdown ` é ignorado; título duplicado aparece duas vezes (a conferência reclama dele)
- [X] T006 Em `tests/docs_coverage.rs`, implementar `fn slug_for(path: &Path) -> String` (caminho do `.rs` sem `src/` e sem `.rs`, com `/` trocado por `-`; `src/gui/mod.rs` → `gui-mod`, `src/controller.rs` → `controller`) com teste, e o teste de integração `funcoes_documentadas`: percorrer `src/**/*.rs` a partir de `CARGO_MANIFEST_DIR`; para cada arquivo com pelo menos uma função, exigir `docs/modulos/<slug>.md` e comparar o conjunto de funções com o de títulos. Falhar com `panic!` na mensagem do contrato, uma linha por problema e ordenada: `faltando em <doc>: <nome>`, `sobrando em <doc>: <nome>`, `documento ausente: <doc> (<arquivo> tem N funções)`, e erro de chaves desbalanceadas. Marcar o teste com `#[ignore = "habilitar quando docs/modulos estiver completo (T032)"]` para o `cargo test` seguir verde durante a escrita
- [X] T007 Em `tests/docs_coverage.rs`, implementar `fn extract_links(markdown: &str) -> Vec<(usize, String)>` (número da linha e destino de cada `[texto](destino)` **fora de blocos cercados**), com testes, e o teste de integração `links_relativos_existem`: para `README.md` e todo `docs/**/*.md`, ignorar destinos que começam com `http://`, `https://`, `mailto:` ou `#`, remover o `#âncora` do final, resolver o resto a partir da pasta do arquivo e exigir que exista. Mensagem de falha: `link quebrado em <arquivo>:<linha> → <destino>`. Âncoras não são verificadas
- [X] T008 Rodar `cargo fmt` e `cargo clippy --all-targets` (esperado: 0 avisos) e `cargo test --test docs_coverage`. Confirmar: os testes do scanner, de `doc_titles`, de `slug_for` e de `extract_links` passam; `links_relativos_existem` **falha** apontando as referências quebradas do README a `src/tests.rs` (é o estado vermelho esperado, corrigido em T016); e `cargo test --test docs_coverage -- --ignored funcoes_documentadas` **falha** listando exatamente **88 funções faltando** e **11 documentos ausentes**, batendo com o mapa de [data-model.md](./data-model.md#mapa-módulo--documento--funções-esperadas). Se a contagem diferir, corrigir o scanner (não o mapa) e investigar

**Checkpoint**: a conferência existe, está testada e mostra o que falta escrever.

---

## Phase 3: User Story 1 - Entender a arquitetura em poucos minutos (Priority: P1) 🎯 MVP

**Goal**: o README leva à documentação, e a visão geral mostra os módulos, os diagramas e os fluxos
(FR-002, FR-003, FR-004, FR-005, FR-006, FR-010, FR-011, FR-012, FR-013, FR-016).

**Independent Test**: com a documentação existente nesta fase, responder só por ela às 3 perguntas da
leitura guiada ([quickstart.md](./quickstart.md#5-leitura-guiada-sc-001-us1)): onde fica o setpoint
pendente, onde fica o desenho do gráfico e como um comando chega à fonte.

> Nesta fase, `docs/arquitetura.md` referencia os módulos por **links para os arquivos de `src/`**, não
> para `docs/modulos/` (que só existe na US2). Assim não há link quebrado no checkpoint.

### Implementation for User Story 1

- [X] T009 [US1] Criar `docs/README.md` (índice, pt-BR): título, um parágrafo dizendo o que a pasta contém, o link para `arquitetura.md` com uma linha de descrição, e uma "ordem de leitura" por perfil (entender a arquitetura → alterar um módulo → manter a documentação). Os itens de módulos e de manutenção entram nas tarefas T031 e T035
- [X] T010 [US1] Criar `docs/arquitetura.md` com introdução e a seção **"Módulos"**: uma tabela com **todos** os arquivos de `src/` (`controller.rs`, `lib.rs`, `logics/mod.rs`, `logics/{protocol,session,app,history,scale,format}.rs`, `gui/mod.rs`, `gui/{panels,chart,widgets,theme}.rs`), com link para o arquivo, o **papel** e **o que ele não faz** (FR-004, FR-006). Ler cada arquivo antes; o monitor `tests` não existe mais e não aparece
- [X] T011 [US1] Em `docs/arquitetura.md`, adicionar **D1** (`flowchart`, "Dependências entre módulos": `controller → gui → logics` e as dependências internas) e **D2** (`flowchart`, "Fluxo de dados": fonte ⇄ thread da sessão ⇄ canais `mpsc` ⇄ `AppState` ⇄ `GuiApp`), cada um com um parágrafo explicando. Antes de desenhar D1, confirmar as dependências reais com `rg "use crate::logics|use super" src/`
- [X] T012 [US1] Em `docs/arquitetura.md`, adicionar **D3** (`sequenceDiagram`, "Ação do usuário até a fonte": clique em "Aplicar" em `gui::panels::setpoint_card` → `AppState::apply_setpoints` → canal de comandos → `session_main` → porta serial → fonte, e a resposta voltando por `SerialEvent::Update` → `AppState::poll_events`) e **D6** (`flowchart`, "Ciclo por quadro": a ordem em que `GuiApp::ui` chama `refresh_ports_if_due`, `poll_events`, `tick`, desenha os painéis e pede o próximo redesenho), com texto explicando cada passo
- [X] T013 [US1] Em `docs/arquitetura.md`, adicionar **D4** (`sequenceDiagram`, "Ciclo de vida da sessão": `spawn_session`, abrir a porta a 115200 baud, DTR/RTS, espera de 2 s, `init_command` com 100 ms entre comandos, evento `Connected`, laço de comandos/leitura/`GET ALL`, `Disconnect` com `lock_session`) e **D5** (`stateDiagram-v2`, "Estado da conexão": `Disconnected`, `Connecting`, `Connected`, `Error` e as transições `connect`, `disconnect`, evento `Connected`, evento `Error`, e `connect` com porta vazia indo direto a `Error`), com texto cobrindo o FR-012. Conferir cada transição lendo `AppState::connect`, `disconnect` e `poll_events`
- [X] T014 [US1] Em `docs/arquitetura.md`, adicionar a seção **"Tipos centrais"** (FR-011) com uma tabela que descreve `AppState`, `ConnectionState`, `SetpointEditor`, `OutputClock`, `History`/`Sample`, `AxisScale`, `AppCommand`, `SerialEvent` e `DPSUpdate` (campos principais e papel), e a seção **"Regras de projeto"**: `logics` não usa `gui` nem egui; `gui` não acessa a serial nem o protocolo; nenhuma E/S serial na thread da interface; o relógio (`now`) e o aviso de redesenho (`notify`) entram por parâmetro, e por quê (testar sem janela e sem porta)
- [X] T015 [US1] Em `README.md`, adicionar o link para `docs/README.md` **logo depois da introdução e antes de "## Sumário"** (ex.: "📚 Documentação de arquitetura: [docs/](docs/README.md)", sem alterar o resto), um item no Sumário e, na seção "Estrutura do código", um link para `docs/arquitetura.md`. Trocar a tabela longa de arquivos dessa seção por um resumo curto (três linhas: `controller.rs` → `gui` → `logics`) que aponte para a documentação, para não manter a lista em dois lugares (pesquisa R7)
- [X] T016 [US1] Em `README.md`, **remover todas as referências ao monitor `tests`** (pesquisa R7, item 2): a linha `tests` da tabela de binários e o texto "dois binários", o item do Sumário, a seção inteira "Monitor de linha de comando (`tests`)", a linha `tests.rs` do diagrama de dependências, a linha da tabela de arquivos e o item "Monitor `tests`" das limitações. Conferir com `grep -nE 'tests\.rs|--bin tests|monitor' README.md` (sem resultado, exceto menção a `cargo test`)
- [X] T017 [US1] Verificar a US1: rodar `cargo test --test docs_coverage links_relativos_existem` (esperado: passa, sem links quebrados em `README.md`, `docs/README.md` e `docs/arquitetura.md`); `grep -c '```mermaid' docs/arquitetura.md` (esperado: **6**); revisar a sintaxe de cada diagrama contra as regras da seção Convenções; e responder às 3 perguntas da leitura guiada usando **só** `README.md` e `docs/`, anotando em quais seções achou cada resposta

**Checkpoint**: a arquitetura já é compreensível pela documentação. Este é o MVP.

---

## Phase 4: User Story 2 - Saber o que cada função faz e como funciona (Priority: P2)

**Goal**: um documento por módulo, com todas as 88 funções documentadas (FR-007, FR-008, FR-009).

**Independent Test**: `cargo test --test docs_coverage -- --ignored funcoes_documentadas` passa (nada
faltando, sobrando nem ausente), e uma amostra de 10 funções tem as 5 partes da ficha preenchidas.

### Implementation for User Story 2

Cada documento começa com **Papel** (o que o módulo faz), **O que não faz**, **Tipos** do módulo e,
quando houver testes, uma frase sobre o que eles cobrem. Depois vêm as entradas **na ordem do arquivo**.
Os nomes são os do mapa em [data-model.md](./data-model.md#mapa-módulo--documento--funções-esperadas);
`*` = privada, marcar `*(interna)*`.

- [X] T018 [P] [US2] `docs/modulos/logics-scale.md` (fonte: `src/logics/scale.rs`): `decimals_for`, `AxisScale::for_peak`. Em `for_peak`, explicar o passo 1-2-5 × 10ⁿ e o passo mínimo, com **Exemplo:** pico 3,3 V com passo mínimo 0,1 → topo 4, 2 casas; pico 0 A com passo mínimo 0,01 → topo 0,04, 4 casas
- [X] T019 [P] [US2] `docs/modulos/logics-format.md` (fonte: `src/logics/format.rs`): `format_hhmmss`, `format_window`. Em `format_window`, **Exemplo:** 30 s → `"30 s"`, 150 s → `"2.5 min"`, 300 s → `"5 min"`
- [X] T020 [P] [US2] `docs/modulos/logics-history.md` (fonte: `src/logics/history.rs`): `History::default`, `new`, `record_if_due`, `clear`, `iter`, `len`, `is_empty`, `last`. Em `record_if_due`, explicar o intervalo e o descarte da amostra mais antiga ("Com a fila cheia, descarta a mais antiga antes de inserir", limite `HISTORY_LEN = 300`), com **Exemplo:** amostra aos 0 ms gravada; aos 50 ms ignorada; aos 100 ms gravada (intervalo de 100 ms). Documentar também `Sample`
- [X] T021 [P] [US2] `docs/modulos/logics-protocol.md` (fonte: `src/logics/protocol.rs`): `DPS150::default`, `new`, `init_command`, `get_protection`, `get_all`, `lock_session`, `group_voltage_id`, `group_current_id`, `enable_output`, `set_float_value`, `build_command`*, `push_serial_data`, `process_buffer`*, `parse_data`*, `read_float`* (15). Não repetir o formato de pacote: apontar para a seção "Protocolo" do `README.md`. **Exemplos:** `build_command` (checksum de `enable_output(true)` = `[0xF1, 0xB1, 219, 1, 1, 221]`), `process_buffer` (lixo antes do cabeçalho e pacote dividido em duas leituras), `parse_data` (type_id 195 → tensão, corrente e potência nos offsets 0, 4 e 8), `read_float` (4 bytes little-endian). Documentar `DPSUpdate`, `commands` e as constantes de rótulos
- [X] T022 [P] [US2] `docs/modulos/logics-session.md` (fonte: `src/logics/session.rs`): `spawn_session`, `available_ports`, `session_main`*. Documentar `AppCommand` e `SerialEvent`. Em `session_main`, descrever passo a passo (abrir a porta, DTR/RTS, espera de 2 s, inicialização, laço de comandos/leitura/`GET ALL`, desconexão) com **Exemplo:** o que acontece quando a porta não abre (evento `Error` com "Falha ao abrir …" e `notify`)
- [X] T023 [US2] `docs/modulos/logics-app.md` **parte 1** (fonte: `src/logics/app.rs`): cabeçalho (Papel, O que não faz, Tipos: `ConnectionState`, `SetpointEditor`, `OutputClock`, `AppState`, constantes `UPDATE_MS_RANGE` e `PORT_REFRESH_INTERVAL`) e 21 entradas: `SetpointEditor::is_dirty`, `OutputClock::tick`*, `OutputClock::elapsed`*, `AppState::default`, `new`, `device`, `connection`, `is_connected`, `is_idle`, `output_on`, `available_ports`, `selected_port`, `selected_port_mut`, `setpoint`, `setpoint_mut`, `selected_profile`, `group_values`, `history`, `update_ms`, `update_interval`, `current_elapsed`. **Exemplo** em `OutputClock::tick`: saída liga em t0 → `elapsed(t0+5 s)` = 5 s; desliga em t0+5 s → `elapsed` continua 5 s. Em `output_on`, explicar que `output_opened == Some(false)` significa saída **ligada**
- [X] T024 [US2] `docs/modulos/logics-app.md` **parte 2**, continuando o mesmo arquivo: 14 entradas de ciclo e ações: `AppState::refresh_ports_if_due`, `poll_events`, `tick`, `connect`, `disconnect`, `mark_setpoint_edited`, `set_editing`, `apply_setpoints`, `discard_setpoint_edits`, `load_profile`, `save_profile`, `toggle_output`, `set_update_ms`, `clear_history`. **Exemplos:** `refresh_ports_if_due` (chamada aos 0,3 s da última não consulta; aos 1 s consulta), `load_profile` (carrega M2 sem enviar nada; só `apply_setpoints` envia), `set_update_ms` (5000 → 1000, limitado a `UPDATE_MS_RANGE` = 100..=1000, e envia `SetPollInterval`). Em cada ação, deixar explícito **se envia comando à fonte** (Princípio III da constituição)
- [X] T025 [US2] `docs/modulos/logics-app.md` **parte 3**, continuando o mesmo arquivo: 6 entradas internas `*(interna)*`: `AppState::send`, `end_session`, `reset_telemetry`, `set_available_ports`, `merge`, `sync_edit_buffers`. **Exemplos:** `merge` (atualização só com `output_voltage` mantém a `temperature` anterior), `sync_edit_buffers` (com edição pendente, `vset_input` 5,0 não é sobrescrito por `vset` 3,3 vindo da fonte; sem pendência, vira 3,3). Terminar com a seção **"Testes"**, resumindo o que os 9 testes do módulo cobrem
- [X] T026 [P] [US2] `docs/modulos/gui-widgets.md` (fonte: `src/gui/widgets.rs`): `ReadoutRow::ui`, `card`, `info_pair`, `status_dot`, `column_ui`. Documentar `ReadoutRow` (campos `unit`, `accent`, `value`, `badge`)
- [X] T027 [P] [US2] `docs/modulos/gui-chart.md` (fonte: `src/gui/chart.rs`): `chart_panel`, `draw_history_chart`*. Em `draw_history_chart`, explicar a escala própria por série, os ticks principais e intermediários, a regra que esconde os rótulos intermediários em gráfico baixo e o alinhamento do ponto mais recente à direita, com **Exemplo:** histórico com 3,3 V de pico → eixo de V de 0 a 4 com ticks a cada 0,25
- [X] T028 [P] [US2] `docs/modulos/gui-panels.md` (fonte: `src/gui/panels.rs`): `top_bar`, `control_column`, `readouts`*, `setpoint_card`*, `output_button`*, `bottom_bar`. Em cada uma, dizer qual ação do usuário chama qual método de `AppState`
- [X] T029 [P] [US2] `docs/modulos/gui-mod.md` (fonte: `src/gui/mod.rs`): `GuiApp::default`, `GuiApp::new`, `GuiApp::ui`. Documentar `GuiApp` e as constantes de `gui/theme.rs` (citadas aqui porque `theme` não tem funções). Em `GuiApp::ui`, **Exemplo:** do cálculo da coluna de controle (`CONTROL_SHARE` 0,36, limitada a `CONTROL_MIN_WIDTH` 330 e `CONTROL_MAX_WIDTH` 440; área de 1000 px → 360 px; de 1600 px → 440 px) e a ordem do ciclo por quadro, remetendo ao diagrama D6
- [X] T030 [P] [US2] `docs/modulos/controller.md` (fonte: `src/controller.rs`): `main`. Explicar as constantes `WIDTH` e `HEIGHT`, a criação das opções da janela e a chamada de `eframe::run_native` com `GuiApp::new()`
- [X] T031 [US2] Em `docs/README.md`, adicionar a seção **"Módulos"** com um item por documento de `docs/modulos/` (link relativo + uma linha), e em `docs/arquitetura.md`, adicionar ao fim da tabela de "Módulos" uma coluna ou seção "Documentação das funções" com o link para o documento de cada módulo
- [X] T032 [US2] Habilitar a conferência: remover o `#[ignore …]` de `funcoes_documentadas` em `tests/docs_coverage.rs` e rodar `cargo test --test docs_coverage`. Esperado: **tudo passa**, sem funções faltando, sobrando ou ausentes (SC-002, SC-006). Corrigir as pendências apontadas (títulos, nomes qualificados, `*(interna)*`)
- [X] T033 [US2] Verificar a completude das fichas com um comando e registrar o resultado no fim de `specs/002-architecture-docs/quickstart.md`: para cada seção `###` de `docs/modulos/*.md`, conferir a presença de `**Propósito**`, `**Entradas**`, `**Saída**`, `**Efeitos**` e `**Como funciona**` (por exemplo, com `awk` contando os campos por seção) → esperado: 0 seções incompletas (SC-003). Conferir também que cada uma das 15 funções da lista de regra não óbvia tem `**Exemplo:**` na própria seção (FR-009)

**Checkpoint**: US1 e US2 completas. Todas as funções estão documentadas e conferidas.

---

## Phase 5: User Story 3 - Manter a documentação confiável ao longo do tempo (Priority: P3)

**Goal**: regras de manutenção escritas e a conferência provadamente capaz de detectar divergência
(FR-014, FR-015).

**Independent Test**: alterar o código ou a documentação de forma temporária e ver a conferência falhar
com a mensagem certa (seção 2 do [quickstart](./quickstart.md#2-a-conferência-realmente-detecta-divergência-us3)).

### Implementation for User Story 3

- [X] T034 [US3] Criar `docs/manutencao.md` (pt-BR) com: **quando atualizar** (função nova, renomeada ou removida; módulo novo; mudança de tipo central; mudança de fluxo ou de transição de estado → rever os diagramas); **como escrever uma entrada** (modelo do contrato §2, regra do nome qualificado com exemplos `AppState::new`, `format_hhmmss`, `ReadoutRow::ui`, marcação `*(interna)*`, exemplo obrigatório nas funções de regra não óbvia); **como documentar um módulo novo** (regra do `slug`); **como rodar a conferência** (`cargo test --test docs_coverage`); **como ler a falha** (mensagem de exemplo do contrato §4); e os **limites do scanner** (funções aninhadas são ignoradas; chaves desbalanceadas em texto literal quebram a contagem)
- [X] T035 [US3] Em `docs/README.md`, adicionar o link para `manutencao.md` com uma linha de descrição e incluir a manutenção na "ordem de leitura" (perfil "manter a documentação"). Conferir com `cargo test --test docs_coverage links_relativos_existem`
- [X] T036 [US3] **Prova de detecção**: executar as quatro alterações temporárias da seção 2 do `specs/002-architecture-docs/quickstart.md` (renomear o título de `History::clear`; adicionar `fn teste_temporario() {}` em `src/logics/format.rs`; apagar `docs/modulos/gui-chart.md`; trocar um link por `modulos/nao-existe.md`), rodar `cargo test --test docs_coverage` a cada uma, confirmar a mensagem esperada e **desfazer cada alteração** (usar `git stash`/`git checkout -- <arquivo>` ou restaurar a cópia). Ao fim, `git status` não pode mostrar mudança em `src/` nem arquivo apagado em `docs/`. Anotar o resultado no fim do quickstart

**Checkpoint**: as três stories estão completas.

---

## Phase 6: Polish & Cross-Cutting Concerns

- [X] T037 Rodar, na raiz do repositório (`Cargo.toml`), os gates automáticos: `cargo fmt --check`, `cargo clippy --all-targets` (esperado: 0 avisos), `cargo test` (esperado: 24 testes da biblioteca + os de `docs_coverage` passando) e `cargo build --release --bin controller`
- [X] T038 Rodar as verificações de estrutura da seção 3 do `specs/002-architecture-docs/quickstart.md`: `ls docs docs/modulos`; `grep -n 'docs/README.md' README.md` (≥ 2 ocorrências); `grep -rnE 'tests\.rs|--bin tests' README.md docs/` (nenhuma); `grep -c '```mermaid' docs/arquitetura.md` (6); `git diff --stat -- src/ Cargo.lock` (vazio, **SC-009**)
- [ ] T039 **Manual (mantenedor)**: abrir `docs/arquitetura.md` no visualizador do repositório (página do branch ou PR no GitHub) e confirmar que **D1–D6 aparecem desenhados**, sem "Unable to render" (SC-007). Se algum falhar, corrigir a sintaxe do diagrama e repetir
- [ ] T040 **Manual (mantenedor)**: leitura guiada com alguém que não conhece o código (SC-001): em até 10 minutos, só com `README.md` e `docs/`, responder às 3 perguntas da seção 5 de `specs/002-architecture-docs/quickstart.md`
- [X] T041 Amostra de 10 funções (SC-003): sortear 10 funções de `src/` (por exemplo, `rg -n '^\s*(pub(\([a-z]+\))? )?fn ' src | shuf -n 10`) e conferir, no documento do módulo, que propósito, entradas, saída, efeitos e "Como funciona" fazem sentido em relação ao código (não só que existem)
- [X] T042 Revisão final: `git status` e `git diff --stat` devem mostrar mudanças **só** em `Cargo.toml`, `README.md`, `docs/`, `tests/docs_coverage.rs` e `specs/002-architecture-docs/`. Marcar as tarefas no `tasks.md`. O commit fica para o mantenedor (não commitar automaticamente)

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: sem dependências. T001 destrava todo `cargo` das fases seguintes.
- **Foundational (Phase 2)**: depende de T001. T003 → T004 → T005 → T006 → T007 → T008 (mesmo arquivo).
- **US1 (Phase 3)**: depende da Foundational. T009 → T010 → T011 → T012 → T013 → T014 (mesmo arquivo
  `docs/arquitetura.md`); T015 → T016 (mesmo `README.md`); T015/T016 podem andar em paralelo com T010–T014
  (arquivos diferentes); T017 fecha a fase.
- **US2 (Phase 4)**: depende da Foundational e **não** depende da US1 para ser escrita, mas T031 edita
  arquivos da US1 (`docs/README.md`, `docs/arquitetura.md`), então vem depois de T009–T014. T018–T022,
  T026–T030 são independentes entre si; T023 → T024 → T025 (mesmo arquivo); T032 depende de T018–T031;
  T033 depende de T032.
- **US3 (Phase 5)**: T034 independente; T035 depende de T009 e T034; T036 depende de T032 (a
  conferência precisa estar habilitada e verde para provar que detecta).
- **Polish (Phase 6)**: depende de todas as stories. T039 e T040 são manuais.

### User Story Dependencies

- **US1 (P1)**: só depende da Foundational. É o MVP.
- **US2 (P2)**: independente da US1 na escrita; a conferência (T032) depende de todos os módulos.
- **US3 (P3)**: a prova de detecção (T036) depende da US2.

### Parallel Opportunities

- Phase 1: T002 em paralelo com T001.
- Phase 3: T015/T016 (`README.md`) em paralelo com T010–T014 (`docs/arquitetura.md`).
- Phase 4: T018, T019, T020, T021, T022, T026, T027, T028, T029, T030 e a sequência T023→T025,
  todos em arquivos diferentes.
- Phase 5: T034 em paralelo com a Phase 4.

---

## Parallel Example: User Story 2

```bash
# Documentos de módulo independentes (um arquivo cada):
Task: "T018 docs/modulos/logics-scale.md"
Task: "T019 docs/modulos/logics-format.md"
Task: "T020 docs/modulos/logics-history.md"
Task: "T021 docs/modulos/logics-protocol.md"
Task: "T022 docs/modulos/logics-session.md"
Task: "T026 docs/modulos/gui-widgets.md"
Task: "T027 docs/modulos/gui-chart.md"
Task: "T028 docs/modulos/gui-panels.md"
Task: "T029 docs/modulos/gui-mod.md"
Task: "T030 docs/modulos/controller.md"
# em paralelo com a sequência de logics-app.md: T023 → T024 → T025
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Phase 1 (Setup) e Phase 2 (Foundational): `cargo test` volta a funcionar e a conferência existe.
2. Phase 3 (US1) até o T017: **pare e valide** com a leitura guiada.
3. Neste ponto o README leva à documentação e a arquitetura já é compreensível.

### Incremental Delivery

1. Setup + Foundational: ferramenta de conferência pronta (vermelha para as funções, de propósito).
2. US1: visão geral e README (MVP).
3. US2: módulos, do núcleo para a borda (`logics-scale` … `logics-app`, depois `gui-*`); a lista de
   pendências da conferência cai a cada documento.
4. US3: manutenção e prova de detecção.
5. Polish: gates, verificações de estrutura, diagramas no GitHub, leitura guiada e amostra.

---

## Notes

- `funcoes_documentadas` fica com `#[ignore]` de T006 até T032, para `cargo test` continuar verde
  durante a escrita. Para ver as pendências no meio do caminho:
  `cargo test --test docs_coverage -- --ignored funcoes_documentadas`.
- Em caso de dúvida sobre o formato, [contracts/doc-format.md](./contracts/doc-format.md) é a fonte; para
  nomes e quantidades, [data-model.md](./data-model.md).
- Fora do escopo: emenda PATCH da constituição (1.0.2) para remover a menção ao monitor `tests`;
  comentários `///` no código.
