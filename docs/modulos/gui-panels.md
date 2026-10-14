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
(`output_button`), que fica a 10 px do fim da coluna, onde começa a barra inferior.

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
| **Propósito** | Desenhar o card com o título, os campos de tensão e corrente com o botão Aplicar, e as memórias M1–M6. |
| **Entradas** | `ui`: a área. `app`: o estado. `colors`: as cores de V e A para os rótulos dos campos. |
| **Saída** | nada |
| **Efeitos** | Conforme o usuário interage: edita o setpoint, aplica, desfaz, carrega uma memória ou salva uma memória. |

**Como funciona**: o card tem quatro partes:

1. **Título** (`setpoint_title`): "Setpoint" e, quando `setpoint().is_dirty()`, o aviso "• não aplicado" e o botão **Desfazer**, que chama `discard_setpoint_edits()`.
2. **Campos**: dois campos numéricos arrastáveis, V (de 0 a 36,00, 2 casas) e A (de 0 a 8,200, 3 casas), com os máximos
   `VSET_MAX` e `ISET_MAX` de [`memory_setup`](logics-memory_setup.md), que editam
   direto `setpoint_mut().vset_input` e `cset_input`. Se algum mudou, chama `mark_setpoint_edited()`. Informa
   a `set_editing(...)` quais campos estão em foco ou sendo arrastados. Se o foco saiu de um campo por causa
   da tecla **Enter**, guarda isso para o botão Aplicar. Entre o campo V e o rótulo A há o espaço `FIELD_GAP`.
3. **Aplicar**, na **mesma linha** dos campos, à direita do campo A e separado dele pelo mesmo `FIELD_GAP`
   (habilitado só conectado; azul quando há pendência; com a mesma largura do botão **Desfazer**, dada por
   `action_button_width`): chama `apply_setpoints()`, assim como o Enter. Não há
   mais uma linha só de botões abaixo dos campos.
4. **Memórias**: um botão M1 a M6 por grupo. Clicar chama `request_select_memory(n)`, que mostra a memória no
   quadro de setup (veja `setup_card`) e, se há edição pendente, pede confirmação antes de trocar. Ao passar o
   mouse, mostra os valores salvos (ou "sem dados"). **Salvar** chama `save_memory_setup()` e só fica habilitado
   conectado e com os seis valores do quadro válidos; sem isso, a dica mostra o primeiro erro.

### `setpoint_title`

*(interna)*

| | |
|---|---|
| **Propósito** | Desenhar a linha do título do card Setpoint: o nome e, com pendência, o aviso "• não aplicado" e o botão Desfazer. |
| **Entradas** | `ui`: a área. `dirty`: se há alteração de tensão/corrente ainda não enviada. `align_x`: a posição horizontal onde o Desfazer deve começar (a do botão Aplicar), se conhecida. |
| **Saída** | A resposta do botão Desfazer (`Some` só com pendência); quem chama vê se ele foi clicado. |
| **Efeitos** | nenhum além de desenhar; quem descarta a edição é quem chama |

**Como funciona**: desenha "Setpoint" em destaque e, só com `dirty`, o aviso e o botão **Desfazer**. O aviso e o
botão ficam nesta linha, que existe com ou sem pendência, e não numa linha extra: assim o card não muda de altura
quando a pendência aparece e some. Para garantir isso, a linha tem a altura mínima de um botão (o texto do botão
arredondado para cima, mais o preenchimento): sem isso, a linha só com o texto seria uns 0,3 px mais baixa que a
com botão, e o que vem abaixo daria um pequeno salto.

**Alinhamento com o Aplicar:** depois do aviso, se `align_x` é conhecido e fica à direita do fim do aviso, a função
acrescenta o espaço que falta para o **Desfazer** começar exatamente em `align_x`, de modo que ele fique na mesma
posição horizontal do botão **Aplicar** da linha de baixo. Se o aviso já passou dessa posição, o botão vem logo
depois dele.

**Exemplo:** o Aplicar começa em x = 250 e o aviso termina em x = 170: a função põe o espaço que falta e o Desfazer
começa em 250. Se o Aplicar começasse em x = 150, o Desfazer ficaria logo depois do aviso.

### `action_button_width`

*(interna)*

| | |
|---|---|
| **Propósito** | Dar a largura comum dos botões Aplicar e Desfazer. |
| **Entradas** | `ui`: a área (para medir o texto na fonte do botão e ler o preenchimento). |
| **Saída** | A largura, em pontos. |
| **Efeitos** | nenhum |

