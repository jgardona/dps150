---

description: "Task list for feature 006-output-button-style"
---

# Tasks: Estilo do botão de saída

**Input**: Design documents from `specs/006-output-button-style/`

**Prerequisites**: [plan.md](./plan.md), [spec.md](./spec.md), [research.md](./research.md),
[data-model.md](./data-model.md), [contracts/output-button-style.md](./contracts/output-button-style.md),
[quickstart.md](./quickstart.md)

**Tests**: INCLUÍDOS onde dá para testar sem janela: o raio do retângulo desenhado e a folga de baixo do botão.
O que exige olhos (estados ligada e mouse por cima, margem real contra a barra inferior) tem roteiro visual
manual (T013).

**Organization**: tarefas agrupadas por user story. Caminhos relativos à raiz do repositório.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: pode rodar em paralelo (arquivos diferentes, sem dependência de tarefa incompleta)
- **[Story]**: US1 (bordas arredondadas), US2 (margem inferior de 10 px)

## Convenções desta feature

- **Só muda `src/gui/panels.rs` (a função `output_button` e o trecho que a chama em `control_column`),
  `src/gui/theme.rs` e a documentação.** Nada em `src/logics/` muda, e nenhuma função nova entra em `src/` (o
  `docs_coverage` não pede entradas novas; a de `output_button` é **alterada**).
- Nada muda no comportamento: texto, cores, dica, habilitação, altura (56 px) e largura (toda a coluna) do botão
  ficam como estão; `toggle_output` e `output_on` não são tocados (FR-004 a FR-006).
- As tarefas T004, T008 e as de documentação mexem na mesma função e nos mesmos documentos: ficam **em
  sequência**.
- Sem `unwrap`, `expect` ou `panic!` em código de produção (Princípio I); comentários em pt-BR com o porquê.
  Testes podem usar `unwrap`/`expect`.

## Phase 1: Setup

- [X] T001 Rodar a linha de base e anotar em `specs/006-output-button-style/quickstart.md` (seção "Registro da execução"): `cargo fmt --check`, `cargo clippy --all-targets` e `cargo test` passando, para comparar depois

---

## Phase 2: Foundational (bloqueia as histórias)

- [X] T002 Em `src/gui/theme.rs`, acrescentar `pub(crate) const OUTPUT_BUTTON_RADIUS: u8 = 5;` ("Raio dos cantos do botão de saída, em pixels lógicos") e `pub(crate) const OUTPUT_BOTTOM_MARGIN: f32 = 10.0;` ("Folga entre a borda de baixo do botão de saída e a barra inferior"), com comentários em pt-BR

**Checkpoint**: as constantes existem.

---

## Phase 3: User Story 1 - Bordas arredondadas no botão de saída (Priority: P1) 🎯 MVP

**Goal**: os quatro cantos do botão de saída com raio de 5 px em todos os estados.

**Independent Test**: abrir o programa e ver o botão (desabilitado, e conectado ligando e desligando) com os cantos arredondados do mesmo jeito.

### Tests for User Story 1

- [X] T003 [US1] Em `src/gui/panels.rs`, no módulo `#[cfg(test)] mod tests` já existente, escrever um teste que **falha antes do código** (o raio padrão do egui é menor que 5): rodar `output_button` com `egui::Context::default()` e `ctx.run_ui(...)` sobre um `AppState::new()` (desconectado, logo botão desabilitado e cinza), dentro de `ui.with_layout(egui::Layout::bottom_up(egui::Align::Min), ...)`, e procurar em `FullOutput::shapes` um `egui::epaint::Shape::Rect` com `corner_radius == egui::CornerRadius::same(OUTPUT_BUTTON_RADIUS)` e preenchimento igual ao do botão desabilitado (o `fill` de `Color32::from_gray(55)` com a atenuação do desabilitado pode mudar a cor: aceitar qualquer `Shape::Rect` preenchido com `corner_radius` de 5 e largura maior que 100 pontos, que é o botão de largura da coluna)

### Implementation for User Story 1

- [X] T004 [US1] Em `src/gui/panels.rs`, na função `output_button`: acrescentar `.corner_radius(OUTPUT_BUTTON_RADIUS)` ao `egui::Button` (depois do `.fill(fill)`), valendo para ligada, desligada e desabilitado, pois é uma só chamada antes de qualquer estado; importar `OUTPUT_BUTTON_RADIUS` de `super::theme`. Não mudar texto, cores, tamanho (`add_sized([w, 56.0], …)`), dica nem o clique
- [X] T005 [P] [US1] Atualizar a documentação: `docs/modulos/gui-panels.md` (entrada de `output_button`: o botão tem cantos com raio `OUTPUT_BUTTON_RADIUS` = 5 px em todos os estados) e a tabela de constantes de `theme` em `docs/modulos/gui-mod.md` (`OUTPUT_BUTTON_RADIUS` = 5 px)
- [X] T006 [US1] Rodar `cargo fmt --check`, `cargo clippy --all-targets`, `cargo test` e `cargo test --test docs_coverage`; tudo DEVE passar (inclusive o teste de T003)

**Checkpoint**: US1 entregue; o botão já tem os cantos arredondados, ainda encostado na barra.

---

## Phase 4: User Story 2 - Margem inferior de 10 px (Priority: P1)

**Goal**: 10 px livres entre a borda de baixo do botão e a barra inferior.

**Independent Test**: abrir o programa e ver o espaço de 10 px entre o botão e a barra inferior, na janela inicial, maximizada e reduzida.

