# Implementation Plan: Campos de configuração das memórias

**Branch**: `feature/memorias-melhorias` | **Date**: 2026-10-12 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `specs/003-memory-setup-fields/spec.md`

## Summary

Acrescentar à coluna de controle um **quadro de setup** (Vset, Iset, OVP, OCP, OPP, OTP) no espaço vazio
entre a linha das memórias e o botão de saída. Selecionar M1–M6 mostra os valores da memória; os seis
campos são editáveis; **Salvar** valida e envia tudo à fonte. A abordagem:

- **Regra nova e pura** em `src/logics/memory_setup.rs`: os seis campos, seus máximos (FR-013), a edição
  pendente e a validação (faixa, Vset ≤ OVP, Iset ≤ OCP). Sem egui, testável sem hardware.
- **`AppState`** guarda o setup em edição, troca de memória sem enviar nada, envia um único comando novo
  (`AppCommand::SaveMemorySetup`) e confere a gravação lendo a fonte de volta (FR-011).
- **Sessão serial** executa o comando: grava OVP/OCP/OPP/OTP e Vset/Iset do grupo, e pede um `GET ALL`.
  O protocolo já sabe montar esses pacotes (`set_float_value`); só ganha testes de fixture.
- **GUI** (`panels.rs`) desenha o quadro e passa os máximos novos ao card "Setpoint" (FR-014).
- **Documentação** (FR-015): README, `docs/` (um documento novo, vários alterados) e a constituição.

## Technical Context

**Language/Version**: Rust, edição 2024, rustc 1.88 ou mais novo

**Primary Dependencies**: nenhuma nova (`eframe`/`egui` 0.34, `serialport` 4.9)

**Storage**: N/A (o estado vive na fonte; nada é gravado em disco)

**Testing**: `cargo test` (testes unitários de `memory_setup`, `app` e `protocol`, mais `docs_coverage`)
+ roteiro manual com a fonte conectada (escrita das proteções é experimental)

**Target Platform**: Linux (desktop)

**Project Type**: aplicativo desktop (biblioteca `dps150` + binário `controller`)

**Performance Goals**: sem trabalho novo por quadro além de desenhar seis campos; nenhum acesso serial
na thread da GUI

**Constraints**: Princípios I–IV da constituição; `logics` sem dependência de egui; nenhum envio
implícito

**Scale/Scope**: 1 módulo novo, 4 módulos alterados, ~6 campos de interface

## Constitution Check

*GATE: passa antes da pesquisa e continua passando depois do desenho.*

| Princípio | Verificação | Resultado |
|---|---|---|
| I. Código | Validação e comando sem `unwrap`/`panic`; protocolo continua puro; valores mágicos (máximos, ids 209–212) viram constantes nomeadas | ✅ |
| II. Testes | Validação, limites e fluxo de salvar com testes sem hardware; fixtures dos pacotes OVP/OCP/OPP/OTP; roteiro manual para a escrita real (quickstart) | ✅ |
| III. UX | Selecionar/editar não envia nada; só **Salvar** envia; pendência sinalizada e preservada; controles desabilitados sem conexão; textos em pt-BR; README atualizado; limite do aparelho respeitado | ✅ |
| IV. Desempenho | Envio só pela thread da sessão; sem redesenho contínuo; sem estrutura que cresça | ✅ |
| Restrições técnicas | Escrita de proteção marcada como experimental no código e no README | ✅ |
| Quality gates | fmt, clippy, test, release e roteiro manual listados no quickstart | ✅ |

**Débito resolvido**: a faixa 30 V / 5 A do setpoint (Princípio III) deixa de existir (FR-014). A
constituição ganha uma emenda PATCH 1.0.2 (veja Documentação).

Sem violações; a tabela de complexidade não se aplica.

## Project Structure

### Documentation (this feature)

```text
specs/003-memory-setup-fields/
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   └── ui-and-serial.md
└── tasks.md             # criado por /speckit-tasks
```

### Source Code (repository root)

```text
src/
├── logics/
│   ├── mod.rs           # + pub mod memory_setup
│   ├── memory_setup.rs  # NOVO: campos, máximos, edição pendente, validação
│   ├── app.rs           # setup em edição, troca de memória, salvar, conferência
│   ├── session.rs       # AppCommand::SaveMemorySetup e sua execução
│   └── protocol.rs      # só testes novos (pacotes de OVP/OCP/OPP/OTP)
└── gui/
    ├── panels.rs        # quadro de setup; setpoint com máximos novos; linha de memórias
    └── widgets.rs       # campo de setup reaproveitável (rótulo + valor + unidade)

docs/
├── README.md            # + linha do módulo novo
├── arquitetura.md       # módulos, diagramas D1/D3, tipos centrais
└── modulos/
    ├── logics-memory_setup.md   # NOVO
    ├── logics-app.md  logics-session.md  gui-panels.md  gui-widgets.md  # alterados
README.md                # memórias, proteções, limitações
.specify/memory/constitution.md  # PATCH 1.0.2
tests/docs_coverage.rs   # sem mudança: passa a exigir o documento novo
```

**Structure Decision**: a regra de validação vai para um módulo próprio de `logics` (e não para
`app.rs`, que já tem 41 funções) para ficar pura e pequena; segue a divisão `logics`/`gui` da feature 001.

## Decisões que mudam o comportamento atual

1. **Selecionar M1–M6 deixa de preencher o card "Setpoint"**; passa a preencher o quadro de setup. O
   comportamento antigo (carregar na saída e **Aplicar**) continua disponível por um botão **"Usar no
   setpoint"** no quadro. Motivo: o quadro precisa mostrar a memória sem marcar o setpoint como pendente.
2. **Trocar de memória com edição pendente** mostra um aviso na própria interface ("Descartar as
   alterações de Mn?" com **Descartar** e **Continuar editando**) em vez de perder em silêncio (US2-4).
3. **Salvar** deixa de ser só Vset/Iset do card; grava o quadro (seis valores).
