---

description: "Task list for feature 003-memory-setup-fields"
---

# Tasks: Campos de configuração das memórias

**Input**: Design documents from `specs/003-memory-setup-fields/`

**Prerequisites**: [plan.md](./plan.md), [spec.md](./spec.md), [research.md](./research.md),
[data-model.md](./data-model.md), [contracts/ui-and-serial.md](./contracts/ui-and-serial.md),
[quickstart.md](./quickstart.md)

**Tests**: INCLUÍDOS (Princípio II da constituição): validação, limites, envio de comandos e pacotes do
protocolo com testes sem hardware; a escrita real das proteções tem roteiro manual (T037).

**Organization**: tarefas agrupadas por user story. Caminhos relativos à raiz do repositório.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: pode rodar em paralelo (arquivos diferentes, sem dependência de tarefa incompleta)
- **[Story]**: US1 (ver os seis valores), US2 (editar e salvar), US3 (validar e faixas)

## Convenções desta feature

- **`docs_coverage` exige um título para cada função de `src/`.** Por isso a documentação de cada função
  é escrita **na mesma fase** que o código; ao fim de cada fase, `cargo test --test docs_coverage` DEVE
  passar. Formato das entradas: [docs/manutencao.md](../../docs/manutencao.md).
- **Sem `unwrap`, `expect` ou `panic!`** em código de produção (Princípio I). Textos da interface em
  pt-BR.
- **Nenhum comando é enviado sem clique em Salvar, Aplicar ou no botão de saída** (Princípio III).
- **Máximos** (FR-013) vêm só das constantes de `src/logics/memory_setup.rs`; nenhum outro arquivo repete
  esses números.
- O ambiente sem `libudev` roda o cargo por uma cópia de trabalho; no projeto real, use `cargo` direto.

## Phase 1: Setup

- [X] T001 Rodar a linha de base e anotar o resultado em `specs/003-memory-setup-fields/quickstart.md` (seção "Registro da execução"): `cargo fmt --check`, `cargo clippy --all-targets` e `cargo test` devem passar antes de qualquer mudança
- [X] T002 Criar `src/logics/memory_setup.rs` com só o comentário de módulo (`//!`, pt-BR: regra pura dos campos de setup das memórias, sem egui) e registrar `pub mod memory_setup;` em `src/logics/mod.rs` (ordem alfabética, depois de `history`)

---

## Phase 2: Foundational (bloqueia todas as histórias)

**Propósito**: a regra pura que as três histórias usam: campos, máximos, edição pendente e validação.

