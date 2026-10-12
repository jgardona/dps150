# `logics::scale` — escalas dos eixos do gráfico

Arquivo: [`src/logics/scale.rs`](../../src/logics/scale.rs) · Visão geral: [arquitetura](../arquitetura.md)

## Papel

Decide **até onde vai o eixo** de cada grandeza no gráfico (o "topo" da escala) e **quantas casas
decimais** os números dele precisam. O gráfico desenha tensão (V), corrente (A) e potência (W) na mesma
área, mas cada uma tem a sua escala, porque as faixas são muito diferentes (por exemplo, 30 V contra 0,05 A).
Cada escala tem **4 divisões principais**, e cada divisão tem **4 subdivisões** (3 ticks intermediários).

## O que não faz

Não desenha nada e não conhece a interface: só calcula números. O desenho está em
[`gui::chart`](gui-chart.md).

## Tipos e constantes

| Item | O que é |
|---|---|
| `CHART_Y_DIVS` | Número de divisões principais do eixo: **4** |
| `CHART_Y_SUBDIVS` | Subdivisões de cada divisão principal: **4** (3 ticks intermediários entre dois principais) |
| `AxisScale` | O resultado: `max` (valor do topo do eixo) e `decimals` (casas decimais dos rótulos) |

## Funções

### `decimals_for`

| | |
|---|---|
| **Propósito** | Descobrir quantas casas decimais são necessárias para escrever um número sem arredondá-lo. |
| **Entradas** | `step`: um número positivo, normalmente o espaço entre dois ticks. |
| **Saída** | O menor número de casas decimais (de 0 a 4) que representa `step` exatamente; 4 se nenhuma de 0 a 4 servir. |
| **Efeitos** | nenhum |

**Como funciona**: tenta 0, 1, 2, 3 e 4 casas, nessa ordem. Para cada `d`, multiplica `step` por 10^`d` e
vê se o resultado é (quase) um número inteiro; o primeiro `d` que serve é a resposta. "Quase" quer dizer
que a diferença para o inteiro mais próximo é menor que 0,1% do valor, o que absorve o erro de ponto
flutuante.

**Exemplo:** `decimals_for(1.0)` → 0 (1 já é inteiro); `decimals_for(0.25)` → 2 (0,25 × 100 = 25);
`decimals_for(0.025)` → 3 (0,025 × 1000 = 25).

### `AxisScale::for_peak`

| | |
|---|---|
| **Propósito** | Calcular o topo e as casas decimais da escala de uma grandeza, a partir do maior valor que aparece no gráfico. |
| **Entradas** | `peak`: o maior valor visível da série (0 se não houver dados). `min_step`: o menor valor que uma divisão pode ter (0,1 para V e W, 0,01 para A). |
| **Saída** | Um `AxisScale` com `max` (topo do eixo) e `decimals` (casas decimais para os rótulos). |
| **Efeitos** | nenhum |

**Como funciona**: o topo é sempre um valor "redondo", para os rótulos do eixo serem fáceis de ler.

1. Calcula `raw_step = max(peak / 4, min_step)`: o valor que cada uma das 4 divisões precisaria ter para o
   pico caber. O `min_step` impede uma escala microscópica quando o pico é zero ou muito pequeno (um ruído
   de poucos mA não deve encher o gráfico).
2. Pega a ordem de grandeza: `magnitude = 10^floor(log10(raw_step))`.
3. Escolhe o `step`: o **menor** de `1 × magnitude`, `2 × magnitude`, `5 × magnitude` e
   `10 × magnitude` que seja maior ou igual a `raw_step`. Há uma tolerância de 0,1% (`raw_step × 0,999`)
   para o erro de ponto flutuante não empurrar o resultado para o passo seguinte sem necessidade.
4. `decimals = decimals_for(step / 4)`: as casas necessárias para os ticks intermediários saírem exatos.
5. `max = step × 4`.

**Exemplo:** pico de 3,3 V, `min_step` 0,1 → `raw_step` 0,825, `magnitude` 0,1, candidatos 0,1 / 0,2 / 0,5 / 1,0
→ `step` 1,0 → **`max` = 4, `decimals` = 2** (ticks de 0,25 em 0,25).
Pico de 0 A, `min_step` 0,01 → `step` 0,01 → **`max` = 0,04, `decimals` = 4** (ticks de 0,0025).
Pico de 15 V, `min_step` 0,1 → `raw_step` 3,75 → `step` 5 → **`max` = 20, `decimals` = 2**.

## Testes

Três testes cobrem este módulo: o topo "redondo" acima do pico (3,3 V → 4; 15 V → 20), o pico zero usando o
passo mínimo (0,04 para a corrente e 0,4 para V e W) e as casas decimais exatas (`decimals_for`).
