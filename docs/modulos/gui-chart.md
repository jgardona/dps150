# `gui::chart` — painel do gráfico

Arquivo: [`src/gui/chart.rs`](../../src/gui/chart.rs) · Visão geral: [arquitetura](../arquitetura.md)

## Papel

Desenha o lado direito da janela: a **legenda** com os valores mais recentes (cada item abre um seletor de
cor), o **gráfico** com as três séries (tensão, corrente e potência) e suas escalas, o **slider** do
intervalo de leitura e o botão **Limpar**.

## O que não faz

Não calcula as escalas (usa [`logics::scale`](logics-scale.md)), não guarda as amostras (vêm do
[`History`](logics-history.md) do `AppState`) e não decide o intervalo: ao mexer no slider, chama
`AppState::set_update_ms`. As cores são o único estado que ele altera, e ficam em `GuiApp`.

## Tipos

| Item | O que é |
|---|---|
| `SeriesSpec` | A descrição de uma série: como ler o valor numa amostra, a cor, a unidade ("V", "A", "W") e o passo mínimo da escala. É um alias de tupla que existe só para o código ficar legível |

## Funções

### `chart_panel`

| | |
|---|---|
| **Propósito** | Desenhar o painel do gráfico completo, dentro de um card. |
| **Entradas** | `ui`: a área do painel. `app`: o estado. `colors`: as cores de V, A e W (podem ser mudadas aqui). |
| **Saída** | nada |
| **Efeitos** | Pode mudar uma cor, o intervalo de leitura (envia `SetPollInterval`) e limpar o histórico. |

**Como funciona**: o card tem três partes, de cima para baixo:

1. **Legenda**: para cada grandeza, um botão sem moldura "■ valor", na cor dela, com o valor mais recente
   (`--` se não há dados). Clicar abre um **popup** com o seletor de cor do egui, com largura
   `COLOR_PICKER_WIDTH`, e um botão **Padrão** que volta à cor original. O popup fecha ao clicar fora. À
   direita, o texto "janela …", com a duração visível: `update_ms × 300 amostras`, formatada por
   `format_window`.
2. **Gráfico**: ocupa a altura que sobra menos 30 px (no mínimo 60 px) e é desenhado por `draw_history_chart`.
3. **Controles**: o slider **Update** (de 100 a 1000 ms, de 50 em 50). Quando o valor muda, chama
   `app.set_update_ms(...)`. À direita, o botão **Limpar**, que chama `app.clear_history()`.

### `draw_history_chart`

*(interna)*

| | |
|---|---|
| **Propósito** | Desenhar a área do gráfico: fundo, grade, escalas e as três linhas. |
| **Entradas** | `ui`: a área. `history`: as amostras. `colors`: as cores de V, A e W. `height`: a altura do gráfico em pixels. |
| **Saída** | nada |
| **Efeitos** | nenhum (só desenha) |

**Como funciona**:

1. **Reserva o retângulo** do gráfico e pinta o fundo. Dentro dele, a **área das linhas** deixa à esquerda
   `3 × 40 px` para as três colunas de escala (V, A, W) e uma margem em cima e embaixo.
2. **Grade horizontal**: 16 espaços (`4 divisões × 4 subdivisões`), com linha mais clara a cada 4 (as
   divisões principais) e mais discreta nos 3 ticks intermediários entre elas. **Grade vertical**: 6 colunas.
   Os rótulos intermediários só aparecem se couberem (cada espaço com pelo menos 11 px de altura); em
   janelas baixas, ficam só os principais.
3. **Eixo X**: cada amostra ocupa `largura / 299` pixels. O ponto **mais recente fica sempre à direita**, e o
   gráfico "anda" para a esquerda conforme chegam amostras. Com poucas amostras, a linha começa no meio.
4. **Para cada série** (V, A, W): acha o **pico** do histórico, pede a escala a `AxisScale::for_peak` (com o
   passo mínimo da grandeza), escreve a unidade e os números da escala numa coluna própria, **na cor da
   série**, e liga os pontos com uma linha de 1,5 px. A altura de cada ponto é `valor / topo` da escala da
   própria série, limitada entre 0 e 1.

Como cada série usa a sua escala, **as alturas de linhas diferentes não são comparáveis entre si**: o que se lê
são os números da coluna de escala da cor correspondente.

**Exemplo:** um histórico com pico de 3,3 V gera, para a tensão, `max` = 4 e 2 casas. Os ticks principais são
0, 1, 2, 3 e 4 e os intermediários vão de 0,25 em 0,25. Um ponto de 3,3 V é desenhado a 3,3 / 4 = 82,5% da
altura. Com 150 amostras no histórico, a linha ocupa a metade direita do gráfico, de `x0 = direita - 149 ×
passo` até a borda direita.
