# Implementation Plan: Fonte Noto Sans e rótulos legíveis nas memórias

**Branch**: `feature/label-correcao` | **Date**: 2026-10-13 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `specs/004-noto-sans-fonts/spec.md`

## Summary

- **Fonte**: guardar a Noto Sans (arquivo variável, peso 400 como padrão) em `assets/`, com a licença OFL,
  e incluí-la no executável (`include_bytes!`). Um módulo novo `src/gui/fonts.rs` monta as definições de
  fonte (Noto Sans na frente de `Proportional`, fontes de reserva do egui mantidas) e as instala na criação
  da janela. O egui 0.34.3 já renderiza fontes variáveis, então não há dependência nova.
- **Tamanho**: uma constante `SETUP_FONT_SIZE = 16.0` em `gui::theme`, aplicada ao título, aos rótulos e aos
  campos (valor e unidade) do quadro de setup.
- **Verificação**: testes que não precisam de janela (as definições de fonte e o arquivo embutido) e um
  roteiro visual (nenhum quadradinho, nenhum corte, nada sai da janela).
- **Documentação**: README, `docs/` (módulo novo `gui-fonts.md`) e a licença ao lado da fonte.

Nenhuma regra de `logics` muda. Só `gui`, `controller.rs`, `assets/` e a documentação.

## Technical Context

**Language/Version**: Rust, edição 2024, rustc 1.88 ou mais novo

**Primary Dependencies**: nenhuma nova (`eframe`/`egui`/`epaint` 0.34.3, que já tem suporte a fontes variáveis)

**Storage**: arquivos versionados em `assets/` (fonte e licença), incluídos no binário em tempo de compilação

**Testing**: `cargo test` (definições de fonte e arquivo embutido, sem janela) + roteiro visual com a janela aberta

**Target Platform**: Linux (desktop); a fonte embutida não depende de nada do sistema

**Project Type**: aplicativo desktop (biblioteca `dps150` + binário `controller`)

**Performance Goals**: abrir a janela em no máximo +0,5 s em relação à versão anterior (SC-006); nenhum
trabalho novo por quadro (a fonte é instalada uma vez)

**Constraints**: Princípios I–IV; `logics` continua sem egui; o executável cresce (≈ 0,7 MB, registrar)

**Scale/Scope**: 1 módulo novo em `gui`, 1 constante, 2 arquivos em `assets/`, ajustes em `widgets.rs`,
`panels.rs` e `controller.rs`

## Constitution Check

*GATE: passa antes da pesquisa e continua passando depois do desenho.*

| Princípio | Verificação | Resultado |
|---|---|---|
| I. Código | Sem `unwrap`/`expect`/`panic` (a fonte é embutida: não há leitura de arquivo em tempo de execução); constantes nomeadas para o tamanho e o nome da fonte; comentários em pt-BR com o porquê | ✅ |
| II. Testes | Definições de fonte e bytes embutidos testados sem janela; o que exige olhos (nenhum quadradinho, nenhum corte) tem roteiro visual no quickstart | ✅ |
| III. UX | Textos em pt-BR sem quadradinho; nada muda no envio de comandos; README atualizado; imagem do README fora de escopo (opcional) | ✅ |
| IV. Desempenho | Fonte instalada uma vez na criação da janela; nenhum redesenho novo; custo de tamanho do executável medido e registrado | ✅ |
| Restrições técnicas | Nenhuma dependência nova. Licença OFL compatível e distribuída junto da fonte | ✅ |
| Quality gates | fmt, clippy, test, release e roteiro visual listados no quickstart | ✅ |

Sem violações; a tabela de complexidade não se aplica.

## Project Structure

### Documentation (this feature)

```text
specs/004-noto-sans-fonts/
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   └── ui-and-assets.md
└── tasks.md             # criado por /speckit-tasks
```

### Source Code (repository root)

```text
assets/
└── fonts/
    ├── NotoSans-VariableFont_wght.ttf   # NOVO (cópia da Noto Sans 2.015, OFL)
    └── OFL.txt                          # NOVO (licença ao lado da fonte)

src/
├── controller.rs        # instala as fontes ao criar a janela
└── gui/
    ├── mod.rs           # + mod fonts; re-exporta install_fonts
    ├── fonts.rs         # NOVO: definições de fonte e instalação
    ├── theme.rs         # + SETUP_FONT_SIZE
    ├── widgets.rs       # setup_field usa SETUP_FONT_SIZE
    └── panels.rs        # título do setup usa SETUP_FONT_SIZE

docs/
├── README.md  arquitetura.md          # módulo novo na lista e nos diagramas
└── modulos/
    ├── gui-fonts.md                   # NOVO
    └── gui-mod.md  gui-panels.md  gui-widgets.md  controller.md   # alterados
README.md                              # fonte, licença, assets, tamanho do setup
```

**Structure Decision**: a fonte fica num módulo próprio de `gui` (`fonts.rs`), e não em `controller.rs`, para
ser testável sem abrir janela e para o `controller` continuar só com o `main`.

## Decisões

1. **Um arquivo variável, só o peso normal (400).** O arquivo variável tem 712 KB e serve todos os
   textos. O egui não escolhe peso por `strong()` (só muda a cor), então o "negrito" de hoje já é só cor e
   continua igual (FR-009). Um negrito de verdade custaria uma segunda entrada de fonte e trocar o `strong()`
   em ~10 pontos; fica fora desta entrega.
2. **Fontes de reserva mantidas.** A Noto Sans entra **na frente** de `Proportional`; Ubuntu-Light, o emoji
   e os ícones do egui continuam depois dela, para caracteres que a Noto Sans não tem (ex.: ✓).
3. **`Monospace` não muda**: nenhum texto da interface o usa.
4. **A janela não cresce.** O print mais recente mostra a janela com sobra de espaço vertical acima do botão
   de saída; o quadro maior cabe nos 700 px atuais. O quickstart confere e, se não couber, aumenta
   `HEIGHT`.
