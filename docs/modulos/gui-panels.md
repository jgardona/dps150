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
| **Efeitos** | Os de `setpoint_card`, `setup_card` e `output_button`. |

**Como funciona**: desenha as leituras (`readouts`), um espaço de 8 px, o card de setpoint
(`setpoint_card`), outro espaço de 8 px e o quadro de setup da memória (`setup_card`). Em seguida, num trecho que cresce **de baixo para cima**, desenha o botão de saída
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
2. **Campos**: dois campos numéricos arrastáveis, V (de 0 a 36,00, 2 casas) e A (de 0 a 8,200, 3 casas), com os máximos
   `VSET_MAX` e `ISET_MAX` de [`memory_setup`](logics-memory_setup.md), que editam
   direto `setpoint_mut().vset_input` e `cset_input`. Se algum mudou, chama `mark_setpoint_edited()`. Informa
   a `set_editing(...)` quais campos estão em foco ou sendo arrastados. Se o foco saiu de um campo por causa
   da tecla **Enter**, guarda isso para o botão Aplicar.
3. **Botões**: **Aplicar** (habilitado só conectado; azul quando há pendência) chama `apply_setpoints()`, assim
   como o Enter; **Desfazer** (só aparece com pendência) chama `discard_setpoint_edits()`.
4. **Memórias**: um botão M1 a M6 por grupo. Clicar chama `request_select_memory(n)`, que mostra a memória no
   quadro de setup (veja `setup_card`) e, se há edição pendente, pede confirmação antes de trocar. Ao passar o
   mouse, mostra os valores salvos (ou "sem dados"). **Salvar** chama `save_memory_setup()` e só fica habilitado
   conectado e com os seis valores do quadro válidos; sem isso, a dica mostra o primeiro erro.

### `setup_card`

*(interna)*

| | |
|---|---|
| **Propósito** | Desenhar o quadro de setup da memória selecionada: Vset, Iset, OVP, OCP, OPP e OTP, em duas linhas de três. |
| **Entradas** | `ui`: a área. `app`: o estado. `colors`: as cores de V, A e W para os rótulos. |
| **Saída** | nada |
| **Efeitos** | Edita o quadro, descarta a edição, confirma ou cancela uma troca de memória e carrega a memória no setpoint. **Nunca envia nada à fonte** (quem envia é o **Salvar**, em `setpoint_card`). |

**Como funciona**:

1. **Título** "Setup Mn" em 16 px (`SETUP_FONT_SIZE`), com "• não salvo" quando há edição pendente.
2. **Grade** de três colunas com `setup_field` para cada campo, na ordem do visor. As cores são a de V (Vset e
   OVP), a de A (Iset e OCP) e a de W (OPP); OTP não tem cor. As edições são anotadas durante o desenho e
   aplicadas só no fim (`set_setup_field`), porque o quadro lê o estado enquanto desenha.
3. **Erros**: com edição pendente, valida os seis valores; os campos com erro ficam vermelhos e as três primeiras
   mensagens aparecem embaixo. Sem edição pendente, não mostra erro (um campo vazio, antes de a fonte informar,
   não é um erro do usuário).
4. **Troca de memória em espera**: se `pending_switch()` tem um pedido, mostra "Descartar as alterações de Mn?"
   com **Descartar** (`confirm_switch`) e **Continuar editando** (`cancel_switch`).
5. **Botões e estado**: **Usar no setpoint** (conectado e com dados; chama `use_memory_in_setpoint`), **Desfazer**
   (só com pendência; chama `discard_memory_setup_edits`) e o estado da gravação: "verificando…", "gravado" (verde)
   ou "não confirmado" (vermelho).
6. **Rodapé**: "* vale para o aparelho todo", porque OVP, OCP, OPP e OTP não são guardados por memória.

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