- [X] T003 Em `src/logics/memory_setup.rs`, criar as constantes nomeadas `VSET_MAX = 36.00`, `ISET_MAX = 8.200`, `OVP_MAX = 36.10`, `OCP_MAX = 8.200`, `OPP_MAX = 295.20`, `OTP_MAX = 99.0` e o `enum SetupField { Vset, Iset, Ovp, Ocp, Opp, Otp }` (`Debug, Clone, Copy, PartialEq, Eq`) com `SetupField::ALL: [SetupField; 6]` na ordem do visor e os métodos `label()` ("Vset", "Iset", "OVP", "OCP", "OPP", "OTP"), `unit()` ("V", "A", "V", "A", "W", "°C"), `max()` e `decimals()` (2, 3, 2, 3, 2, 0), conforme a tabela de data-model.md; `is_global()` devolve `true` para Ovp, Ocp, Opp e Otp. Testes: cada campo devolve o máximo, a unidade e as casas da tabela
- [X] T004 Em `src/logics/memory_setup.rs`, criar `SetupValues { group: u8, vset, iset, ovp, ocp, opp, otp: f32 }` (`Debug, Clone, Copy, PartialEq`) e `enum SetupError { Missing(SetupField), AboveMax(SetupField), VsetAboveOvp, IsetAboveOcp }` com `field()` (`Option<SetupField>`; `VsetAboveOvp`→Vset, `IsetAboveOcp`→Iset) e `message()` em pt-BR exatamente: "<rótulo>: valor ausente", "<rótulo>: máximo de <máx com as casas> <unidade>", "Vset não pode passar do OVP", "Iset não pode passar do OCP". Testes das quatro mensagens (ex.: OVP → "OVP: máximo de 36.10 V")
- [X] T005 Em `src/logics/memory_setup.rs`, criar `MemorySetup` (`Debug, Default`) com seis `Option<f32>` (`None` = fonte ainda não informou) e `dirty: bool`, e os métodos `get(field)`, `set(field, value)` (marca `dirty = true`), `load(field, Option<f32>)` (grava sem marcar pendência), `is_dirty()`, `clear_dirty()`. Testes: `set` marca pendência; `load` não marca; começa tudo `None` e sem pendência
- [X] T006 Em `src/logics/memory_setup.rs`, criar `MemorySetup::validate(&self, group: u8) -> Result<SetupValues, Vec<SetupError>>`, que devolve **todos** os erros de uma vez, na ordem de `SetupField::ALL`: valor `None` ou não finito → `Missing`; valor negativo ou `> max()` → `AboveMax`; depois, só se Vset e OVP forem válidos, `Vset > OVP` → `VsetAboveOvp`; só se Iset e OCP forem válidos, `Iset > OCP` → `IsetAboveOcp`; o `group` é só copiado para `SetupValues` (o chamador garante 1..6; documentar isso). Testes obrigatórios: cada campo aceita o máximo exato (36.00, 8.200, 36.10, 8.200, 295.20, 99) e rejeita o valor logo acima; zero é aceito; negativo e `NaN` são recusados; `None` é `Missing`; Vset 12 com OVP 10 → `VsetAboveOvp`; Iset 3 com OCP 2 → `IsetAboveOcp`; Vset = OVP é aceito; vários erros juntos são todos devolvidos
- [X] T007 [P] Em `src/logics/protocol.rs` (só no bloco `#[cfg(test)]`), acrescentar testes de fixture de bytes: `set_float_value(commands::OVP, 36.0)`, `OCP`, `OPP`, `OTP` e, para o grupo 2, `group_voltage_id(2)` e `group_current_id(2)` produzem pacotes `0xf1 0xb1 <type_id> 0x04 <f32 LE> <checksum>` com o checksum calculado à mão no próprio teste (type_ids 209, 210, 211, 212, 199 e 200); `group_voltage_id(0)` e `group_voltage_id(7)` devolvem `None`
- [X] T008 [P] Criar `docs/modulos/logics-memory_setup.md` no formato de [docs/manutencao.md](../../docs/manutencao.md) (Papel, O que não faz, Tipos, Constantes com a tabela de máximos, uma entrada por função de T003–T006, com `**Exemplo:**` em `validate`), e incluir o módulo em `docs/README.md` (tabela "Módulos") e na lista de documentação de funções e na tabela de módulos de `docs/arquitetura.md`
- [X] T009 Rodar `cargo fmt --check`, `cargo clippy --all-targets`, `cargo test --lib` e `cargo test --test docs_coverage`; tudo DEVE passar (checkpoint da fase)

**Checkpoint**: regra pura pronta, testada e documentada.

---

## Phase 3: User Story 1 - Ver os seis valores da memória selecionada (Priority: P1) 🎯 MVP

**Goal**: o quadro de setup aparece no espaço vazio e mostra Vset, Iset, OVP, OCP, OPP e OTP da memória
selecionada; trocar de memória não envia nada.

**Independent Test**: conectar, clicar em M1…M6 e conferir que o quadro mostra os valores da memória
clicada, iguais ao visor da fonte; sem conexão, os campos aparecem desabilitados com "—".

### Tests for User Story 1

- [X] T010 [US1] Em `src/logics/app.rs` (módulo de testes), escrever testes que falham antes do código: (a) com `with_test_session`, um `SerialEvent::Update` com `g2_vset=5.0, g2_cset=0.5, ovp=36.0, ocp=2.5, opp=295.2, otp=60.0` e `select_memory(2)` deixam `memory_setup()` com esses seis valores; (b) `select_memory(3)` troca só Vset/Iset e mantém as proteções; (c) nenhuma dessas ações coloca comando no canal (`cmds.try_recv()` é `Err`); (d) sem dado do grupo, os campos de Vset/Iset ficam `None`; (e) `use_memory_in_setpoint(2)` preenche `setpoint().vset_input`/`cset_input`, marca "não aplicado" e não envia nada (migrar o teste de `load_profile`)

### Implementation for User Story 1

