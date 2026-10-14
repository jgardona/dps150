# `gui` — `GuiApp` e o ciclo de cada quadro

Arquivo: [`src/gui/mod.rs`](../../src/gui/mod.rs) · Visão geral: [arquitetura](../arquitetura.md)

## Papel

Raiz do módulo `gui`. Define o `GuiApp`, a aplicação gráfica que o egui executa, e coordena **cada quadro**:
atualiza o estado, divide a janela em colunas e chama os painéis. Também declara os submódulos `panels`,
`chart`, `widgets`, `fonts` e `theme`, e reexporta `install_fonts`, de [`fonts`](gui-fonts.md).

## O que não faz

Não tem regra de negócio e não acessa a serial: tudo isso fica em [`AppState`](logics-app.md). O que ele
decide é só de **apresentação**: a ordem de desenho e a largura das colunas.

## Tipos

| Item | O que é |
|---|---|
| `GuiApp` | A aplicação: um `AppState` (as regras e o estado) e `series_colors`, as três cores escolhidas para V, A e W, que são uma preferência de **apresentação** e por isso ficam aqui |

## Constantes de `theme`

O módulo [`gui/theme.rs`](../../src/gui/theme.rs) não tem funções; só constantes visuais:

| Constante | Valor | Uso |
|---|---|---|
| `CONTROL_SHARE` | 0,36 | Fração da largura da janela para a coluna de controle |
| `CONTROL_MIN_WIDTH` / `CONTROL_MAX_WIDTH` | 330 / 440 px | Limites da largura da coluna de controle |
| `FIELD_GAP` | 10 px | Espaço extra entre os grupos da linha de campos do Setpoint (campo V e rótulo A; campo A e botão Aplicar), somado ao espaçamento padrão entre widgets |
| `SETUP_FONT_SIZE` | 16 px | Tamanho do texto do quadro de setup das memórias (título, rótulos, valores e unidades) |
| `COLOR_PICKER_WIDTH` | 200 px | Largura do seletor de cor da legenda |
| `CHART_AXIS_COL_W` | 40 px | Largura de cada uma das três colunas de escala do gráfico |
| `DEFAULT_SERIES_COLORS` | vermelho, verde, azul | Cores iniciais de V, A e W |
| `OK_COLOR`, `WARN_COLOR`, `ERROR_COLOR`, `ACTION_COLOR` | verde, amarelo, vermelho, azul | Estados de conexão, avisos e o botão "Aplicar" |

## Funções

### `GuiApp::default`

| | |
|---|---|
| **Propósito** | Criar a aplicação pelo trait `Default`. |
| **Entradas** | nenhuma |
| **Saída** | O mesmo que `GuiApp::new()`. |
| **Efeitos** | nenhum |

**Como funciona**: chama `new()`.

### `GuiApp::new`

| | |
|---|---|
| **Propósito** | Criar a aplicação com o estado inicial. |
| **Entradas** | nenhuma |
| **Saída** | Um `GuiApp` com um `AppState` novo e as cores padrão. |
| **Efeitos** | nenhum |

**Como funciona**: cria o `AppState` (desconectado) e copia `DEFAULT_SERIES_COLORS` para as cores atuais.
O [`main`](controller.md) o entrega ao egui.

### `GuiApp::ui`

| | |
|---|---|
| **Propósito** | Executar um quadro: atualizar o estado e desenhar a janela inteira. Implementa `eframe::App`. |
| **Entradas** | `ui`: a área da janela. O segundo parâmetro (o `Frame` do eframe) não é usado. |
| **Saída** | nada |
| **Efeitos** | Atualiza o estado da aplicação, desenha e agenda o próximo quadro. |

**Como funciona**: o egui chama esta função a cada quadro. Ela faz, nesta ordem (veja o
[diagrama](../arquitetura.md#ciclo-por-quadro)):

1. Lê o instante atual **uma única vez** (`now`). As regras recebem esse valor em vez de ler o relógio, o que
   as torna testáveis.
2. Atualiza o estado: `refresh_ports_if_due(now)`, `poll_events()` e `tick(now)`.
3. Desenha o **painel superior** (`top_bar`) e o **inferior** (`bottom_bar`).
4. Divide a área que sobrou em duas colunas, a de **controle** (`control_column`) e a do **gráfico**
   (`chart_panel`), e desenha cada uma numa sub-área (`column_ui`).
5. Pede o próximo quadro em até 100 ms (o menor entre 100 ms e o intervalo de leitura), para o relógio e o
   gráfico andarem mesmo sem dados novos.

**A largura das colunas.** A coluna de controle ocupa `36%` da área (`CONTROL_SHARE`), mas nunca menos de 330
nem mais de 440 px (`clamp`). O gráfico fica com o resto, descontado um espaço de duas vezes o espaçamento
padrão entre itens. Isso mantém os controles legíveis em janelas pequenas e deixa o espaço extra para o gráfico
em janelas grandes.

**Exemplo:** numa área de 1000 px, `0,36 × 1000 = 360` e o resultado fica em **360 px**. Numa área de 1600 px,
`0,36 × 1600 = 576` passa do máximo e fica em **440 px**. Numa de 800 px, `0,36 × 800 = 288` está abaixo do
mínimo e fica em **330 px**. Em todos os casos, o gráfico recebe a largura que sobra.

Ao fim, `ui.advance_cursor_after_rect(full)` avisa o egui de que toda a área foi usada, já que as colunas foram
desenhadas em sub-áreas.
