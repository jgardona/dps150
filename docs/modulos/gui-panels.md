# `gui::panels` — barras e coluna de controle

Arquivo: [`src/gui/panels.rs`](../../src/gui/panels.rs) · Visão geral: [arquitetura](../arquitetura.md)

## Papel

Desenha tudo o que não é o gráfico: a **barra superior** (conexão e informações do aparelho), a **coluna de
controle** (as três leituras, o setpoint, as memórias e o botão de saída) e a **barra inferior**
(capacidade, energia e tempo). É a camada que transforma cliques em chamadas ao `AppState`.

## O que não faz

Não decide nenhuma regra: não calcula o que fica pendente, não monta comandos e não fala com a serial. Cada
ação do usuário vira **uma chamada** a um método de [`AppState`](logics-app.md), e o estado que a tela mostra
vem sempre dele. As cores das grandezas chegam por parâmetro (`colors`), na ordem V, A, W.

## Funções

### `top_bar`

| | |
|---|---|
| **Propósito** | Desenhar a barra superior: status, porta, botão de conexão, mensagem de erro e dados do aparelho. |
| **Entradas** | `ui`: a área onde desenhar. `app`: o estado da aplicação. |
| **Saída** | nada |
| **Efeitos** | Pode conectar ou desconectar, e trocar a porta escolhida. |

**Como funciona**: da esquerda para a direita:

1. **Ponto de status**: cinza (desconectado), amarelo (conectando), verde (conectado) ou vermelho (erro), com o
   texto como dica ao passar o mouse.
2. **Seletor de porta**: só habilitado sem sessão (`is_idle`). Lista `available_ports()` e grava a escolha em
   `selected_port_mut()`.
3. **Botão de conexão**: o texto muda com o estado (Conectar, Cancelar, Desconectar). Ao clicar: sem sessão,
   chama `app.connect(...)` passando uma função que pede o redesenho da tela; com sessão (ou conectando),
   chama `app.disconnect()`.
4. **Mensagem de erro**, em vermelho, se o estado for `Error`.
5. **Informações do aparelho**, à direita, **só quando conectado**: modelo, firmware, tensão de entrada e
   temperatura, lidos de `app.device()`.

### `control_column`

| | |
|---|---|
| **Propósito** | Montar a coluna de controle, de cima para baixo. |
| **Entradas** | `ui`: a área da coluna. `app`: o estado. `colors`: as cores de V, A e W. |
| **Saída** | nada |
| **Efeitos** | Os de `setpoint_card` e `output_button`. |

**Como funciona**: desenha as leituras (`readouts`), um espaço de 8 px e o card de setpoint
(`setpoint_card`). Em seguida, num trecho que cresce **de baixo para cima**, desenha o botão de saída
(`output_button`), que fica assim encostado no fim da coluna.

### `readouts`

*(interna)*

| | |
|---|---|
| **Propósito** | Desenhar as três leituras grandes: tensão, corrente e potência. |
| **Entradas** | `ui`: a área. `app`: o estado (só leitura). `colors`: as cores de V, A e W. |
| **Saída** | nada |
| **Efeitos** | nenhum (só desenha) |

**Como funciona**: usa o `ReadoutRow` para cada grandeza, na cor própria, com o valor medido pela fonte
(`0` enquanto ela não informou): tensão com 2 casas, corrente com 3 e potência com 2. O **selo** de modo
aparece na leitura de tensão se a fonte está em `CV` e na de corrente se está em `CC`; a potência não tem selo.

### `setpoint_card`

*(interna)*

| | |
|---|---|
| **Propósito** | Desenhar o card com os campos de tensão e corrente, os botões Aplicar e Desfazer e as memórias M1–M6. |
| **Entradas** | `ui`: a área. `app`: o estado. `colors`: as cores de V e A para os rótulos dos campos. |
| **Saída** | nada |
| **Efeitos** | Conforme o usuário interage: edita o setpoint, aplica, desfaz, carrega uma memória ou salva uma memória. |

**Como funciona**: o card tem quatro partes:

1. **Título** "Setpoint", com o aviso "• não aplicado" quando `setpoint().is_dirty()`.
2. **Campos**: dois campos numéricos arrastáveis, V (de 0 a 30, 2 casas) e A (de 0 a 5, 3 casas), que editam
   direto `setpoint_mut().vset_input` e `cset_input`. Se algum mudou, chama `mark_setpoint_edited()`. Informa
   a `set_editing(...)` quais campos estão em foco ou sendo arrastados. Se o foco saiu de um campo por causa
   da tecla **Enter**, guarda isso para o botão Aplicar.
3. **Botões**: **Aplicar** (habilitado só conectado; azul quando há pendência) chama `apply_setpoints()`, assim
   como o Enter; **Desfazer** (só aparece com pendência) chama `discard_setpoint_edits()`.
4. **Memórias**: um botão M1 a M6 por grupo. Clicar chama `load_profile(n)`, que só preenche os campos. Ao
   passar o mouse, mostra os valores salvos (ou "sem dados"). **Salvar** (habilitado só conectado) chama
   `save_profile(n)` para a memória selecionada.

### `output_button`

*(interna)*

| | |
|---|---|
| **Propósito** | Desenhar o botão grande que liga e desliga a saída. |
| **Entradas** | `ui`: a área. `app`: o estado. |
| **Saída** | nada |
| **Efeitos** | Ao clicar, liga ou desliga a saída da fonte. |

**Como funciona**: o texto e a cor dizem o **estado** atual: "SAÍDA LIGADA" (verde) quando `app.output_on()`,
"SAÍDA DESLIGADA" (cinza) quando não; a dica diz o que o clique fará. O botão ocupa a largura toda, com 56 px
de altura, e só é habilitado quando conectado. Clicar chama `app.toggle_output()`.

### `bottom_bar`

| | |
|---|---|
| **Propósito** | Desenhar a barra inferior com os acumuladores. |
| **Entradas** | `ui`: a área. `app`: o estado (só leitura). `now`: o instante atual. |
| **Saída** | nada |
| **Efeitos** | nenhum (só desenha) |

**Como funciona**: mostra, separados por traços, a **capacidade** (Ah, 3 casas) e a **energia** (Wh, 2 casas),
que vêm da própria fonte, e o **tempo** com a saída ligada, que o programa mede: `app.current_elapsed(now)`
formatado por `format_hhmmss`.
