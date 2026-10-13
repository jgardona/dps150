# Modelo de dados: fonte e tamanho do setup

**Feature**: [spec.md](./spec.md) | **Plan**: [plan.md](./plan.md)

Esta feature não tem dados de negócio. Os itens abaixo são os valores fixos que ela introduz.

## Constantes (`gui::fonts` e `gui::theme`)

| Constante | Valor | Onde | Uso |
|---|---|---|---|
| `NOTO_SANS_NAME` | `"noto-sans"` | `gui::fonts` | Nome da fonte nas definições do egui |
| `NOTO_SANS_BYTES` | conteúdo de `assets/fonts/NotoSans-VariableFont_wght.ttf` | `gui::fonts` | A fonte, embutida no executável |
| `SETUP_FONT_SIZE` | `16.0` | `gui::theme` | Tamanho do texto do quadro de setup (título, rótulos, valores, unidades) |

## Definições de fonte (resultado de `fonts::definitions()`)

| Família | Ordem das fontes |
|---|---|
| `Proportional` | `noto-sans`, depois as do egui (Ubuntu-Light, NotoEmoji, emoji-icon-font) |
| `Monospace` | as do egui (sem mudança) |

Invariantes:

- `noto-sans` existe em `font_data` e é a **primeira** de `Proportional`.
- As fontes de reserva do egui continuam em `Proportional` (nenhuma é removida).
- `Monospace` fica igual ao padrão do egui.
- Os bytes embutidos começam com a assinatura de uma fonte TrueType (`0x00 0x01 0x00 0x00`) e são maiores que 100 KB.

## Arquivos em `assets/fonts/`

| Arquivo | Origem | Licença |
|---|---|---|
| `NotoSans-VariableFont_wght.ttf` | Noto Sans 2.015, `notofonts/latin-greek-cyrillic` | SIL OFL 1.1 |
| `OFL.txt` | mesmo repositório | texto da licença |
