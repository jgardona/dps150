---

description: "Task list for feature 004-noto-sans-fonts"
---

# Tasks: Fonte Noto Sans e rótulos legíveis nas memórias

**Input**: Design documents from `specs/004-noto-sans-fonts/`

**Prerequisites**: [plan.md](./plan.md), [spec.md](./spec.md), [research.md](./research.md),
[data-model.md](./data-model.md), [contracts/ui-and-assets.md](./contracts/ui-and-assets.md),
[quickstart.md](./quickstart.md)

**Tests**: INCLUÍDOS onde dá para testar sem janela (definições de fonte e bytes embutidos). O que exige
olhos (nenhum quadradinho, nenhum corte, legibilidade) tem roteiro visual manual (T021).

**Organization**: tarefas agrupadas por user story. Caminhos relativos à raiz do repositório.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: pode rodar em paralelo (arquivos diferentes, sem dependência de tarefa incompleta)
- **[Story]**: US1 (quadro de setup em 16 px), US2 (toda a interface em Noto Sans), US3 (fonte no projeto e
  no executável, sem depender do sistema)

## Convenções desta feature

- **`docs_coverage` exige um título para cada função de `src/`**: a documentação de cada função nova é
  escrita na mesma fase do código, e ao fim de cada fase `cargo test --test docs_coverage` DEVE passar.
  Formato das entradas: [docs/manutencao.md](../../docs/manutencao.md). O slug do módulo `src/gui/fonts.rs`
  é `docs/modulos/gui-fonts.md`.
- **Nada em `src/logics/` muda.** Só `src/gui/`, `src/controller.rs`, `assets/`, `README.md`, `docs/`.
- Sem `unwrap`, `expect` ou `panic!` em código de produção (Princípio I); comentários em pt-BR com o porquê.
- O ambiente sem `libudev` roda o cargo por uma cópia de trabalho; no projeto real, use `cargo` direto.
- Tamanho do texto do quadro de setup: **16.0** (`SETUP_FONT_SIZE`). Os demais tamanhos NÃO mudam.

## Phase 1: Setup

- [X] T001 Rodar a linha de base e anotar em `specs/004-noto-sans-fonts/quickstart.md` (seção "Registro da execução"): `cargo fmt --check`, `cargo clippy --all-targets`, `cargo test` passando e o **tamanho do binário** de `cargo build --release --bin controller` (`ls -l target/release/controller`), para comparar depois (SC-006)

---

## Phase 2: Foundational (bloqueia US2 e US3)

- [X] T002 Criar `assets/fonts/` e copiar a fonte: `cp "/usr/share/fonts/google-noto-vf/NotoSans[wght].ttf" assets/fonts/NotoSans-VariableFont_wght.ttf` (712460 bytes, Noto Sans 2.015, eixo `wght` 400–700 com padrão 400); baixar a licença com `curl -fsSL https://raw.githubusercontent.com/notofonts/latin-greek-cyrillic/main/OFL.txt -o assets/fonts/OFL.txt` e conferir que o texto começa com "Copyright" e contém "SIL OPEN FONT LICENSE Version 1.1". Conferir que `assets/` não está no `.gitignore`

**Checkpoint**: a fonte e a licença estão no repositório.

---

## Phase 3: User Story 1 - Ler os campos das memórias sem esforço (Priority: P1) 🎯 MVP

**Goal**: título, rótulos, valores e unidades do quadro de setup com 16 px, sem corte nem sobreposição.

**Independent Test**: abrir o programa e conferir o quadro de setup: tudo em 16 px, com os seis máximos
(36,00 V, 8,200 A, 36,10 V, 8,200 A, 295,20 W, 99 °C) inteiros; botão de saída e barra inferior visíveis.

- [X] T003 [US1] Em `src/gui/theme.rs`, acrescentar `pub(crate) const SETUP_FONT_SIZE: f32 = 16.0;` com comentário em pt-BR ("Tamanho do texto do quadro de setup das memórias (título, rótulos, valores e unidades)")
- [X] T004 [US1] Em `src/gui/widgets.rs`, na função `setup_field`: trocar `.small()` do rótulo por `.size(SETUP_FONT_SIZE)` (mantendo as cores: erro vermelho, `accent` com `.strong()`, sem cor `.weak()`); e desenhar o valor (`DragValue`) dentro de `ui.scope(|ui| { ui.style_mut().override_font_id = Some(egui::FontId::proportional(SETUP_FONT_SIZE)); … })`, para o valor e a unidade ficarem em 16 px sem mudar o estilo global (pesquisa R6). O "—" de campo sem valor também usa 16 px. Importar `SETUP_FONT_SIZE` de `super::theme`
- [X] T005 [US1] Em `src/gui/panels.rs`, na função `setup_card`: o título "Setup Mn" usa `.size(SETUP_FONT_SIZE)` (além de `.strong()`); botões, mensagens de erro, aviso de troca, estado da gravação e rodapé NÃO mudam
- [X] T006 [P] [US1] Atualizar a documentação: `docs/modulos/gui-widgets.md` (`setup_field`: tamanho de 16 px, `override_font_id` no valor), `docs/modulos/gui-panels.md` (`setup_card`, passo 1: título em 16 px) e a tabela de constantes de `theme` em `docs/modulos/gui-mod.md` (`SETUP_FONT_SIZE` = 16)
- [X] T007 [P] [US1] Em `README.md`, na seção "Memórias M1–M6": dizer que o quadro de setup usa texto de 16 px, maior que o resto da interface, para leitura à distância
- [X] T008 [US1] Rodar `cargo fmt --check`, `cargo clippy --all-targets`, `cargo test` e `cargo test --test docs_coverage`; tudo DEVE passar. Conferência visual do quadro: T021 (itens 3 a 6 e 8)