- [X] T011 [US1] Em `src/logics/app.rs`, adicionar o campo `memory_setup: MemorySetup` a `AppState` (e a `new`), os métodos `memory_setup(&self) -> &MemorySetup`, `select_memory(&mut self, n: u8)` (grava `selected_profile`, descarta a pendência e recarrega os campos da fonte) e `sync_memory_setup(&mut self)` (privado: sem pendência, `load` de Vset/Iset do `group_values(selected_profile)` e de `ovp`, `ocp`, `opp`, `otp` do estado; chamado depois de cada `merge` em `poll_events` e em `select_memory`). Renomear `load_profile` para `use_memory_in_setpoint(n)` mantendo o corpo atual (preenche o card Setpoint, marca pendente, não envia). Comentários em pt-BR explicando o porquê
- [X] T012 [US1] Em `src/logics/app.rs`, na função `reset_telemetry`, limpar também o `memory_setup` (volta a `MemorySetup::default()`), para uma nova conexão não mostrar valores da sessão anterior
- [X] T013 [P] [US1] Em `src/gui/widgets.rs`, criar `setup_field(ui, field: SetupField, value: Option<f32>, accent: Option<Color32>, enabled: bool) -> Option<f32>`: uma célula com o rótulo pequeno em cima (cor `accent` para V/A/W, cinza fraco nos demais; `OVP`, `OCP`, `OPP` e `OTP` terminam com `*`) e embaixo o valor com `field.decimals()` casas e `field.unit()`; `None` mostra "—" desabilitado. Nesta fase só exibe (devolve `None`); a edição vem em T019
- [X] T014 [US1] Em `src/gui/panels.rs`, criar `setup_card(ui, app, colors)` (privada) com o título "Setup Mn" e a grade de duas linhas por três campos, na ordem de `SetupField::ALL` (Vset, Iset, OVP / OCP, OPP, OTP), cores V=`colors[0]`, A=`colors[1]`, W=`colors[2]` (as de V para Vset/OVP, as de A para Iset/OCP, a de W para OPP; OTP sem cor), campos desabilitados sem conexão, e o rodapé fraco "* vale para o aparelho todo"; chamá-lo em `control_column` entre o card Setpoint e o botão de saída. Trocar `app.load_profile(n)` por `app.select_memory(n)` no clique de M1–M6, mantendo a dica de hover com os valores salvos
- [X] T015 [US1] Em `src/gui/panels.rs`, conferir o encaixe do quadro na janela de 540 px de altura (pesquisa R8): se o botão de saída for empurrado para fora, aumentar `HEIGHT` em `src/controller.rs` e registrar o novo valor em `docs/modulos/controller.md` (tabela de constantes)
- [X] T016 [P] [US1] Atualizar a documentação das funções alteradas ou novas desta fase: `docs/modulos/logics-app.md` (`AppState::memory_setup`, `select_memory`, `sync_memory_setup`, `use_memory_in_setpoint` no lugar de `load_profile`, `reset_telemetry` e `AppState::poll_events` se o texto mudar), `docs/modulos/gui-widgets.md` (`setup_field`), `docs/modulos/gui-panels.md` (`setup_card`, `control_column`, `setpoint_card`, com o texto da linha de memórias)
- [X] T017 [US1] Em `README.md`, seção "Memórias M1–M6": descrever o quadro de setup (seis campos, ordem do visor, "—" sem dado), que clicar numa memória só mostra os valores, (o botão "Usar no setpoint" é descrito na T026); atualizar o texto de "Passe o mouse…" e "Clique num grupo…"
- [X] T018 [US1] Rodar `cargo fmt --check`, `cargo clippy --all-targets`, `cargo test` e abrir o aplicativo para ver o quadro (sem fonte: campos "—" desabilitados); `docs_coverage` DEVE passar

**Checkpoint**: US1 entregue e demonstrável (MVP): ver os valores, sem editar.

---

## Phase 4: User Story 2 - Editar os campos e salvar na fonte (Priority: P1)

**Goal**: os seis campos são editáveis; **Salvar** valida e envia os seis valores; a pendência é sinalizada
e preservada; a gravação é conferida pela releitura.

**Independent Test**: com a fonte, em M2 mudar os seis campos, **Salvar** e conferir o visor e a releitura
("gravado"); com edição pendente, clicar em outra memória mostra o aviso de descarte.

### Tests for User Story 2

