# `logics::app` — estado e regras da aplicação

Arquivo: [`src/logics/app.rs`](../../src/logics/app.rs) · Visão geral: [arquitetura](../arquitetura.md)

## Papel

É o **cérebro** do programa. O `AppState` guarda tudo o que a aplicação sabe (o último estado da fonte, a
conexão, a porta escolhida, o setpoint em edição, a memória selecionada, o tempo de saída, o intervalo de
leitura e o histórico do gráfico) e contém **todas as regras** sobre esse estado: quando um setpoint está
pendente, o que acontece ao carregar uma memória, quando o relógio conta, quando uma amostra entra no
histórico.

A interface só **lê** o estado por métodos como `device()` e `is_connected()`, e só **age** chamando métodos
como `apply_setpoints()`. É assim que as regras ficam testáveis sem janela e sem a fonte (veja
[Testes](#testes)).

## O que não faz

Não desenha nada e não conhece a biblioteca de interface. Não abre a porta serial: pede à
[sessão](logics-session.md) por canais (`AppCommand` para a fonte, `SerialEvent` de volta). Não lê o relógio:
o instante atual (`now`) entra por parâmetro.

## Tipos e constantes

| Item | O que é |
|---|---|
| `UPDATE_MS_RANGE` | Faixa aceita para o intervalo de leitura: de 100 a 1000 ms |
| `PORT_REFRESH_INTERVAL` | Intervalo mínimo entre duas consultas à lista de portas: 1 s |
| `ConnectionState` | O ponto em que a conexão está: `Disconnected`, `Connecting`, `Connected` ou `Error(mensagem)` (veja o [diagrama de estados](../arquitetura.md#estado-da-conexão)) |
| `SetpointEditor` | A tensão e a corrente em edição (`vset_input`, `cset_input`), mais três marcas privadas: há alteração pendente (`dirty`) e se o usuário está com o foco ou arrastando o campo de tensão (`editing_v`) ou de corrente (`editing_c`) |
| `OutputClock` | O tempo com a saída ligada (privado): o instante em que ela ligou (`run_start`) e o valor congelado quando desligou (`frozen`) |
| `AppState` | O estado completo. Os campos são privados e só mudam pelos métodos abaixo |

## Funções

### `SetpointEditor::is_dirty`

| | |
|---|---|
| **Propósito** | Dizer se há uma edição de tensão/corrente ainda não enviada à fonte. |
| **Entradas** | nenhuma |
| **Saída** | `true` se há alteração pendente. |
| **Efeitos** | nenhum |

**Como funciona**: devolve a marca `dirty`. A interface usa para mostrar "não aplicado" e para colorir o
botão "Aplicar".

### `OutputClock::tick`

*(interna)*

| | |
|---|---|
| **Propósito** | Atualizar o relógio de saída conforme a saída esteja ligada ou não. |
| **Entradas** | `running`: a saída está ligada agora? `now`: o instante atual. |
| **Saída** | nada |
| **Efeitos** | Altera `run_start` e `frozen`. |

**Como funciona**: só reage a **mudanças**. Se a saída está ligada e o relógio estava parado, grava `now` como
início e zera o valor congelado. Se a saída está desligada e o relógio estava contando, congela o tempo
decorrido (`now - início`) e para de contar. Nos demais casos (continua ligada ou continua desligada) não
faz nada.

**Exemplo:** a saída liga em t0 → o relógio começa; em t0 + 5 s o tempo é 5 s. A saída desliga em t0 + 5 s →
o valor fica congelado em 5 s, e continua 5 s em t0 + 60 s. Se ligar de novo, volta a 0 e conta de novo.

### `OutputClock::elapsed`

*(interna)*

| | |
|---|---|
| **Propósito** | Obter o tempo de saída ligada. |
| **Entradas** | `now`: o instante atual. |
| **Saída** | A duração: se contando, `now - início`; se parado, o valor congelado. |
| **Efeitos** | nenhum |

**Como funciona**: usa `saturating_duration_since`, que nunca dá resultado negativo, mesmo que `now` seja
anterior ao início.

### `AppState::default`

| | |
|---|---|
| **Propósito** | Criar o estado inicial pelo trait `Default`. |
| **Entradas** | nenhuma |
| **Saída** | O mesmo que `AppState::new()`. |
| **Efeitos** | nenhum |

**Como funciona**: chama `new()`.

### `AppState::new`

| | |
|---|---|
| **Propósito** | Criar o estado inicial do programa. |
| **Entradas** | nenhuma |
| **Saída** | Um `AppState` desconectado, sem dados da fonte, com a memória M1 selecionada e o intervalo de leitura em 100 ms. |
| **Efeitos** | nenhum |

**Como funciona**: preenche cada campo com o valor inicial: conexão `Disconnected`, sem canais, lista de
portas vazia, setpoint zerado, `selected_profile = 1`, relógio parado, `update_ms = 100` e histórico vazio.

### `AppState::device`

| | |
|---|---|
| **Propósito** | Dar acesso ao último estado conhecido da fonte. |
| **Entradas** | nenhuma |
| **Saída** | Uma referência ao `DPSUpdate` acumulado (medições, setpoints, memórias, etc.). |
| **Efeitos** | nenhum |

**Como funciona**: devolve o campo interno. Os campos são `Option`: o que a fonte ainda não informou é `None`.

### `AppState::connection`

| | |
|---|---|
| **Propósito** | Dar acesso ao estado da conexão. |
| **Entradas** | nenhuma |
| **Saída** | Uma referência ao `ConnectionState`. |
| **Efeitos** | nenhum |

**Como funciona**: devolve o campo. A interface escolhe cor, texto e botão a partir dele.

### `AppState::is_connected`

| | |
|---|---|
| **Propósito** | Dizer se a conexão está estabelecida. |
| **Entradas** | nenhuma |
| **Saída** | `true` só em `Connected`. |
| **Efeitos** | nenhum |

**Como funciona**: compara o estado com `Connected`. A interface habilita Aplicar, Salvar e o botão de saída
apenas quando é verdadeiro.

### `AppState::is_idle`

| | |
|---|---|
| **Propósito** | Dizer se não há sessão em andamento. |
| **Entradas** | nenhuma |
| **Saída** | `true` em `Disconnected` ou `Error`. |
| **Efeitos** | nenhum |

**Como funciona**: compara o estado com os dois valores ociosos. Nesses estados a escolha de porta fica livre
e a lista de portas é atualizada.

### `AppState::output_on`

| | |
|---|---|
| **Propósito** | Dizer se a saída da fonte está ligada. |
| **Entradas** | nenhuma |
| **Saída** | `true` se a saída está ligada. |
| **Efeitos** | nenhum |

**Como funciona**: o campo da fonte se chama `output_opened` e significa "circuito **aberto**", ou seja,
saída **desligada**. Então a saída está ligada quando `output_opened == Some(false)`. Se a fonte ainda não
informou nada (`None`), o resultado é `false`.

### `AppState::available_ports`

| | |
|---|---|
| **Propósito** | Dar a lista das portas seriais encontradas. |
| **Entradas** | nenhuma |
| **Saída** | Uma fatia com os nomes das portas. |
| **Efeitos** | nenhum |

**Como funciona**: devolve a lista guardada, atualizada por `refresh_ports_if_due`.

### `AppState::selected_port`

| | |
|---|---|
| **Propósito** | Dar o nome da porta escolhida. |
| **Entradas** | nenhuma |
| **Saída** | O texto da porta (vazio se nenhuma). |
| **Efeitos** | nenhum |

**Como funciona**: devolve o campo.

### `AppState::selected_port_mut`

| | |
|---|---|
| **Propósito** | Deixar a interface trocar a porta escolhida. |
| **Entradas** | nenhuma |
| **Saída** | Uma referência mutável ao texto da porta. |
| **Efeitos** | A interface pode alterar a porta escolhida por essa referência. |

**Como funciona**: devolve `&mut String`, que é o que o seletor de opções do egui precisa
(`selectable_value`). É o único campo que a interface altera sem passar por uma regra.

### `AppState::setpoint`

| | |
|---|---|
| **Propósito** | Dar acesso de leitura ao editor de setpoint. |
| **Entradas** | nenhuma |
| **Saída** | Uma referência ao `SetpointEditor`. |
| **Efeitos** | nenhum |

**Como funciona**: devolve o campo. A interface lê `vset_input`, `cset_input` e `is_dirty()`.

### `AppState::setpoint_mut`

| | |
|---|---|
| **Propósito** | Deixar a interface editar os valores de tensão e corrente. |
| **Entradas** | nenhuma |
| **Saída** | Uma referência mutável ao `SetpointEditor`. |
| **Efeitos** | A interface altera `vset_input` e `cset_input` diretamente. |

**Como funciona**: os campos numéricos da tela (`DragValue`) editam um `&mut f32`, então a interface precisa
dessa referência. Editar os números **não** marca a edição como pendente: quem chama deve chamar
`mark_setpoint_edited` quando o campo mudar.

### `AppState::selected_profile`

| | |
|---|---|
| **Propósito** | Dar a memória (M1 a M6) selecionada. |
| **Entradas** | nenhuma |
| **Saída** | O número do grupo, de 1 a 6. |
| **Efeitos** | nenhum |

**Como funciona**: devolve o campo. É o alvo do botão "Salvar".

### `AppState::group_values`

| | |
|---|---|
| **Propósito** | Obter a tensão e a corrente salvas num grupo de memória. |
| **Entradas** | `n`: o grupo, de 1 a 6. |
| **Saída** | `Some((tensão, corrente))` ou `None` se `n` está fora de 1..6 ou a fonte ainda não informou os dois valores. |
| **Efeitos** | nenhum |

**Como funciona**: um `match` em `n` lê o par de campos do grupo (`gN_vset` e `gN_cset`) do estado da fonte.
O operador `?` faz o resultado ser `None` se qualquer um dos dois estiver ausente.

### `AppState::history`

| | |
|---|---|
| **Propósito** | Dar acesso ao histórico do gráfico. |
| **Entradas** | nenhuma |
| **Saída** | Uma referência ao `History`. |
| **Efeitos** | nenhum |

**Como funciona**: devolve o campo.

### `AppState::update_ms`

| | |
|---|---|
| **Propósito** | Dar o intervalo de leitura em milissegundos. |
| **Entradas** | nenhuma |
| **Saída** | O intervalo, entre 100 e 1000. |
| **Efeitos** | nenhum |

**Como funciona**: devolve o campo. O slider do gráfico mostra esse valor e, ao ser movido, chama
`set_update_ms`.

### `AppState::update_interval`

| | |
|---|---|
| **Propósito** | Dar o intervalo de leitura como `Duration`. |
| **Entradas** | nenhuma |
| **Saída** | O intervalo (`update_ms` milissegundos). |
| **Efeitos** | nenhum |

**Como funciona**: converte `update_ms` com `Duration::from_millis`.

### `AppState::current_elapsed`

| | |
|---|---|
| **Propósito** | Dar o tempo com a saída ligada, para mostrar na barra inferior. |
| **Entradas** | `now`: o instante atual. |
| **Saída** | A duração. |
| **Efeitos** | nenhum |

**Como funciona**: delega a `OutputClock::elapsed`.

### `AppState::refresh_ports_if_due`

| | |
|---|---|
| **Propósito** | Atualizar a lista de portas seriais, mas sem exagerar. |
| **Entradas** | `now`: o instante atual. |
| **Saída** | nada |
| **Efeitos** | Pode consultar o sistema pela lista de portas (E/S) e alterar a lista e a porta escolhida. |

**Como funciona**: só age **sem sessão** (`is_idle`). Se a última consulta foi há menos de
`PORT_REFRESH_INTERVAL` (1 s), não faz nada. Senão, pede a lista a `session::available_ports()`, guarda-a (por
`set_available_ports`) e anota `now` como a última consulta. A consulta existe nessa função, e não a cada
quadro, porque listar portas é E/S e a função roda a cada quadro da interface.

**Exemplo:** consulta feita em t = 0. Uma chamada em t = 0,3 s não consulta. Uma chamada em t = 1 s consulta de
novo. Uma porta recém-plugada pode demorar até 1 s para aparecer.

### `AppState::poll_events`

| | |
|---|---|
| **Propósito** | Processar o que a sessão serial avisou desde o último quadro. |
| **Entradas** | nenhuma |
| **Saída** | nada |
| **Efeitos** | Esvazia o canal de eventos e altera o estado, a conexão e os campos de edição. |

**Como funciona**: se não há sessão, não faz nada. Senão, recolhe de uma vez todos os eventos do canal e trata
cada um, na ordem:

- `Update(atualização)`: junta a atualização ao estado (`merge`) e sincroniza os campos de edição
  (`sync_edit_buffers`);
- `Connected`: o estado da conexão vira `Connected`;
- `Error(mensagem)`: o estado vira `Error(mensagem)` e a sessão é encerrada (`end_session`), o que apaga os
  canais e zera a telemetria.

### `AppState::tick`

| | |
|---|---|
| **Propósito** | Avançar o relógio de saída e gravar uma amostra no histórico. É chamada a cada quadro. |
| **Entradas** | `now`: o instante atual. |
| **Saída** | nada |
| **Efeitos** | Altera o relógio de saída e, se conectado, pode acrescentar uma amostra ao histórico. |

**Como funciona**: (1) avança o relógio com `OutputClock::tick(output_on(), now)`. (2) Se **não** está
conectado, para aqui: sem conexão o histórico não recebe nada. (3) Monta uma amostra com a tensão, a corrente
e a potência da saída (`0` para o que a fonte ainda não informou) e a oferece ao histórico, que a grava só se
já passou `update_interval`.

### `AppState::connect`

| | |
|---|---|
| **Propósito** | Iniciar a conexão com a fonte na porta escolhida. |
| **Entradas** | `notify`: função que a sessão chamará quando houver evento novo (a `gui` passa um pedido de redesenho). |
| **Saída** | nada |
| **Efeitos** | Cria a thread da sessão, muda a conexão para `Connecting` e zera a telemetria. |

**Como funciona**: se nenhuma porta foi escolhida, vai direto para
`Error("Nenhum dispositivo/porta serial disponível para conexão.")` e nada mais acontece. Senão, chama
`spawn_session` com a porta e o intervalo atual, guarda os dois canais, muda para `Connecting` e zera a
telemetria (estado da fonte, relógio e histórico). A conclusão chega depois, como evento.

### `AppState::disconnect`

| | |
|---|---|
| **Propósito** | Encerrar a sessão, ou cancelar uma conexão em andamento. |
| **Entradas** | nenhuma |
| **Saída** | nada |
| **Efeitos** | Envia `Disconnect` à sessão, muda a conexão para `Disconnected` e zera a telemetria. |

**Como funciona**: envia `AppCommand::Disconnect` (a thread bloqueia a sessão e termina), muda o estado para
`Disconnected` e chama `end_session`, que descarta os canais e zera a telemetria. A lista de portas é
atualizada logo em seguida.

### `AppState::mark_setpoint_edited`

| | |
|---|---|
| **Propósito** | Registrar que o usuário mexeu num campo de tensão ou corrente. |
| **Entradas** | nenhuma |
| **Saída** | nada |
| **Efeitos** | Marca a edição como pendente. |

**Como funciona**: põe `dirty = true`. Enquanto estiver assim, `sync_edit_buffers` não sobrescreve o que o
usuário digitou.

### `AppState::set_editing`

| | |
|---|---|
| **Propósito** | Informar quais campos estão em foco ou sendo arrastados. |
| **Entradas** | `voltage`: o campo de tensão está sendo editado? `current`: o de corrente? |
| **Saída** | nada |
| **Efeitos** | Atualiza as marcas `editing_v` e `editing_c`. |

**Como funciona**: só guarda os dois booleanos, que a interface recalcula a cada quadro. Um campo que está
sendo editado nunca é sobrescrito por `sync_edit_buffers`, mesmo sem edição pendente.

### `AppState::apply_setpoints`

| | |
|---|---|
| **Propósito** | Enviar à fonte a tensão e a corrente editadas. |
| **Entradas** | nenhuma (usa os campos do setpoint) |
| **Saída** | nada |
| **Efeitos** | **Envia** `SetVoltage` e `SetCurrent` à fonte e encerra a edição pendente. |

**Como funciona**: envia os dois comandos, nessa ordem, com os valores de `vset_input` e `cset_input`, e
zera `dirty`. Sem sessão, os comandos são descartados (os botões ficam desabilitados fora de `Connected`).

### `AppState::discard_setpoint_edits`

| | |
|---|---|
| **Propósito** | Desistir da edição e voltar aos valores atuais da fonte. |
| **Entradas** | nenhuma |
| **Saída** | nada |
| **Efeitos** | Zera a marca de pendência e copia os valores da fonte para os campos. **Não** envia nada. |

**Como funciona**: zera `dirty` e chama `sync_edit_buffers`, que, agora sem pendência, copia o `vset` e o
`cset` da fonte para os campos.

### `AppState::load_profile`

| | |
|---|---|
| **Propósito** | Carregar nos campos de edição os valores de uma memória (M1 a M6), para o usuário conferir. |
| **Entradas** | `n`: o grupo, de 1 a 6. |
| **Saída** | nada |
| **Efeitos** | Altera os campos de edição, a marca de pendência e a memória selecionada. **Não envia nada à fonte.** |

**Como funciona**: se `group_values(n)` tem os dois valores, copia-os para `vset_input` e `cset_input` e
marca a edição como pendente. Em qualquer caso (mesmo sem dados), `selected_profile` passa a ser `n`, porque é
o alvo do "Salvar". O envio à fonte só acontece quando o usuário clica em "Aplicar".

**Exemplo:** M2 guarda 12,0 V e 1,5 A. `load_profile(2)` deixa os campos em 12,0 e 1,5, mostra "não aplicado" e
**não** manda nenhum comando. Só `apply_setpoints` os envia.

### `AppState::save_profile`

| | |
|---|---|
| **Propósito** | Gravar na fonte os valores dos campos como uma memória. |
| **Entradas** | `n`: o grupo, de 1 a 6. |
| **Saída** | nada |
| **Efeitos** | **Envia** `SaveProfile` à fonte e muda a memória selecionada. |

**Como funciona**: envia `SaveProfile(n, vset_input, cset_input)` e põe `selected_profile = n`. Não altera a
marca de pendência, porque gravar uma memória não muda o setpoint ativo.

### `AppState::toggle_output`

| | |
|---|---|
| **Propósito** | Ligar a saída se estiver desligada, ou desligar se estiver ligada. |
| **Entradas** | nenhuma |
| **Saída** | nada |
| **Efeitos** | **Envia** `EnableOutput` à fonte. |

**Como funciona**: envia `EnableOutput(!output_on())`, isto é, o contrário do estado atual. Se a fonte ainda
não informou o estado, a saída é tratada como desligada e o comando pede para ligar.

### `AppState::set_update_ms`

| | |
|---|---|
| **Propósito** | Mudar o intervalo de leitura. |
| **Entradas** | `ms`: o novo intervalo, em milissegundos. |
| **Saída** | nada |
| **Efeitos** | Altera `update_ms` e **envia** `SetPollInterval` à sessão. |

**Como funciona**: limita `ms` à faixa `UPDATE_MS_RANGE` (100 a 1000), guarda o resultado e avisa a sessão do
novo intervalo, sem reconectar. O mesmo valor passa a valer para as amostras do histórico.

**Exemplo:** `set_update_ms(5000)` deixa o intervalo em 1000 ms e envia `SetPollInterval(1000 ms)`;
`set_update_ms(50)` deixa em 100 ms.

### `AppState::clear_history`

| | |
|---|---|
| **Propósito** | Apagar o histórico do gráfico. |
| **Entradas** | nenhuma |
| **Saída** | nada |
| **Efeitos** | Esvazia o histórico. Não envia nada à fonte. |

**Como funciona**: chama `History::clear`. É o botão "Limpar".

### `AppState::send`

*(interna)*

| | |
|---|---|
| **Propósito** | Colocar um comando no canal da sessão. |
| **Entradas** | `cmd`: o `AppCommand`. |
| **Saída** | nada |
| **Efeitos** | Envia o comando pelo canal, se houver sessão. |

**Como funciona**: se existe o emissor de comandos, envia; senão, **descarta em silêncio**. Erros de envio
(a thread já terminou) também são ignorados. É o único ponto por onde as regras falam com a fonte.

### `AppState::end_session`

*(interna)*

| | |
|---|---|
| **Propósito** | Limpar tudo o que pertence a uma sessão que terminou. |
| **Entradas** | nenhuma |
| **Saída** | nada |
| **Efeitos** | Descarta os canais, esquece a última consulta de portas e zera a telemetria. |

**Como funciona**: põe os dois canais em `None`, zera `last_port_refresh` (a lista de portas é atualizada no
quadro seguinte) e chama `reset_telemetry`. Não muda o `ConnectionState`: quem chama define o estado final
(`Disconnected` ou `Error`).

### `AppState::reset_telemetry`

*(interna)*

| | |
|---|---|
| **Propósito** | Apagar os dados de uma sessão anterior. |
| **Entradas** | nenhuma |
| **Saída** | nada |
| **Efeitos** | Zera o estado da fonte, o relógio de saída e o histórico. |

**Como funciona**: troca o estado da fonte por um `DPSUpdate` vazio, o relógio por um novo (parado, em zero) e
esvazia o histórico. Acontece ao conectar, desconectar e quando há erro.

### `AppState::set_available_ports`

*(interna)*

| | |
|---|---|
| **Propósito** | Guardar a lista de portas e manter a escolha consistente com ela. |
| **Entradas** | `ports`: a lista de nomes encontrada. |
| **Saída** | nada |
| **Efeitos** | Altera a lista e, se preciso, a porta escolhida. |

**Como funciona**: substitui a lista. Se a porta escolhida **não está mais** nela, a escolha passa a ser a
primeira da lista, ou vazia se a lista está vazia. Assim nunca sobra uma porta que não existe selecionada.

**Exemplo:** a porta escolhida é `/dev/ttyACM1` e a lista nova é `["/dev/ttyACM0"]` → a escolha vira
`/dev/ttyACM0`. Com a lista vazia, a escolha vira `""`.

### `AppState::merge`

*(interna)*

| | |
|---|---|
| **Propósito** | Juntar uma atualização parcial da fonte ao estado conhecido. |
| **Entradas** | `new`: o `DPSUpdate` recebido. |
| **Saída** | nada |
| **Efeitos** | Altera o estado da fonte guardado. |

**Como funciona**: a fonte envia pacotes diferentes (alguns trazem só a temperatura, outros trazem tudo), então
cada `DPSUpdate` só tem **alguns** campos preenchidos. Uma macro percorre os 37 campos: se o campo veio
(`Some`), ele substitui o guardado; se não veio (`None`), o valor antigo é mantido. Assim a tela nunca "perde"
um número só porque o pacote seguinte não o trazia.

**Exemplo:** o estado tem `temperature = 34,0`. Chega uma atualização só com `output_voltage = 5,0`. Depois do
`merge`, o estado tem `output_voltage = 5,0` **e** continua com `temperature = 34,0`.

### `AppState::sync_edit_buffers`

*(interna)*

| | |
|---|---|
| **Propósito** | Fazer os campos de tensão e corrente acompanharem a fonte, sem atropelar o que o usuário está digitando. |
| **Entradas** | nenhuma |
| **Saída** | nada |
| **Efeitos** | Pode copiar `vset` e `cset` da fonte para os campos de edição. |

**Como funciona**: (1) se há edição pendente (`dirty`), **não faz nada**: o que o usuário digitou fica. (2) Senão,
copia o `vset` da fonte para o campo de tensão, **se** o campo não está em foco ou sendo arrastado, e a mesma
coisa para o `cset` e o campo de corrente. Só copia quando a fonte já informou o valor.

**Exemplo:** o usuário digitou 5,0 V e há edição pendente. Chega uma atualização com `vset = 3,3`. O campo
continua em 5,0. Depois de "Desfazer" (ou "Aplicar" e a fonte confirmar), o próximo `sync_edit_buffers` põe
o campo em 3,3.

## Testes

Nove testes cobrem o módulo, todos sem janela e sem porta serial. Eles usam um `AppState` ligado a canais de
teste (`with_test_session`, que existe só nos testes), de onde enviam eventos e onde recolhem os comandos
emitidos:

- a edição pendente é preservada diante de uma atualização, e "Desfazer" sincroniza com a fonte;
- carregar uma memória preenche os campos e **não envia comandos**;
- Aplicar e Salvar enviam os valores dos campos;
- o merge parcial mantém os campos ausentes;
- o relógio de saída conta, congela e zera;
- o histórico só grava quando conectado;
- um erro da sessão leva ao estado ocioso e limpa o histórico;
- a porta escolhida que some da lista é substituída;
- o intervalo de leitura é limitado à faixa e enviado à sessão.
