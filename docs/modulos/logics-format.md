# `logics::format` — formatação de durações

Arquivo: [`src/logics/format.rs`](../../src/logics/format.rs) · Visão geral: [arquitetura](../arquitetura.md)

## Papel

Transforma uma duração (`std::time::Duration`) em texto para a interface: o tempo com a saída ligada
("01:02:03") e a janela de tempo do gráfico ("30 s", "2.5 min").

## O que não faz

Não formata nenhuma outra grandeza (tensão, corrente e potência são formatadas direto nos painéis) e não
mede tempo: só recebe uma duração pronta.

## Funções

### `format_hhmmss`

| | |
|---|---|
| **Propósito** | Escrever uma duração como horas, minutos e segundos com dois dígitos cada. |
| **Entradas** | `d`: a duração. |
| **Saída** | Texto no formato `HH:MM:SS`. |
| **Efeitos** | nenhum |

**Como funciona**: usa só os segundos inteiros da duração (as frações são descartadas). As horas são
`segundos / 3600`, os minutos são `(segundos % 3600) / 60` e os segundos são `segundos % 60`. Cada parte
é preenchida com zero à esquerda até ter 2 dígitos; as horas não têm limite (100 horas aparecem como
`100:00:00`).

**Exemplo:** 3723 s → `"01:02:03"`; 0 s → `"00:00:00"`.

### `format_window`

| | |
|---|---|
| **Propósito** | Escrever a duração da janela do gráfico de forma curta, em segundos ou em minutos. |
| **Entradas** | `d`: a duração da janela (número de amostras × intervalo de leitura). |
| **Saída** | Texto como `"30 s"`, `"1 min"` ou `"2.5 min"`. |
| **Efeitos** | nenhum |

**Como funciona**: abaixo de 60 s, mostra os segundos arredondados para o inteiro, seguidos de ` s`. De 60 s
para cima, divide por 60 e mostra os minutos com uma casa decimal (`{:.1}`); se a casa decimal for zero, ela
é retirada (`"5.0 min"` vira `"5 min"`).

**Exemplo:** 30 s → `"30 s"`; 60 s → `"1 min"`; 150 s → `"2.5 min"`; 300 s → `"5 min"`.
Com 300 amostras, o intervalo de 100 ms dá 30 s e o de 1000 ms dá 5 min.

## Testes

Dois testes: o formato `HH:MM:SS` (3723 s e zero) e a janela do gráfico (30 s, 150 s e 300 s).