- [X] T019 [US2] Em `src/logics/app.rs` (testes), escrever testes que falham antes do código: (a) `set_setup_field(Ovp, 30.0)` marca `memory_setup().is_dirty()` e uma `Update` posterior com `ovp=36.0` NÃO sobrescreve o 30.0; (b) `discard_memory_setup_edits()` volta ao valor da fonte e limpa a pendência; (c) `save_memory_setup()` com seis valores válidos e `selected_profile = 2` coloca exatamente um `AppCommand::SaveMemorySetup(SetupValues { group: 2, .. })` no canal e limpa a pendência; (d) com um valor inválido (Vset 12 e OVP 10) não envia nada e devolve os erros; (e) sem conexão não envia nada; (f) `request_select_memory(3)` com pendência NÃO troca de memória e deixa `pending_switch() == Some(3)`, sem enviar nada; `confirm_switch()` troca e descarta; `cancel_switch()` mantém a edição; sem pendência `request_select_memory` troca direto; (g) conferência: depois de salvar, `save_status()` é `Verifying`; uma `Update` com os mesmos seis valores (tolerância de meio passo: 0,005 V, 0,0005 A, 0,005 W, 0,5 °C) leva a `Confirmed`; três `Update` com `ovp` diferente levam a `NotConfirmed`; uma `Update` sem `ovp` (leitura parcial) NÃO consome tentativa

### Implementation for User Story 2

- [X] T020 [US2] Em `src/logics/session.rs`, trocar `AppCommand::SaveProfile(u8, f32, f32)` por `AppCommand::SaveMemorySetup(SetupValues)` e, em `session_main`, executar na ordem de [contracts/ui-and-serial.md](./contracts/ui-and-serial.md) seção 3: `set_float_value` de `commands::OVP`, `OCP`, `OPP`, `OTP`, depois o Vset e o Iset do grupo (`group_voltage_id`/`group_current_id`; grupo sem id → não envia os dois últimos), e por fim `get_all()`, com `thread::sleep(Duration::from_millis(100))` entre os pacotes (como na inicialização). Comentário no código: escrita das proteções **experimental**, não confirmada em aparelho real
- [X] T021 [US2] Em `src/logics/app.rs`, implementar: `set_setup_field(&mut self, field, value: f32)` (chama `memory_setup.set`); `discard_memory_setup_edits()`; `save_memory_setup(&mut self) -> Result<(), Vec<SetupError>>` (sem conexão devolve `Err(vec![])` e não envia; com conexão, `validate(selected_profile)`, e só em `Ok` envia `AppCommand::SaveMemorySetup`, limpa a pendência e entra em `SaveStatus::Verifying { expected, reads_left: 3 }`); `enum SaveStatus { Idle, Verifying { expected: SetupValues, reads_left: u8 }, Confirmed, NotConfirmed }` com `save_status()`; `pending_switch() -> Option<u8>`, `request_select_memory(n)`, `confirm_switch()`, `cancel_switch()` (campo `pending_switch: Option<u8>`); e `check_save_status(&mut self, update: &DPSUpdate)` (privado, chamado em `poll_events` antes do `merge`): só considera `Update` que traga `ovp` (leitura completa); igual dentro da tolerância → `Confirmed`; diferente → `reads_left -= 1` e, ao chegar a 0, `NotConfirmed`. Remover `save_profile`. Qualquer edição, troca de memória ou novo Salvar zera o `SaveStatus` para `Idle`
- [X] T022 [US2] Em `src/gui/widgets.rs`, completar `setup_field` (T013) com um `egui::DragValue` por campo: `.range(0.0..=field.max())`, `.clamp_existing_to_range(false)` (pesquisa R7), `.fixed_decimals(field.decimals())`, `.suffix(" <unidade>")`, `.speed` proporcional ao máximo (≈ 0,2 % do máximo); devolve `Some(novo valor)` quando `changed()`
- [X] T023 [US2] Em `src/gui/panels.rs`, tornar `setup_card` editável e completo: campos chamam `app.set_setup_field`; título mostra "• não salvo" (`WARN_COLOR`) com pendência; **Desfazer** (só com pendência) chama `discard_memory_setup_edits`; **Usar no setpoint** chama `use_memory_in_setpoint(selected_profile)` (dica: "Carrega Vset/Iset no card Setpoint; confirme com Aplicar"); linha de estado da gravação ("verificando…" cinza, "gravado" `OK_COLOR`, "não confirmado" vermelho com dica "A fonte não devolveu os valores enviados"); aviso de troca quando `pending_switch()` é `Some(n)`: "Descartar as alterações de M<atual>?" com **Descartar** (`confirm_switch`) e **Continuar editando** (`cancel_switch`). O clique em M1–M6 passa a chamar `request_select_memory(n)`
- [X] T024 [US2] Em `src/gui/panels.rs`, na linha das memórias do `setpoint_card`, ligar **Salvar** a `app.save_memory_setup()`: habilitado só quando conectado e `memory_setup().validate(selected_profile).is_ok()`; dica "Grava os seis valores do quadro em M<n>" (com o primeiro erro quando desabilitado por validação)
- [X] T025 [P] [US2] Atualizar a documentação das funções: `docs/modulos/logics-app.md` (`set_setup_field`, `discard_memory_setup_edits`, `save_memory_setup`, `save_status`, `pending_switch`, `request_select_memory`, `confirm_switch`, `cancel_switch`, `check_save_status`; remover `save_profile`; o tipo `SaveStatus` na seção de tipos; exemplos de `save_memory_setup` e `check_save_status`), `docs/modulos/logics-session.md` (`AppCommand::SaveMemorySetup`, ordem dos pacotes, `session_main`), `docs/modulos/gui-panels.md` e `docs/modulos/gui-widgets.md` (edição e `setup_card`), e a frase sobre `SaveProfile` em `docs/modulos/logics-protocol.md` (linha ~126)
- [X] T026 [US2] Em `README.md`: descrever edição, "• não salvo", Salvar (seis valores, quando fica habilitado), conferência ("verificando…/gravado/não confirmado"), "Usar no setpoint", o aviso de descarte, que OVP/OCP/OPP/OTP valem para o aparelho todo e que a **escrita das proteções é experimental**; atualizar "Sem conexão, este botão, o **Aplicar** e o **Salvar** ficam desabilitados" e a limitação "Proteções" (agora aparecem e são editáveis; LVP continua só lido)
- [X] T027 [US2] Rodar `cargo fmt --check`, `cargo clippy --all-targets`, `cargo test`; `docs_coverage` DEVE passar

