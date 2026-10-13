# `gui::fonts` — a fonte da interface

Arquivo: [`src/gui/fonts.rs`](../../src/gui/fonts.rs) · Visão geral: [arquitetura](../arquitetura.md)

## Papel

Guarda a **Noto Sans** dentro do executável e a instala como fonte padrão da interface. Com isso, compilar e rodar
o programa não dependem de nenhuma fonte instalada no sistema, e a aparência é a mesma em qualquer máquina.

## O que não faz

Não escolhe tamanhos nem cores de texto (isso é de [`gui::panels`](gui-panels.md), [`gui::widgets`](gui-widgets.md)
e [`gui::chart`](gui-chart.md)), não lê fonte do disco em tempo de execução e não oferece ao usuário outra fonte.

## Constantes

| Constante | Valor | Uso |
|---|---|---|
| `NOTO_SANS_NAME` | `"noto-sans"` | Nome da fonte nas definições do egui |
| `NOTO_SANS_BYTES` | o arquivo `assets/fonts/NotoSans-VariableFont_wght.ttf` (712 460 bytes) | A fonte, embutida no executável por `include_bytes!` |

A fonte é a Noto Sans 2.015 (arquivo variável; o eixo de peso `wght` vai de 400 a 700 e o padrão, 400, é o que o
egui usa). A licença é a SIL Open Font License 1.1, e o texto dela fica ao lado, em
[`assets/fonts/OFL.txt`](../../assets/fonts/OFL.txt).

## Funções

### `definitions`

*(interna)*

| | |
|---|---|
| **Propósito** | Montar as definições de fonte do egui com a Noto Sans na frente da família `Proportional`. |
| **Entradas** | nenhuma |
| **Saída** | Um `FontDefinitions` pronto para instalar. |
| **Efeitos** | nenhum (só monta; quem instala é `install_fonts`) |

**Como funciona**: parte das definições padrão do egui, acrescenta a Noto Sans em `font_data` e a coloca na
posição 0 de `Proportional`. As fontes do egui (Ubuntu-Light, emoji e ícones) ficam **atrás** dela, como reserva:
um caractere que a Noto Sans não tem cai nelas, em vez de virar quadradinho. A família `Monospace` não muda, porque
nenhum texto da interface a usa.

**Exemplo:** `Proportional` era `[Ubuntu-Light, NotoEmoji-Regular, emoji-icon-font]` e passa a ser
`[noto-sans, Ubuntu-Light, NotoEmoji-Regular, emoji-icon-font]`. O "á" sai da Noto Sans; um "✓", que ela não tem,
sai do emoji.

### `install_fonts`

| | |
|---|---|
| **Propósito** | Instalar a Noto Sans como fonte padrão da interface. |
| **Entradas** | `ctx`: o contexto do egui. |
| **Saída** | nada |
| **Efeitos** | Troca as fontes do contexto; vale a partir do próximo quadro. |

**Como funciona**: chama `ctx.set_fonts(definitions())`. Quem a chama é o [`main`](controller.md), dentro do
construtor da janela e **antes** de criar o `GuiApp`, para nenhum texto ser desenhado com a fonte antiga.

## Testes

Sete testes, todos sem janela:

- a Noto Sans é a primeira fonte de `Proportional`, e as fontes do egui continuam atrás dela na mesma ordem;
- `Monospace` não muda;
- os bytes embutidos são uma fonte TrueType (assinatura `00 01 00 00`) de mais de 100 KB e são exatamente os do
  arquivo em `assets/fonts/`;
- a Noto Sans é de fato usada: o mesmo texto, em 16 px, mede uma largura diferente com e sem ela;
- os seis valores máximos do quadro de setup (36,00 V, 8,200 A, 36,10 V, 8,200 A, 295,20 W e 99 °C) cabem, em 16 px,
  na largura mínima da coluna de controle (330 px).
