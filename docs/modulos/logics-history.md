# `logics::history` — histórico das amostras do gráfico

Arquivo: [`src/logics/history.rs`](../../src/logics/history.rs) · Visão geral: [arquitetura](../arquitetura.md)

## Papel

Guarda as últimas medições de tensão, corrente e potência, em ordem, para o gráfico desenhar. É uma fila de
tamanho fixo: quando enche, a amostra mais antiga sai para dar lugar à nova. Também controla **quando** gravar a
próxima amostra, a partir do intervalo que recebe.

## O que não faz

Não sabe de onde vêm os números (quem lê a fonte e monta a amostra é [`AppState::tick`](logics-app.md))
e não desenha. Também não lê o relógio: o instante atual (`now`) entra por parâmetro, o que deixa os testes
independentes do tempo real.

## Tipos e constantes

| Item | O que é |
|---|---|
| `HISTORY_LEN` | Número máximo de amostras guardadas: **300** |
| `Sample` | Uma medição: `voltage` (V), `current` (A) e `power` (W) |
| `History` | A fila de amostras mais o instante da última gravada (`last_sample`) |

Com 300 amostras, a janela visível do gráfico é `300 × intervalo de leitura`: de 30 s (100 ms) a 5 min (1000 ms).

## Funções

### `History::default`

| | |
|---|---|
| **Propósito** | Criar um histórico vazio pelo trait `Default`. |
| **Entradas** | nenhuma |
| **Saída** | Um `History` vazio, igual ao de `History::new`. |
| **Efeitos** | nenhum |

**Como funciona**: só chama `History::new()`. Existe porque tipos públicos com um `new()` sem argumentos
devem oferecer `Default`.

### `History::new`

| | |
|---|---|
| **Propósito** | Criar um histórico vazio. |
| **Entradas** | nenhuma |
| **Saída** | Um `History` sem amostras e sem registro da última gravação. |
| **Efeitos** | nenhum |

**Como funciona**: cria a fila já com espaço reservado para 300 amostras, para ela não precisar crescer
durante a sessão.

### `History::record_if_due`

| | |
|---|---|
| **Propósito** | Gravar uma amostra, mas só se já passou tempo suficiente desde a última. |
| **Entradas** | `now`: o instante atual. `interval`: o tempo mínimo entre duas amostras. `sample`: a amostra a gravar. |
| **Saída** | nada |
| **Efeitos** | Pode acrescentar `sample` ao fim da fila, descartar a mais antiga e atualizar o instante da última gravação. |

**Como funciona**:

1. Se já existe uma última gravação e `now - last_sample < interval`, não faz nada e sai. A subtração é
   "saturada" (nunca dá negativo), então um `now` anterior à última gravação também não grava.
2. Se a fila já tem `HISTORY_LEN` amostras, remove a mais antiga: **com a fila cheia, descarta a mais antiga
   antes de inserir**.
3. Acrescenta `sample` ao fim e guarda `now` como o instante da última gravação.

A primeira amostra é sempre gravada (não há última gravação para comparar).

**Exemplo:** com `interval` de 100 ms: uma amostra em t = 0 ms é gravada; outra em t = 50 ms é
ignorada; outra em t = 100 ms é gravada. Com 300 amostras na fila, a de número 301 expulsa a primeira.

### `History::clear`

| | |
|---|---|
| **Propósito** | Apagar todo o histórico. |
| **Entradas** | nenhuma |
| **Saída** | nada |
| **Efeitos** | Esvazia a fila e esquece o instante da última gravação. |

**Como funciona**: esvazia a fila e põe `last_sample` em `None`. Por isso a próxima chamada de
`record_if_due` grava na hora, sem esperar o intervalo. É o que o botão "Limpar" e a troca de conexão usam.

### `History::iter`

| | |
|---|---|
| **Propósito** | Percorrer as amostras guardadas. |
| **Entradas** | nenhuma |
| **Saída** | Um iterador sobre `&Sample`, da mais antiga para a mais recente. |
| **Efeitos** | nenhum |

**Como funciona**: devolve o iterador da fila. O gráfico o usa para achar o pico de cada série e para
desenhar as linhas.

### `History::len`

| | |
|---|---|
| **Propósito** | Dizer quantas amostras há. |
| **Entradas** | nenhuma |
| **Saída** | O número de amostras, de 0 a `HISTORY_LEN`. |
| **Efeitos** | nenhum |

**Como funciona**: devolve o tamanho da fila.

### `History::is_empty`

| | |
|---|---|
| **Propósito** | Dizer se o histórico está vazio. |
| **Entradas** | nenhuma |
| **Saída** | `true` se não há nenhuma amostra. |
| **Efeitos** | nenhum |

**Como funciona**: pergunta à fila se está vazia. Existe junto com `len` por convenção da linguagem.

### `History::last`

| | |
|---|---|
| **Propósito** | Obter a amostra mais recente. |
| **Entradas** | nenhuma |
| **Saída** | `Some(amostra)` com uma cópia da última, ou `None` se o histórico está vazio. |
| **Efeitos** | nenhum |

**Como funciona**: lê o fim da fila e copia o valor (`Sample` é `Copy`). A legenda do gráfico mostra esses
números ao lado de cada cor.

## Testes

Quatro testes: a primeira amostra é sempre gravada, uma amostra antes do intervalo é ignorada, o limite de 300
descarta a mais antiga e `clear` libera a próxima gravação mesmo dentro do intervalo.
