# Implementation Plan: Estilo do botão de saída

**Branch**: `fix/ajustes-botao-saida` | **Date**: 2026-10-14 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `specs/006-output-button-style/spec.md`

## Summary

Arredondar os cantos do botão **SAÍDA LIGADA / DESLIGADA** com raio de 5 px e deixar 10 px livres entre a borda
de baixo dele e a barra inferior. É uma mudança de **estilo em uma função** (`output_button`, em
`src/gui/panels.rs`), com duas constantes novas em `gui::theme` (`OUTPUT_BUTTON_RADIUS = 5` e
`OUTPUT_BOTTOM_MARGIN = 10.0`). O raio entra pelo próprio `egui::Button`, que o aplica em todos os estados
(desabilitado, normal, mouse por cima, pressionado). Nada em `logics` muda, e o comportamento do botão fica igual.

## Technical Context

**Language/Version**: Rust, edição 2024, rustc 1.88 ou mais novo

**Primary Dependencies**: nenhuma nova (`eframe`/`egui` 0.34.3)

**Storage**: N/A

**Testing**: `cargo test` (raio do retângulo desenhado e folga de baixo, sem janela) + roteiro visual com a janela aberta

**Target Platform**: Linux (desktop)

**Project Type**: aplicativo desktop (biblioteca `dps150` + binário `controller`)

**Performance Goals**: nenhum custo novo por quadro

**Constraints**: Princípios I–IV; `logics` não muda; o comportamento do botão fica idêntico

**Scale/Scope**: 1 função alterada, 2 constantes, 2 testes, 3 documentos

## Constitution Check

*GATE: passa antes da pesquisa e continua passando depois do desenho.*

| Princípio | Verificação | Resultado |
|---|---|---|
| I. Código | Sem `unwrap`/`expect`/`panic`; os números 5 e 10 viram constantes nomeadas | ✅ |
| II. Testes | O raio e a folga são testados sem janela (lendo a forma desenhada e a posição do botão); o que exige olhos tem roteiro visual | ✅ |
| III. UX | O clique continua sendo a única ação que liga/desliga a saída; botão desabilitado sem conexão; textos em pt-BR; README atualizado | ✅ |
| IV. Desempenho | Mesmos widgets, mesma frequência de redesenho | ✅ |
| Restrições técnicas | Nenhuma dependência nova | ✅ |
| Quality gates | fmt, clippy, test, release e roteiro visual no quickstart | ✅ |

Sem violações; a tabela de complexidade não se aplica.

## Project Structure

### Documentation (this feature)

```text
specs/006-output-button-style/
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   └── output-button-style.md
└── tasks.md             # criado por /speckit-tasks
```

### Source Code (repository root)

```text
src/gui/
├── theme.rs     # + OUTPUT_BUTTON_RADIUS (5) e OUTPUT_BOTTOM_MARGIN (10.0)
└── panels.rs    # output_button: raio no botão e margem de baixo; devolve a Response

docs/modulos/
├── gui-panels.md   # output_button
└── gui-mod.md      # tabela de constantes de theme
README.md           # seção "Ligar e desligar a saída"
```

**Structure Decision**: só a camada `gui` muda; `toggle_output` e `output_on` (em `logics`) ficam intocados, o que
garante o FR-005 por construção.

## Decisões

1. **Raio pelo próprio botão** (`egui::Button::corner_radius`), não pelo estilo global: assim só este botão muda
   e o raio vale em todos os estados (o egui o passa ao `Frame` que desenha o botão).
2. **Margem como espaço antes do botão, em layout de baixo para cima.** O botão é o primeiro item do trecho
   "de baixo para cima" de `control_column`, então um espaço antes dele o sobe. O espaço acrescentado é `OUTPUT_BOTTOM_MARGIN` (o
   egui não soma o espaçamento entre itens depois de um espaço, como o teste mostrou: a conta com
   `item_spacing.y` descontado dava 7 px). A conta fica dentro de `output_button` (que passa a ser chamada só
   nesse layout) e é conferida por teste.
3. **Nada de `logics` muda**, nem a janela: o botão sobe 10 px dentro da coluna, que tem folga (vimos nos
   prints que sobra espaço entre o quadro de setup e o botão).
