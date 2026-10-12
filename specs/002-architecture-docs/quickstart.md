# Quickstart: validar a documentação de arquitetura

**Feature**: [spec.md](./spec.md) | **Plan**: [plan.md](./plan.md)

Este guia comprova, de ponta a ponta, que a feature cumpre a spec. Ele não descreve como escrever a
documentação; para isso, veja [contracts/doc-format.md](./contracts/doc-format.md).

## Pré-requisitos

- Rust 1.88 ou mais novo e `libudev` de desenvolvimento (`sudo dnf install systemd-devel`) para
  compilar o projeto real.
- O bloco `[[bin]] name = "tests"` removido do `Cargo.toml` (pré-requisito, pesquisa R9). Sem isso,
  `cargo test` não compila nenhum alvo.
- Não precisa da fonte conectada.

## 1. Gates automáticos

```bash
cargo fmt --check
cargo clippy --all-targets          # esperado: 0 avisos
cargo test                          # esperado: testes da biblioteca + docs_coverage passam
cargo build --release --bin controller
```

`cargo test` roda a conferência de cobertura (`tests/docs_coverage.rs`). Para rodar só ela:

```bash
cargo test --test docs_coverage
```

**Esperado** (SC-002, SC-005, SC-006, SC-008): `funcoes_documentadas`, `links_relativos_existem` e os
testes do scanner passam.

## 2. A conferência realmente detecta divergência (US3)

Prove que ela não passa em falso. Em cada caso, rode `cargo test --test docs_coverage`, confira a falha
e **desfaça a alteração**:

| Alteração temporária | Falha esperada |
|---|---|
| Renomear o título `### \`History::clear\`` em `docs/modulos/logics-history.md` | `faltando … History::clear` e `sobrando … History::clearr` |
| Adicionar `fn teste_temporario() {}` em `src/logics/format.rs` | `faltando em docs/modulos/logics-format.md: teste_temporario` |
| Apagar `docs/modulos/gui-chart.md` | `documento ausente: docs/modulos/gui-chart.md` |
| Trocar um link por `modulos/nao-existe.md` | `link quebrado em …` |

## 3. Verificações de estrutura

```bash
ls docs docs/modulos                              # FR-001, FR-003, FR-007
grep -n 'docs/README.md' README.md                # FR-002: pelo menos 2 ocorrências
grep -rnE 'tests\.rs|--bin tests' README.md docs/ # esperado: nenhuma ocorrência
grep -c '```mermaid' docs/arquitetura.md          # FR-005, FR-011, FR-012: esperado 6 (D1–D6)
git diff --stat -- src/ Cargo.lock                # SC-009: nenhuma linha em src/ (Cargo.lock inalterado)
```

## 4. Renderização dos diagramas (SC-007)

Não há renderizador de Mermaid local. Abra `docs/arquitetura.md` no visualizador do repositório (página
do GitHub do branch ou do PR) e confirme que os **6 diagramas aparecem desenhados**, sem a mensagem
"Unable to render" e sem texto cortado. Em caso de erro, o GitHub mostra a linha do problema no
próprio bloco.

## 5. Leitura guiada (SC-001, US1)

Entregue o `README.md` a alguém que não conhece o código e peça que, usando só a documentação, em até
10 minutos e sem abrir o código-fonte, responda:

1. Onde fica a regra que mantém o setpoint editado até clicar em "Aplicar"?
   *(esperado: `logics::app`, `AppState::sync_edit_buffers` / `SetpointEditor`)*
2. Onde fica o desenho do gráfico?
   *(esperado: `gui::chart`, `draw_history_chart`; os números das escalas vêm de `logics::scale`)*
3. Como um comando chega à fonte?
   *(esperado: `gui::panels` → `AppState::apply_setpoints` → canal → `session_main` → porta serial)*

## 6. Amostra de 10 funções (SC-003, US2)

Sorteie 10 funções de `src/` (por exemplo, `shuf -n 10` sobre a lista de `fn`) e confira que cada uma
tem, no documento do seu módulo, propósito, entradas, saída, efeitos e "Como funciona" preenchidos. As
funções da lista de regra não óbvia (pesquisa R3) precisam ter exemplo com valores.

## 7. Fechamento

- [ ] `docs/manutencao.md` explica quando atualizar e como rodar a conferência (FR-014).
- [ ] Nenhum arquivo em `src/` foi alterado (SC-009).

---

## Registro da execução (2026-10-12)

### Completude das fichas (T033, SC-003 e FR-009)

Conferência por `awk`/script sobre `docs/modulos/*.md`:

- **88 seções** (uma por função), **0 incompletas**: todas têm **Propósito**, **Entradas**, **Saída**,
  **Efeitos** e **Como funciona**.
- As **15 funções de regra não óbvia** têm `**Exemplo:**` na própria seção: `AxisScale::for_peak`,
  `decimals_for`, `DPS150::process_buffer`, `DPS150::parse_data`, `DPS150::build_command`,
  `History::record_if_due`, `OutputClock::tick`, `AppState::sync_edit_buffers`, `AppState::merge`,
  `AppState::refresh_ports_if_due`, `AppState::load_profile`, `session_main`, `format_window`,
  `draw_history_chart` e `GuiApp::ui`.

### Prova de detecção (T036, seção 2)

Cada alteração foi aplicada com backup, a conferência rodou (`cargo test --test docs_coverage`, saída com
código 101) e o arquivo foi restaurado em seguida:

| Alteração temporária | Mensagem obtida |
|---|---|
| Renomear o título `History::clear` para `History::clearr` | `faltando em docs/modulos/logics-history.md: History::clear` e `sobrando em docs/modulos/logics-history.md: History::clearr` |
| Acrescentar `fn teste_temporario() {}` a `src/logics/format.rs` | `faltando em docs/modulos/logics-format.md: teste_temporario` |
| Apagar `docs/modulos/gui-chart.md` | `documento ausente: docs/modulos/gui-chart.md (src/gui/chart.rs tem 2 funções)`, mais 3 `link quebrado` para o arquivo apagado |
| Trocar o link de `controller.md` por `modulos/nao-existe.md` em `docs/README.md` | `link quebrado em docs/README.md:22 → modulos/nao-existe.md` |

Depois de restaurar: `git status --short src` vazio, 11 documentos em `docs/modulos/` e `cargo test --test
docs_coverage` com 19 testes passando.

### Estrutura (T038) e amostra de 10 funções (T041)

- `ls docs docs/modulos`: índice, visão geral, manutenção e 11 documentos de módulo.
- `grep -n 'docs/README.md' README.md`: 3 ocorrências (depois da introdução, no Sumário e em "Estrutura do código").
- Referências a `tests.rs` ou `--bin tests` em `README.md` e `docs/`: nenhuma.
- Blocos ` ```mermaid ` em `docs/arquitetura.md`: 6 (D1–D6).
- `git diff --stat -- src/ Cargo.lock`: vazio (**SC-009**). Mudaram só `Cargo.toml`, `README.md`, `docs/`,
  `tests/docs_coverage.rs` e `specs/002-architecture-docs/`.
- **Amostra de 10 funções** (sorteio com semente fixa): `AppState::send`, `chart_panel`, `top_bar`, `main`,
  `GuiApp::ui`, `DPS150::get_protection`, `History::last`, `spawn_session`, `readouts` e
  `AppState::selected_port`. Cada entrada foi lida contra o código da função e o descreve corretamente
  (por exemplo, `get_protection` é descrita como não usada pela interface, e de fato só aparece no
  próprio protocolo).