**Checkpoint**: US1 e US2 funcionam juntas; o roteiro manual (T037) pode começar.

---

## Phase 5: User Story 3 - Não aceitar valores perigosos ou sem sentido (Priority: P2)

**Goal**: valores acima do máximo, cruzados (Vset > OVP, Iset > OCP), vazios ou não numéricos são barrados
com mensagem clara; o card Setpoint usa as mesmas faixas.

**Independent Test**: digitar Vset 36,50 e OVP 36,20 e ver os campos em vermelho e **Salvar** desabilitado;
digitar Vset 12 com OVP 10 e ver o erro cruzado; o card Setpoint aceita 36,00 V e 8,200 A.

### Tests for User Story 3

- [X] T028 [US3] Em `src/logics/app.rs` (testes), escrever: (a) `save_memory_setup()` com Vset 36.50 devolve `AboveMax(Vset)` e não envia; (b) com os seis máximos exatos envia (SC-007); (c) `apply_setpoints` com `vset_input = 36.0` e `cset_input = 8.2` envia `SetVoltage(36.0)` e `SetCurrent(8.2)` (FR-014); (d) constantes: `VSET_MAX == 36.0` e `ISET_MAX == 8.2` são as usadas pelo card (teste em `memory_setup` de que `SetupField::Vset.max() == VSET_MAX`)

### Implementation for User Story 3

- [X] T029 [US3] Em `src/gui/panels.rs`/`src/gui/widgets.rs`: marcar em vermelho (`Color32::RED`) o texto do campo cujo `SetupError::field()` bate com ele (calculado uma vez por quadro com `memory_setup().validate(...)`), e listar embaixo da grade as mensagens de `SetupError::message()` (no máximo as três primeiras, em vermelho, tamanho pequeno); sem erro, nenhuma linha extra
- [X] T030 [US3] Em `src/gui/panels.rs`, no `setpoint_card`, trocar `.range(0.0..=30.0)` e `.range(0.0..=5.0)` por `.range(0.0..=VSET_MAX)` e `.range(0.0..=ISET_MAX)` importados de `crate::logics::memory_setup` (FR-014); atualizar a dica de **Aplicar** se citar a faixa
- [X] T031 [P] [US3] Atualizar a documentação: `docs/modulos/gui-panels.md` (campos do setpoint com 0–36,00 V e 0–8,200 A, marcação de erro), `docs/modulos/gui-widgets.md` se `setup_field` mudar de assinatura, e `README.md`: linha 104 ("de 0 a 36 V … de 0 a 8,2 A"), remover a limitação "Faixa do setpoint … 30 V e 5 A" e acrescentar a tabela de máximos (FR-013) na seção de memórias
- [X] T032 [US3] Rodar `cargo fmt --check`, `cargo clippy --all-targets`, `cargo test`; `docs_coverage` DEVE passar

