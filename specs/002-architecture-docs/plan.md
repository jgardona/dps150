# Implementation Plan: Documentação de arquitetura do software

**Branch**: `002-architecture-docs` | **Date**: 2026-10-12 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `specs/002-architecture-docs/spec.md`

## Summary

Criar a pasta `docs/` com a documentação de arquitetura em Markdown, ligada ao `README.md`:

- **Visão geral** (`docs/arquitetura.md`): lista de módulos com papel e limites, e 6 diagramas Mermaid
  (dependências, fluxo de dados, ação do usuário até a fonte, ciclo de vida da sessão, estado da
  conexão, ciclo por quadro).
- **Um documento por módulo** (`docs/modulos/<slug>.md`): as 88 funções de produção do código, cada
  uma com propósito, entradas, saída, efeitos e "como funciona" (com exemplo nas de regra não óbvia).
- **Manutenção** (`docs/manutencao.md`) e uma **conferência automática** (`tests/docs_coverage.rs`, um
  teste de integração em Rust puro) que compara funções do código com títulos da documentação e
  confere os links, rodando dentro do `cargo test` que o projeto já usa.

Nenhum código de produção muda. Dois ajustes de apoio são necessários e estão descritos na pesquisa:
remover do `Cargo.toml` o binário `tests` (cujo arquivo foi apagado e que hoje impede o `cargo test`) e
corrigir no README as referências a esse binário.

## Technical Context

**Language/Version**: Rust, edição 2024, rustc 1.88 ou mais novo (a conferência usa só a biblioteca
padrão); documentação em Markdown com diagramas Mermaid

**Primary Dependencies**: nenhuma nova. `eframe`/`egui` e `serialport` seguem como estão, sem uso
pela conferência

**Storage**: N/A (arquivos `.md` versionados no repositório)

**Testing**: `cargo test` (teste de integração `tests/docs_coverage.rs`: cobertura de funções, links e
testes do scanner) + verificação visual dos diagramas no visualizador do repositório + leitura
guiada (SC-001)

**Target Platform**: repositório no GitHub; leitura em navegador e no editor; Linux para rodar a
conferência

**Project Type**: documentação de um app desktop (GUI) já existente

**Performance Goals**: conferência em menos de 1 s; nenhum efeito no controlador em execução

**Constraints**: nenhum código de produção alterado (FR-017); diagramas em texto (FR-010); pt-BR
(FR-016); sem dependência nova; sem fonte conectada para validar

**Scale/Scope**: 11 arquivos-fonte com funções, 88 funções, 6 diagramas, 14 documentos novos
(1 índice + 1 visão geral + 1 de manutenção + 11 de módulo), mais o README ajustado e o teste

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Princípio | Gate | Pré-pesquisa | Pós-design |
|---|---|---|---|
| I. Qualidade | `fmt`/`clippy` sem avisos | ⚠️ `cargo clippy --all-targets` e `cargo test` não compilam hoje (`src/tests.rs` removido, `Cargo.toml` ainda o declara) | ✅ Pré-requisito R9 remove o bloco; o novo teste nasce limpo de `fmt` e `clippy` |
| I. Qualidade | Comentários e mensagens em pt-BR, explicando o porquê | ✅ | ✅ A documentação e as mensagens da conferência seguem pt-BR (R6) |
| I. Qualidade | Código de produção sem `unwrap`/`panic!` | ✅ Não alterado | ✅ O teste usa `panic!`/`assert!` como qualquer teste; nada em `src/` muda |
| II. Testes | Lógica nova com teste | — | ✅ O scanner de funções é uma função pura com testes de casos fixos (R4) |
| II. Testes | Comportamento do protocolo coberto | ✅ Feito na 001 | ✅ Inalterado |
| III. UX | README atualizado em feature visível | ⚠️ README cita o monitor removido (links quebrados) | ✅ README ganha o link para `docs/` e perde as referências ao monitor (R7) |
| IV. Desempenho | Sem regressão em tempo de execução | ✅ | ✅ Nada de runtime muda; a conferência lê arquivos em menos de 1 s |
| Restrições | Dependência nova justificada | ✅ | ✅ Nenhuma. Python/Node descartados em favor de teste em Rust puro (R4) |
| Fluxo | Gates 1–4 antes de integrar | ⚠️ Gates 3 e 4 hoje falham pelo `Cargo.toml` | ✅ Passam depois do pré-requisito R9 |
| Fluxo | Branch `feature/<nome>`, roteiro manual quando há hardware | ⚠️ Branch atual `doc/documentacao-componentes` | ✅ Sem hardware envolvido; o nome do branch é escolha do mantenedor |