**Checkpoint**: US1 entregue; o quadro já fica legível mesmo com a fonte antiga.

---

## Phase 4: User Story 2 - Toda a interface na fonte Noto Sans (Priority: P1)

**Goal**: todos os textos usam a Noto Sans, com as fontes de reserva do egui atrás, sem quadradinhos.

**Independent Test**: abrir o programa e conferir região por região que a fonte é a Noto Sans e que nenhum
caractere vira quadradinho.

### Tests for User Story 2

- [X] T009 [US2] Em `src/gui/fonts.rs` (novo, só o módulo de testes e os esqueletos das funções vazios o suficiente para compilar), escrever testes que falham antes do código: (a) `definitions().font_data` tem `"noto-sans"`; (b) `"noto-sans"` é a **primeira** fonte de `FontFamily::Proportional`; (c) as fontes do egui (`FontDefinitions::default()`) continuam em `Proportional`, na mesma ordem, depois da Noto Sans (FR-008); (d) `FontFamily::Monospace` é igual ao do `FontDefinitions::default()`; (e) os bytes embutidos começam com `[0x00, 0x01, 0x00, 0x00]` (TrueType) e têm mais de 100 KB; (f) os bytes embutidos são exatamente os de `assets/fonts/NotoSans-VariableFont_wght.ttf` (comparar com `std::fs::read` no teste, usando `env!("CARGO_MANIFEST_DIR")`; um teste pode usar `unwrap`/`expect`)

### Implementation for User Story 2

- [X] T010 [US2] Em `src/gui/fonts.rs`, implementar: `const NOTO_SANS_NAME: &str = "noto-sans";`, `const NOTO_SANS_BYTES: &[u8] = include_bytes!("../../assets/fonts/NotoSans-VariableFont_wght.ttf");`, `fn definitions() -> egui::FontDefinitions` (parte de `FontDefinitions::default()`, insere `NOTO_SANS_NAME` em `font_data` com `Arc::new(FontData::from_static(NOTO_SANS_BYTES))` e na posição **0** de `FontFamily::Proportional`; não toca em `Monospace`) e `pub fn install_fonts(ctx: &egui::Context)` (chama `ctx.set_fonts(definitions())`). Comentários em pt-BR: por que a Noto Sans fica na frente e as de reserva ficam atrás (pesquisa R4), por que é embutida (R3) e que o peso é o padrão 400 do eixo `wght` (R1). Sem `unwrap`/`expect`
- [X] T011 [US2] Em `src/gui/mod.rs`, declarar `mod fonts;` (ordem alfabética, depois de `chart`) e `pub use fonts::install_fonts;`. Em `src/controller.rs`, trocar `Box::new(|_cc| Ok(Box::new(GuiApp::new())))` por um construtor que chama `dps150::gui::install_fonts(&cc.egui_ctx)` **antes** de criar o `GuiApp` (a fonte precisa estar instalada antes do primeiro quadro)
- [X] T012 [US2] Documentação das funções e dos módulos: criar `docs/modulos/gui-fonts.md` no formato de `docs/manutencao.md` (Papel, O que não faz, Constantes com `NOTO_SANS_NAME` e `NOTO_SANS_BYTES`, uma entrada para `definitions` (interna, com `**Exemplo:**` mostrando a ordem de `Proportional`) e uma para `install_fonts`, Testes); em `docs/modulos/controller.md`, atualizar `main` (agora instala as fontes ao criar a janela); em `docs/modulos/gui-mod.md`, citar `fonts` na lista de submódulos; incluir `gui-fonts.md` em `docs/README.md` (tabela "Módulos") e, em `docs/arquitetura.md`, na tabela de módulos, na lista de documentação das funções e no diagrama de dependências (aresta `GMOD --> FONTS`; o binário continua ligado só a `gui`)
- [X] T013 [US2] Em `README.md`: na seção de uso/aparência, dizer que a interface usa a fonte Noto Sans
- [X] T014 [US2] Rodar `cargo fmt --check`, `cargo clippy --all-targets`, `cargo test` e `cargo test --test docs_coverage`; tudo DEVE passar (os testes de T009 agora passam)

**Checkpoint**: US1 e US2 funcionam juntas; falta provar a independência do sistema (US3) e a conferência visual.

---

## Phase 5: User Story 3 - Compilar e rodar sem depender da fonte instalada no sistema (Priority: P1)