**Checkpoint**: as três histórias completas.

---

## Phase 6: Polish & Cross-Cutting

- [X] T033 [P] Atualizar `docs/arquitetura.md`: tabela de módulos (papel e "não faz" de `logics::memory_setup`), diagrama D1 (dependência `logics::app → logics::memory_setup` e `gui::panels → memory_setup`), D3 (a ação "Salvar" passa por `save_memory_setup` → `SaveMemorySetup` → `session_main` → porta), "Tipos centrais" (`SetupField`, `SetupValues`, `SaveStatus`, `AppCommand::SaveMemorySetup` no lugar de `SaveProfile`) e a regra "Nenhum comando é enviado sem ação explícita" (linha ~288: `save_memory_setup` no lugar de `save_profile`)
- [X] T034 [P] Emendar `.specify/memory/constitution.md` para a versão 1.0.2 (PATCH): relatório de impacto no topo; Princípio I sem a frase "O monitor exploratório `tests` está isento."; remover o débito "faixa de setpoint de 30 V / 5 A"; manter o débito do texto "Update" em inglês; `**Version**: 1.0.2`, `Last Amended` com a data do dia
- [X] T035 Varrer a documentação à procura da faixa antiga e de nomes removidos: `grep -rnE '30 V|5 A\b|de 0 a 30|de 0 a 5|save_profile|load_profile|SaveProfile' README.md docs/ .specify/memory/` NÃO DEVE achar a faixa do setpoint nem os nomes antigos (SC-008); corrigir o que sobrar
- [X] T036 Rodar os gates completos: `cargo fmt --check`, `cargo clippy --all-targets` (0 avisos), `cargo test` (lib + `docs_coverage`), `cargo build --release --bin controller`; registrar os resultados no "Registro da execução" do quickstart
- [ ] T037 **Manual (mantenedor, com a fonte)**: executar o roteiro de [quickstart.md](./quickstart.md) seção 2, passos 1–9, anotar o resultado de cada passo (principalmente o passo 4: a escrita das proteções funciona? e o 6: as proteções são por memória?). Se o passo 6 mostrar proteções por memória, abrir ajuste em `session_main` e no texto do rodapé; se a escrita falhar, manter "experimental" e registrar no README
- [ ] T038 Se a aparência mudou, atualizar a imagem `imagens/gui.png` (o README a mostra) — requer a fonte ou uma captura do mantenedor (opcional)

---

## Dependencies & Execution Order

- **Setup (Fase 1)** → **Foundational (Fase 2)** → histórias → **Polish**.
- **Fase 2 bloqueia todas**: T003 → T004 → T005 → T006 (mesmo arquivo, em sequência); T007 e T008 em paralelo com elas (outros arquivos; T008 depois de T006 para documentar o código final).
- **US1 (Fase 3)** depende da Fase 2. **US2 (Fase 4)** depende da US1 (usa `select_memory`, o quadro e `setup_field`). **US3 (Fase 5)** depende da US2 (a validação já existe, mas a marcação precisa dos campos editáveis; T030 pode rodar já depois da Fase 2).
- Dentro de cada história: testes (T010, T019, T028) antes do código; `app.rs` antes de `panels.rs`; documentação (T016, T025, T031) depois do código da fase.
- `docs_coverage` só passa com a documentação em dia: T009, T018, T027 e T032 fecham cada fase.

### Parallel opportunities

- Fase 2: T007 ‖ T003–T006 (arquivos diferentes). T008 ‖ T007.
- US1: T013 (widgets) ‖ T011/T012 (app); T016 ‖ T017 (docs diferentes).
- Polish: T033 ‖ T034.

## Implementation Strategy

- **MVP**: Fases 1–3 (US1): ver os seis valores. Não altera a fonte; risco baixo.
- **Entrega incremental**: US2 acrescenta a escrita (o único risco de hardware, **experimental**); US3 fecha
  a validação e a faixa do setpoint.
- **Antes do merge**: T037 (roteiro manual com a fonte) é obrigatório por mexer em comunicação
  (constituição, quality gate 5).