**Resultado**: aprovado. Os ⚠️ pré-pesquisa são resíduos do commit `8ecd93f` e do README, e o plano os
corrige. Nenhuma violação precisa entrar em Complexity Tracking.

## Project Structure

### Documentation (this feature)

```text
specs/002-architecture-docs/
├── spec.md
├── plan.md              # Este arquivo
├── research.md          # Fase 0 — fatos medidos e decisões R0–R9
├── data-model.md        # Fase 1 — entidades, regras e mapa módulo → documento → funções
├── quickstart.md        # Fase 1 — roteiro de validação
├── contracts/
│   └── doc-format.md    # Fase 1 — layout, formato da entrada e contrato da conferência
├── checklists/
│   └── requirements.md
└── tasks.md             # Fase 2 (/speckit-tasks — ainda não criado)
```

### Source Code (repository root)

```text
README.md                         # ← ajustado: link para docs/, sem referências ao monitor `tests`
Cargo.toml                        # ← pré-requisito: remove o [[bin]] "tests" (arquivo já apagado)
docs/                             # NOVO
├── README.md                     # índice
├── arquitetura.md                # módulos + diagramas D1–D6 + tipos centrais
├── manutencao.md                 # quando atualizar e como conferir
└── modulos/
    ├── controller.md
    ├── gui-mod.md   gui-panels.md   gui-chart.md   gui-widgets.md
    └── logics-app.md  logics-protocol.md  logics-session.md
        logics-history.md  logics-scale.md  logics-format.md
tests/
└── docs_coverage.rs              # NOVO: cobertura de funções + links + testes do scanner
src/                              # inalterado
```

**Structure Decision**: documentação em `docs/` na raiz, espelhando `src/` em `docs/modulos/` (um
documento por arquivo-fonte com funções), e a conferência como teste de integração em `tests/`, fora do
build do controlador. Os módulos sem funções (`lib.rs`, `logics/mod.rs`, `gui/theme.rs`) ficam só na
visão geral.

## Ordem de implementação recomendada

1. **Pré-requisito**: remover o bloco `[[bin]] name = "tests"` do `Cargo.toml` e confirmar que
   `cargo check --all-targets` volta a funcionar.
2. **Conferência primeiro** (`tests/docs_coverage.rs`): escrever o scanner com seus testes de casos
   fixos; ver `funcoes_documentadas` falhar listando as 88 funções faltando. Essa lista é o guia da
   escrita.
3. **Esqueleto**: `docs/README.md` (índice) e `docs/manutencao.md`.
4. **US1 — visão geral**: `docs/arquitetura.md` com a lista de módulos e os diagramas D1–D6, e os
   links no README (MVP: já dá para entender a arquitetura).
5. **US2 — módulos**, do núcleo para a borda: `logics-scale`, `logics-format`, `logics-history`,
   `logics-protocol`, `logics-session`, `logics-app`, depois `gui-widgets`, `gui-chart`, `gui-panels`,
   `gui-mod` e `controller`. A cada documento, rodar a conferência e ver a lista de pendências cair.
6. **README**: remover as referências ao monitor e resumir a seção "Estrutura do código" apontando
   para `docs/arquitetura.md`.
7. **Fechamento**: rodar o [quickstart](./quickstart.md) inteiro (gates, prova de detecção, diagramas
   no visualizador, leitura guiada, amostra de funções).

## Complexity Tracking

Nenhuma violação da constituição precisa de justificativa.

## Melhorias futuras (fora do escopo)

- Emenda PATCH da constituição (1.0.2): o Princípio I cita o "monitor exploratório `tests`", que não
  existe mais.
- Exigir a conferência de documentação no gate de integração do repositório (por exemplo, em CI), se
  um dia houver CI.
- Comentários `///` no código e `cargo doc`, complementares ao que esta feature escreve em Markdown.
