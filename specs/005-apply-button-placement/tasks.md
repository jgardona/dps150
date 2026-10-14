---

description: "Task list for feature 005-apply-button-placement"
---

# Tasks: Ajuste do botão Aplicar na seção Setpoint

**Input**: Design documents from `specs/005-apply-button-placement/`

**Prerequisites**: [plan.md](./plan.md), [spec.md](./spec.md), [research.md](./research.md),
[data-model.md](./data-model.md), [contracts/setpoint-layout.md](./contracts/setpoint-layout.md),
[quickstart.md](./quickstart.md)

**Tests**: INCLUÍDOS onde dá para testar sem janela: a largura das linhas do card (medida em Noto Sans de 14 px)
e o envio de comandos já coberto em `src/logics/app.rs`. O que exige olhos tem roteiro visual manual (T017).

**Organization**: tarefas agrupadas por user story. Caminhos relativos à raiz do repositório.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: pode rodar em paralelo (arquivos diferentes, sem dependência de tarefa incompleta)
- **[Story]**: US1 (Aplicar na linha dos campos), US2 (o botão se comporta como antes), US3 (Desfazer na linha do
  título)

## Convenções desta feature

- **Só muda `src/gui/panels.rs` (a função `setpoint_card`), `src/gui/theme.rs` e a documentação.** Nada em
  `src/logics/` muda, e nenhuma função nova entra em `src/` (então o `docs_coverage` não pede entradas novas; as
  entradas de `setpoint_card` são **alteradas**).
- As tarefas T004, T008, T011 e T012 mexem na mesma função e nos mesmos documentos: ficam **em sequência**.
- Nenhuma chamada a `AppState` muda: `apply_setpoints`, `discard_setpoint_edits`, `mark_setpoint_edited`,
  `set_editing`, `is_connected` e `setpoint().is_dirty()` continuam como estão (FR-005, FR-007).
- Textos, cores, dicas, limites e tamanhos NÃO mudam; só posição, espaçamento e alinhamento.
- Sem `unwrap`, `expect` ou `panic!` em código de produção (Princípio I); comentários em pt-BR com o porquê.

## Phase 1: Setup

- [X] T001 Rodar a linha de base e anotar em `specs/005-apply-button-placement/quickstart.md` (seção "Registro da execução"): `cargo fmt --check`, `cargo clippy --all-targets` e `cargo test` passando, para comparar depois

---

## Phase 2: Foundational (bloqueia as histórias)

- [X] T002 Em `src/gui/theme.rs`, acrescentar `pub(crate) const FIELD_GAP: f32 = 10.0;` com comentário em pt-BR ("Espaço extra entre os grupos da linha de campos do Setpoint: entre o campo V e o rótulo A, e entre o campo A e o botão Aplicar. Soma-se ao espaçamento padrão entre widgets (8 px de cada lado)"). É o valor que `setpoint_card` já usa em `ui.add_space(10.0)`

**Checkpoint**: a constante existe.

---

## Phase 3: User Story 1 - Aplicar na mesma linha dos campos de tensão e corrente (Priority: P1) 🎯 MVP

**Goal**: V, A e **Aplicar** na mesma linha, com o mesmo espaço entre os grupos, e nenhuma linha de botões
abaixo dos campos.

**Independent Test**: abrir o programa e ver V, A e **Aplicar** numa linha, com espaços iguais, cabendo na
largura mínima da coluna (330 px) com 36,00 V e 8,200 A.

### Tests for User Story 1

- [X] T003 [US1] Em `src/gui/panels.rs`, criar um módulo `#[cfg(test)] mod tests` com um auxiliar que mede o texto (contexto do egui com `install_fonts`, duas passagens, `layout_no_wrap` com `FontId::proportional(14.0)`, como em `src/gui/fonts.rs`) e um teste que **falha antes do código** só se a conta estourar: a linha "V" + campo "36.00 V" + `FIELD_GAP` + "A" + campo "8.200 A" + `FIELD_GAP` + botão "Aplicar", somando preenchimento de cada campo e do botão (2 × 4 px), o espaçamento padrão entre widgets (8 px, em cada junção, incluindo os dois lados do `FIELD_GAP`) e as margens do card (2 × 8 px), cabe em `CONTROL_MIN_WIDTH` (330 px). O teste pode usar `unwrap`/`expect`

