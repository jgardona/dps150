# Implementation Plan: Ajuste do botão Aplicar na seção Setpoint

**Branch**: `fix/ajuste-botao-aplicar` | **Date**: 2026-10-13 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `specs/005-apply-button-placement/spec.md`

## Summary

Mover o botão **Aplicar** do card Setpoint para a linha dos campos V e A, logo à direita do campo A, com o
mesmo espaço que já separa V de A, e deixar o **Desfazer** (só com pendência) na linha do título, depois
de "• não aplicado". É uma mudança de **layout em uma função** (`setpoint_card`, em `src/gui/panels.rs`), com uma constante
nova em `gui::theme` para o espaço entre os grupos da linha, usada nos dois pontos para garantir que as duas
distâncias sejam iguais. Nenhuma regra, comando, cor, texto ou limite muda.

## Technical Context

**Language/Version**: Rust, edição 2024, rustc 1.88 ou mais novo

**Primary Dependencies**: nenhuma nova (`eframe`/`egui` 0.34.3)

**Storage**: N/A

**Testing**: `cargo test` (largura da linha na coluna mínima, sem janela) + roteiro visual com a janela aberta

**Target Platform**: Linux (desktop)

**Project Type**: aplicativo desktop (biblioteca `dps150` + binário `controller`)

**Performance Goals**: nenhum custo novo por quadro (os mesmos widgets, em outra ordem)

**Constraints**: Princípios I–IV; `logics` não muda; o comportamento do botão fica idêntico

**Scale/Scope**: 1 função alterada, 1 constante, 2 testes, 3 documentos

## Constitution Check

*GATE: passa antes da pesquisa e continua passando depois do desenho.*

| Princípio | Verificação | Resultado |
|---|---|---|
| I. Código | Sem `unwrap`/`expect`/`panic`; espaço entre grupos vira constante nomeada em vez de número repetido | ✅ |
| II. Testes | A regra de "cabe na largura mínima" é testada sem janela (medindo o texto em Noto Sans); o que exige olhos tem roteiro visual | ✅ |
| III. UX | Nada é enviado sem clique em **Aplicar** (ou **Enter**); botão desabilitado sem conexão; pendência sinalizada; textos em pt-BR; README atualizado | ✅ |
| IV. Desempenho | Mesmos widgets, mesma frequência de redesenho | ✅ |
| Restrições técnicas | Nenhuma dependência nova | ✅ |
| Quality gates | fmt, clippy, test, release e roteiro visual no quickstart | ✅ |

Sem violações; a tabela de complexidade não se aplica.

## Project Structure

### Documentation (this feature)

```text
specs/005-apply-button-placement/
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   └── setpoint-layout.md
└── tasks.md             # criado por /speckit-tasks
```

### Source Code (repository root)

```text
src/gui/
├── theme.rs     # + FIELD_GAP (espaço entre os grupos da linha de campos)
└── panels.rs    # setpoint_card: Aplicar na linha dos campos; Desfazer na linha do título

docs/modulos/
├── gui-panels.md   # setpoint_card
└── gui-mod.md      # tabela de constantes de theme
README.md           # seção Setpoint
```

**Structure Decision**: só a camada `gui` muda; `logics` (`apply_setpoints`, `discard_setpoint_edits`, a
pendência) fica intocada, o que garante o FR-005 por construção.

## Decisões

1. **Espaço único entre os grupos** (`FIELD_GAP = 10.0`, o valor que já existe): usado entre o campo V e o
   rótulo A **e** entre o campo A e o botão. Como é a mesma construção nos dois pontos, a diferença entre as
   distâncias é 0 px (SC-002), sem medir nada.
2. **Desfazer na linha do título** (FR-006, decisão do usuário): aparece só com pendência, depois de
   "• não aplicado". Essa linha já muda de conteúdo com a pendência, então a altura do card não muda e nada se
   desloca (SC-003). Ver [research.md](./research.md) R3.
3. **Nada de `logics` muda**, nem a janela (a coluna fica uma linha mais baixa, o que só dá folga).