**Como funciona**: mede os textos "Aplicar" e "Desfazer" na fonte de botão da interface, fica com o maior e soma o
preenchimento horizontal do botão nos dois lados. Os dois botões usam esse valor como largura **mínima**, então têm
o mesmo tamanho e o texto de cada um fica centralizado. Como o valor sai da medida do texto, e não de um número
fixo, continua certo se a fonte ou os textos mudarem.

**Exemplo:** "Desfazer" é mais largo que "Aplicar" na Noto Sans de 14 px. Com "Desfazer" medindo 52 e preenchimento
de 4 de cada lado, a largura comum é 60: o **Aplicar** cresce até 60 e o **Desfazer** continua com 60.

### `apply_button_x_id`

*(interna)*

| | |
|---|---|
| **Propósito** | Dar o identificador sob o qual se guarda a posição horizontal do botão Aplicar. |
| **Entradas** | nenhuma |
| **Saída** | Um `egui::Id` fixo. |
| **Efeitos** | nenhum |

**Como funciona**: cria o `Id` a partir do texto `"setpoint_apply_button_x"`. `setpoint_card` guarda nele, na memória
temporária do egui, onde o **Aplicar** começou, e lê o valor do quadro anterior para alinhar o **Desfazer** (que é
desenhado antes, na linha de cima). Como o **Aplicar** é desenhado a cada quadro, o valor sempre existe depois do
primeiro. Se o **Aplicar** andar (a largura dos campos mudou), `setpoint_card` pede mais um quadro de redesenho
para o **Desfazer** se acertar sem depender de um evento do usuário.

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
| **Saída** | A `Response` do botão (quem chama pode ignorá-la). |
| **Efeitos** | Ao clicar, liga ou desliga a saída da fonte. |

**Como funciona**: o texto e a cor dizem o **estado** atual: "SAÍDA LIGADA" (verde) quando `app.output_on()`,
"SAÍDA DESLIGADA" (cinza) quando não; a dica diz o que o clique fará. O botão ocupa a largura toda, com 56 px
de altura, e só é habilitado quando conectado. Clicar chama `app.toggle_output()`.

**Estilo**: os quatro cantos têm raio `OUTPUT_BUTTON_RADIUS` (5 px) em todos os estados (desabilitado, desligada,
ligada, mouse por cima e pressionado), porque o raio vai no próprio botão (`Button::corner_radius`) e o egui o
repassa ao quadro que o desenha. Antes de criar o botão, a função acrescenta um espaço de `OUTPUT_BOTTOM_MARGIN`
(10 px): ela é chamada num layout de baixo para cima, em que o primeiro item fica na base da área, então o espaço
sobe o botão e deixa 10 px livres entre ele e a barra inferior. O egui não acrescenta o espaçamento entre itens
depois desse espaço, e o teste da folga confirma os 10 px.

**Exemplo:** a área termina em y = 600. O espaço de 10 px ocupa de 590 a 600, e o botão, com 56 px de altura, ocupa de
534 a 590, com os cantos arredondados em 5 px.

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

## Testes

Nove testes, sem janela. Sete medem o texto em Noto Sans no corpo padrão da interface:

- a linha V + A + **Aplicar**, com os valores máximos (36,00 V e 8,200 A), cabe na largura mínima da coluna de
  controle (330 px), contando o preenchimento, o espaçamento entre widgets, o `FIELD_GAP` e as margens do card;
- a linha do título com "Setpoint", "• não aplicado" e **Desfazer** cabe na mesma largura;
- a linha do título tem a mesma altura com e sem pendência, então o card não muda de altura;
- o **Desfazer** começa exatamente na posição horizontal pedida (a do **Aplicar**);
- se o **Aplicar** estiver mais à esquerda que o fim do aviso, o **Desfazer** vem logo depois do aviso;
- a largura do **Desfazer** é a largura comum dos dois botões, e essa largura é a do texto mais largo ("Desfazer") mais o
  preenchimento. O **Aplicar** usa o mesmo valor, e a linha V + A + **Aplicar** continua cabendo na coluna mínima.

Os outros dois olham o botão de saída (desabilitado, que é o estado de um `AppState` novo):

- o retângulo desenhado do botão tem cantos com raio de 5;
- a folga entre o fundo do botão e o fundo da área é de 10 px (± 0,5), e o botão continua com 56 px de altura e a
  largura da área.