### Implementation for User Story 1

- [X] T004 [US1] Em `src/gui/panels.rs`, na função `setpoint_card`: (a) trocar `ui.add_space(10.0)` entre o campo V e o rótulo A por `ui.add_space(FIELD_GAP)`; (b) depois do campo A, na **mesma** `ui.horizontal`, acrescentar `ui.add_space(FIELD_GAP)` e o botão **Aplicar** (o mesmo `egui::Button`, com o mesmo texto, `.color(Color32::BLACK)`, `.strong()`, `.fill(ACTION_COLOR)` com pendência ou `Color32::from_gray(90)` sem ela, `add_enabled(app.is_connected(), …)` e a dica "Envia Vset/Iset ao aparelho (ou Enter)"); (c) manter na mesma linha o `enter_pressed` e o `if clicked || (enter_pressed && app.is_connected()) { app.apply_setpoints(); }`; (d) remover a linha `ui.horizontal` que continha **Aplicar** e **Desfazer** (o **Desfazer** vai para a linha do título na T011; até lá, deixar a linha antiga só com o **Desfazer**). Importar `FIELD_GAP` de `super::theme`
- [X] T005 [P] [US1] Atualizar a documentação: `docs/modulos/gui-panels.md` (entrada de `setpoint_card`: **Campos** agora inclui o botão **Aplicar** à direita do campo A, com o mesmo espaço `FIELD_GAP`; ajustar o item **Botões**) e a tabela de constantes de `theme` em `docs/modulos/gui-mod.md` (`FIELD_GAP` = 10 px)
- [X] T006 [P] [US1] Em `README.md`, na seção "Setpoint": dizer que o botão **Aplicar** fica na mesma linha dos campos V e A, à direita do campo de corrente
- [X] T007 [US1] Rodar `cargo fmt --check`, `cargo clippy --all-targets`, `cargo test` e `cargo test --test docs_coverage`; tudo DEVE passar (inclusive o teste de T003)

**Checkpoint**: US1 entregue; o **Desfazer** ainda está na posição antiga, numa linha só dele, até a US3.

---

## Phase 4: User Story 2 - O botão continua se comportando como antes (Priority: P1)

**Goal**: provar que só a posição mudou.

**Independent Test**: com a fonte conectada, **Aplicar** e **Enter** enviam os valores; sem conexão o botão fica desabilitado; azul com pendência, cinza sem.

- [X] T008 [US2] Conferir por diff (`git diff -- src/gui/panels.rs`) que, na parte do botão **Aplicar**, só mudaram a posição e o espaçamento: texto, cores (`ACTION_COLOR` com pendência, `Color32::from_gray(90)` sem), `add_enabled(app.is_connected(), …)`, dica e a lógica `clicked || (enter_pressed && app.is_connected())` são idênticos aos de antes; anotar o resultado no "Registro da execução" do quickstart. Rodar `cargo test --lib aplicar` (os testes `aplicar_envia_os_valores_dos_campos` e `aplicar_envia_os_maximos_do_setpoint` de `src/logics/app.rs` DEVEM passar sem alteração)
- [ ] T009 [US2] **Manual (mantenedor, com a fonte)**: executar a seção 3 do [quickstart.md](./quickstart.md) (clicar em **Aplicar**, **Enter** num campo, dica, cor com e sem pendência, botão desabilitado sem conexão) e anotar o resultado

**Checkpoint**: o comportamento do botão está provado igual.

---

## Phase 5: User Story 3 - O botão Desfazer continua acessível (Priority: P2)

**Goal**: **Desfazer** na linha do título, depois de "• não aplicado", sem mudar a altura do card.

**Independent Test**: editar um campo e ver "• não aplicado" e **Desfazer** na linha do título, sem nada se mover; clicar em **Desfazer** volta os campos à fonte; ao sumir a pendência, nada se move.

### Tests for User Story 3

- [X] T010 [US3] Em `src/gui/panels.rs` (módulo de testes de T003), escrever um teste que mede a linha do título na pendência: "Setpoint" (strong) + "• não aplicado" + botão "Desfazer" (preenchimento de 2 × 4 px), com o espaçamento padrão de 8 px entre os três e as margens do card (2 × 8 px), cabe em `CONTROL_MIN_WIDTH` (330 px)