**Goal**: a fonte e a licença vivem no repositório e a fonte vai dentro do executável.

**Independent Test**: compilar e executar com o ambiente de fontes do sistema vazio e ver a Noto Sans.

- [X] T015 [US3] Provar que a fonte está dentro do binário: `cargo build --release --bin controller` e conferir que o binário contém o nome da fonte (`grep -c "Noto Sans" target/release/controller` maior que 0) e que o tamanho cresceu cerca de 0,7 MB em relação à linha de base de T001; anotar os dois tamanhos em `specs/004-noto-sans-fonts/quickstart.md`
- [X] T016 [P] [US3] Em `README.md`, criar a seção **"Fonte e licença"**: nome (Noto Sans), versão (2.015), origem (`notofonts/latin-greek-cyrillic`), licença (SIL Open Font License 1.1) com link para `assets/fonts/OFL.txt`, e a frase de que a fonte é embutida no executável (não depende de fonte instalada); acrescentar `assets/` à seção "Estrutura do código" e ao Sumário
- [X] T017 [P] [US3] Conferir os links relativos do README e de `docs/` para `assets/fonts/OFL.txt` e para `docs/modulos/gui-fonts.md`: `cargo test --test docs_coverage` (`links_relativos_existem`) DEVE passar
- [ ] T018 [US3] **Manual (mantenedor, com tela)**: executar `FONTCONFIG_FILE=/dev/null ./target/release/controller` e conferir que a interface continua em Noto Sans; copiar só o binário para outra pasta, executar e conferir a mesma aparência (quickstart, seção 2)

**Checkpoint**: as três histórias completas.

---

## Phase 6: Polish & Cross-Cutting

- [X] T019 [P] Varrer a documentação: `grep -rniE 'ubuntu|fonte padrão|\.small\(\)' README.md docs/` NÃO DEVE deixar texto que descreva a fonte antiga ou o tamanho antigo do quadro de setup; corrigir o que sobrar (por exemplo, `docs/modulos/gui-widgets.md` e `gui-panels.md`)
- [X] T020 Rodar os gates completos: `cargo fmt --check`, `cargo clippy --all-targets` (0 avisos), `cargo test` (lib + `docs_coverage`), `cargo build --release --bin controller`; registrar os resultados e os tamanhos no "Registro da execução" do quickstart
- [ ] T021 **Manual (mantenedor, com tela)**: executar o roteiro visual de [quickstart.md](./quickstart.md) seção 3 (itens 1 a 8): toda a interface em Noto Sans, nenhum quadradinho nas mensagens com acento, os seis máximos inteiros no quadro de setup, largura mínima da coluna (330 px), 16 px, botão de saída e barra inferior visíveis na janela de 700 px, legibilidade a 50 cm, escalas do gráfico sem se encostar, campo com erro legível. Se algo não couber, aumentar `HEIGHT` em `src/controller.rs` (e atualizar `docs/modulos/controller.md`) ou ajustar o espaçamento, e registrar a decisão
- [ ] T022 **Manual (mantenedor, com tela)**: medir o tempo de abertura da janela antes e depois (quickstart, seção 4; SC-006: aumento de no máximo 0,5 s) e anotar no registro da execução
- [ ] T023 Se a aparência mudou de forma visível, atualizar `imagens/thumbnail.png` e `imagens/gui.png` com uma captura nova (opcional; depende de uma captura do mantenedor)

---

## Dependencies & Execution Order

- **Setup (T001)** → **Foundational (T002)** → histórias → **Polish**.
- **US1** (T003–T008) só toca `theme.rs`, `widgets.rs`, `panels.rs` e documentos: não depende da fonte e pode ser feita **antes ou em paralelo** com a US2 (mas T006 e T012 editam `gui-mod.md`: em sequência).
- **US2** (T009–T014) depende de T002 (o `include_bytes!` precisa do arquivo). Dentro dela: T009 (testes) → T010 → T011 → T012/T013 → T014.
- **US3** (T015–T018) depende da US2 (o binário só contém a fonte depois de T010/T011). T016 e T017 podem rodar assim que T002 existir, mas T016 e T013 editam o `README.md`: em sequência.
- **Polish**: T019–T020 depois das três histórias; T021–T022 exigem tela e ficam com o mantenedor.

### Parallel opportunities

- US1: T006 ‖ T007 (documentos diferentes), depois de T003–T005.
- US3: T016 ‖ T017 depois de T015.
- Polish: T019 antes de T020 (os gates precisam rodar com a documentação já corrigida); não há paralelo.

## Implementation Strategy

- **MVP**: Fases 1–3 (US1): o quadro de setup em 16 px. É o que o print mostra como problema e não depende da fonte.
- **Entrega incremental**: US2 troca a fonte de toda a interface; US3 comprova que ela vem dentro do executável e documenta a licença.
- **Antes do merge**: T021 e T022 (conferência visual e tempo de abertura) são do mantenedor, porque este ambiente não abre a janela. O que não for verificado deve ser dito como tal no relatório.