### Tests for User Story 2

- [X] T007 [US2] Em `src/gui/panels.rs` (módulo de testes), escrever um teste que **falha antes do código**: rodar `output_button` dentro de `ui.with_layout(egui::Layout::bottom_up(egui::Align::Min), ...)` (duas passagens do `ctx.run_ui`, como nos outros testes) e exigir que `ui.max_rect().bottom() - resposta.rect.bottom()` seja `OUTPUT_BOTTOM_MARGIN` com tolerância de 0,5 px; exigir também que a altura da resposta continue 56 px (± 1 px, por causa da expansão de interação) e que a largura seja a disponível (SC-003). Para isso `output_button` precisa devolver a `egui::Response` (T008)

### Implementation for User Story 2

- [X] T008 [US2] Em `src/gui/panels.rs`, na função `output_button`: (a) mudar a assinatura para devolver `egui::Response` (a do botão, já com `.on_hover_text(hover)` e o tratamento do clique) em vez de nada; (b) no começo da função, antes de criar o botão, acrescentar `ui.add_space(OUTPUT_BOTTOM_MARGIN);` com comentário em pt-BR (a função é chamada num layout de baixo para cima, em que o primeiro item fica na base). Na implementação, o teste mostrou que descontar `item_spacing.y` dava 7 px; sem o desconto a folga é de 10 px; (c) em `control_column`, a chamada `output_button(ui, app);` dentro do `with_layout(bottom_up)` continua igual (a resposta devolvida pode ser ignorada com `let _ =` se o compilador reclamar). Importar `OUTPUT_BOTTOM_MARGIN` de `super::theme`
- [X] T009 [P] [US2] Atualizar a documentação: `docs/modulos/gui-panels.md` (entrada de `output_button`: **Saída** passa a ser a `Response` do botão; **Como funciona** descreve a margem de 10 px e a conta com `item_spacing.y`; na entrada de `control_column`, trocar "que fica assim encostado no fim da coluna" por "que fica a 10 px do fim da coluna, onde começa a barra inferior"; na seção de testes, os dois testes novos) e a tabela de constantes de `theme` em `docs/modulos/gui-mod.md` (`OUTPUT_BOTTOM_MARGIN` = 10 px)
- [X] T010 [P] [US2] Em `README.md`, na seção "Ligar e desligar a saída": dizer que o botão tem cantos arredondados e fica a uma pequena folga (10 px) da barra inferior
- [X] T011 [US2] Rodar `cargo fmt --check`, `cargo clippy --all-targets`, `cargo test` e `cargo test --test docs_coverage`; tudo DEVE passar (inclusive o teste de T007)

**Checkpoint**: as duas histórias completas.

---

## Phase 5: Polish & Cross-Cutting

- [X] T012 Rodar os gates completos: `cargo fmt --check`, `cargo clippy --all-targets` (0 avisos), `cargo test` (lib + `docs_coverage`), `cargo build --release --bin controller`; conferir por `git diff -- src/gui/panels.rs` que no botão mudaram só o raio, a margem e o retorno (texto, cores, dica, `add_enabled_ui`, `add_sized([w, 56.0], …)` e `toggle_output` iguais) e que `grep -rn "encostado" README.md docs/` não deixa texto da posição antiga; registrar tudo no "Registro da execução" do quickstart
- [ ] T013 **Manual (mantenedor, com tela e, para o passo 3, com a fonte)**: executar o roteiro visual de [quickstart.md](./quickstart.md) seções 2 e 3: cantos arredondados em todos os estados (desabilitado, desligada, ligada, mouse por cima, pressionado, sem cantos retos por baixo), 10 px entre o botão e a barra inferior com a janela inicial, maximizada e reduzida, o botão sem mudar de tamanho nem de posição ao ligar e desligar, o texto inteiro, e o botão sem encostar no quadro de setup. Se a folga visível contra a barra não parecer 10 px (a barra pode ter margem própria acima do texto), ajustar `OUTPUT_BOTTOM_MARGIN` e registrar a decisão
- [ ] T014 Se a aparência mudou de forma visível, atualizar `imagens/thumbnail.png` e `imagens/gui.png` com uma captura nova (opcional; depende de uma captura do mantenedor)

---

## Dependencies & Execution Order

- **Setup (T001)** → **Foundational (T002)** → US1 → US2 → **Polish**.
- **US1** (T003–T006): T003 (teste) → T004 (código) → T005 → T006.
- **US2** (T007–T011) depende da US1 (mesma função e mesmos documentos): T007 → T008 → T009 ‖ T010 → T011. T009 e T005 editam `docs/modulos/gui-panels.md` e `gui-mod.md`: **em sequência**, nunca juntos. T009 ‖ T010 só entre si (documentos diferentes).
- **Polish**: T012 depois das duas histórias; T013 e T014 exigem tela.

### Parallel opportunities

- US2: T009 ‖ T010 (documentos diferentes), depois de T008.
- Fora isso, tudo é sequencial (uma função, dois documentos compartilhados).

## Implementation Strategy

- **MVP**: Fases 1–3 (US1): o botão com cantos arredondados.
- **Entrega incremental**: US2 acrescenta a margem de 10 px e o teste que a mede; T012 prova que o comportamento do botão não mudou.
- **Antes do merge**: T013 (conferência visual) é do mantenedor, porque este ambiente não abre a janela. O que não for verificado deve ser dito como tal no relatório.