### Implementation for User Story 3

- [X] T011 [US3] Em `src/gui/panels.rs`, na função `setpoint_card`: dentro da `ui.horizontal` do título (a que mostra "Setpoint" e, com pendência, "• não aplicado"), acrescentar, **depois** do aviso e só com `app.setpoint().is_dirty()`, o botão `ui.button("Desfazer")` que chama `app.discard_setpoint_edits()` ao clique (mesmo texto e mesmo efeito de antes); remover a linha `ui.horizontal` antiga que só tinha o **Desfazer** (T004(d)). O card NÃO pode ganhar nem perder altura com a pendência: a linha do título já existe com ou sem ela
- [X] T012 [US3] Atualizar a documentação: `docs/modulos/gui-panels.md` (entrada de `setpoint_card`: **Título** com "• não aplicado" e o **Desfazer** ao lado, quando há pendência; remover do item **Botões** a menção ao **Desfazer** em linha própria) e `README.md`, seção "Setpoint" (o **Desfazer** aparece ao lado de "• não aplicado", no título do card)
- [X] T013 [US3] Rodar `cargo fmt --check`, `cargo clippy --all-targets`, `cargo test` e `cargo test --test docs_coverage`; tudo DEVE passar (inclusive o teste de T010)

**Checkpoint**: as três histórias completas.

---

## Phase 6: Polish & Cross-Cutting

- [X] T014 [P] Varrer a documentação: `grep -rniE 'linha própria|abaixo dos campos|Aplicar.*Desfazer' README.md docs/` NÃO DEVE deixar texto que descreva a posição antiga dos botões; corrigir o que sobrar (em especial `docs/modulos/gui-panels.md` e `docs/arquitetura.md`)
- [X] T015 Rodar os gates completos: `cargo fmt --check`, `cargo clippy --all-targets` (0 avisos), `cargo test` (lib + `docs_coverage`), `cargo build --release --bin controller`; registrar os resultados no "Registro da execução" do quickstart
- [ ] T016 **Manual (mantenedor, com tela)**: executar a seção 2 do [quickstart.md](./quickstart.md) (itens 1 a 7): V, A e **Aplicar** numa linha, espaços iguais, mesma altura, largura mínima da coluna com 36,00 V e 8,200 A, **Desfazer** no título sem nada se mover, botão desabilitado sem conexão. Se algo não couber ou ficar desalinhado, ajustar `FIELD_GAP` ou o alinhamento e registrar a decisão
- [ ] T017 Se a aparência mudou de forma visível, atualizar `imagens/thumbnail.png` e `imagens/gui.png` com uma captura nova (opcional; depende de uma captura do mantenedor)

---

## Dependencies & Execution Order

- **Setup (T001)** → **Foundational (T002)** → US1 → US2 e US3 → **Polish**.
- **US1** (T003–T007): T003 (teste) → T004 (código) → T005 ‖ T006 → T007.
- **US2** (T008–T009) depende da US1 (confere o código dela). T009 é manual e pode rodar a qualquer momento depois de T004.
- **US3** (T010–T013) depende da US1 (mesma função e T004(d)): T010 → T011 → T012 → T013. T012 e T005 editam `docs/modulos/gui-panels.md`, e T012 e T006 editam `README.md`: **em sequência**, nunca juntos.
- **Polish**: T014 antes de T015 (os gates rodam com a documentação corrigida); T016 e T017 exigem tela.

### Parallel opportunities

- US1: T005 ‖ T006 (documentos diferentes).
- Polish: nenhum paralelo útil.

## Implementation Strategy

- **MVP**: Fases 1–3 (US1): **Aplicar** na linha dos campos. É o pedido do usuário, e o **Desfazer** continua funcionando na posição antiga até a US3.
- **Entrega incremental**: US2 prova que nada mudou no comportamento; US3 move o **Desfazer** para o título e fecha a altura constante do card.
- **Antes do merge**: T009 e T016 (conferência com a fonte e visual) são do mantenedor, porque este ambiente não abre a janela. O que não for verificado deve ser dito como tal no relatório.
