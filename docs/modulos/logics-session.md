# `logics::session` — sessão serial

Arquivo: [`src/logics/session.rs`](../../src/logics/session.rs) · Visão geral: [arquitetura](../arquitetura.md)

## Papel

É a conversa com a fonte pela porta serial, numa **thread própria**: abre a porta, inicializa a fonte, executa
os comandos que a interface pede, lê o que a fonte responde e avisa a interface. Fica isolada numa thread
porque a inicialização espera 2 s e a leitura da porta pode bloquear: se isso rodasse na thread da janela,
a interface congelaria.

A interface e a thread conversam por dois canais (`mpsc`): comandos (`AppCommand`) de lá para cá e eventos
(`SerialEvent`) daqui para lá. O diagrama do ciclo de vida está na
[visão geral](../arquitetura.md#ciclo-de-vida-da-sessão-serial).

## O que não faz

Não guarda o estado da aplicação (isso é de [`AppState`](logics-app.md)), não decide regras (como a
edição pendente do setpoint) e não conhece a interface gráfica: para pedir o redesenho da tela, chama uma
função `notify` que recebe pronta.

## Tipos e constantes

| Item | O que é |
|---|---|
| `BAUD_RATE` | A velocidade da serial: 115200 baud |
| `AppCommand` | O que a interface pede à sessão: `SetVoltage(V)`, `SetCurrent(A)`, `EnableOutput(bool)`, `SaveMemorySetup(valores)` (as proteções e o Vset/Iset de uma memória), `SetPollInterval(intervalo)` e `Disconnect` |
| `SerialEvent` | O que a sessão avisa à interface: `Update(DPSUpdate)` (dados novos), `Connected` (a inicialização terminou) e `Error(mensagem)` |

## Funções

### `spawn_session`

| | |
|---|---|
| **Propósito** | Iniciar uma sessão com a fonte numa thread nova. |
| **Entradas** | `port_name`: o nome da porta (ex.: `/dev/ttyACM0`). `poll_interval`: o intervalo entre consultas à fonte. `notify`: função chamada sempre que há um evento novo para a interface. |
| **Saída** | Uma dupla: o emissor do canal de comandos (`Sender<AppCommand>`) e o receptor do canal de eventos (`Receiver<SerialEvent>`). |
| **Efeitos** | Cria uma thread, que abre a porta e fala com a fonte. |

**Como funciona**: cria os dois canais, move as pontas certas para a thread (o receptor de comandos e o
emissor de eventos, mais `port_name`, `poll_interval` e `notify`) e devolve à interface as outras duas
pontas. A função retorna na hora: o trabalho de abrir a porta acontece dentro da thread, e o resultado
chega como evento (`Connected` ou `Error`).

### `available_ports`

| | |
|---|---|
| **Propósito** | Listar as portas seriais que o sistema enxerga. |
| **Entradas** | nenhuma |
| **Saída** | Os nomes das portas (lista vazia se não houver ou se a consulta falhar). |
| **Efeitos** | Consulta o sistema (no Linux, a `libudev`); isso é E/S e pode demorar. |

**Como funciona**: pede a lista ao crate `serialport` e fica só com o nome de cada porta. Um erro da
consulta vira lista vazia. Como a consulta custa, `AppState::refresh_ports_if_due` a limita a uma vez por
segundo.

### `session_main`

*(interna)*

| | |
|---|---|
| **Propósito** | O corpo da thread: toda a vida de uma sessão, da abertura da porta até a desconexão. |
| **Entradas** | `port_name`: a porta. `poll_interval`: o intervalo entre consultas (pode mudar durante a sessão). `tx`: o canal de eventos. `cmd_rx`: o canal de comandos. `notify`: o aviso de redesenho. |
| **Saída** | nada (retorna quando a sessão termina) |
| **Efeitos** | Abre e escreve na porta serial; envia eventos à interface; chama `notify`. |

**Como funciona**:

1. **Abre a porta** a 115200 baud, com tempo limite de leitura de 50 ms. Se falhar, envia
   `SerialEvent::Error("Falha ao abrir {porta}: {erro}")`, chama `notify` e encerra a thread.
2. **Liga DTR e RTS** (sinais de controle da serial) e **espera 2 s** para a porta estabilizar.
3. **Inicializa a fonte**: escreve cada pacote de `DPS150::init_command()` com 100 ms entre eles. Isso inclui
   desligar a saída.
4. Envia `SerialEvent::Connected` e chama `notify`.
5. **Laço principal**, repetido até o `Disconnect`:
   - **Comandos**: esvazia o canal de comandos e executa cada um: `SetVoltage`/`SetCurrent` viram um `SET` de
     número; `EnableOutput` liga/desliga a saída; `SaveMemorySetup(valores)` escreve cada pacote de
     `memory_setup_packets` com 100 ms entre eles (proteções, Vset/Iset do grupo e uma releitura); `SetPollInterval` troca o intervalo do passo seguinte; `Disconnect` escreve o bloqueio de
     sessão (`lock_session`) e **encerra**.
   - **Leitura**: tenta ler da porta (espera até 50 ms). Os bytes lidos vão para
     `DPS150::push_serial_data`, e cada atualização devolvida vira um `SerialEvent::Update`. Se houve alguma,
     chama `notify` uma vez.
   - **Consulta periódica**: se já passou `poll_interval` desde a última, escreve `get_all()` e reinicia a
     contagem.

Erros de escrita na porta são ignorados de propósito (a próxima consulta tenta de novo). Como a leitura
espera até 50 ms, o laço dá uma volta a cada 50 ms no máximo, sem consumir CPU à toa.

**Exemplo:** ao conectar numa porta que não existe, `session_main` envia
`Error("Falha ao abrir /dev/ttyUSB9: …")`, chama `notify` e retorna; nenhuma inicialização acontece. A
interface recebe o erro em `poll_events` e mostra a mensagem na barra superior.

### `memory_setup_packets`

*(interna)*

| | |
|---|---|
| **Propósito** | Montar os pacotes que gravam o setup de uma memória, na ordem de envio. |
| **Entradas** | `power_supply`: o protocolo. `v`: os seis valores validados e o grupo. |
| **Saída** | A lista de pacotes (cada um é um `Vec<u8>`). |
| **Efeitos** | nenhum (só monta; quem escreve na porta é `session_main`) |

**Como funciona**: monta, nesta ordem: OVP, OCP, OPP e OTP (`set_float_value` com os type_ids 209 a 212), a tensão e a corrente do grupo (`group_voltage_id` e `group_current_id`; grupo sem id não gera esses dois) e, por fim, o `GET ALL`, que faz a interface ler de volta o que a fonte guardou. A escrita das proteções é **experimental**: ainda não foi confirmada em aparelho real.

**Exemplo:** grupo 2, OVP 36,0, OCP 2,5, OPP 295,2, OTP 60,0, Vset 5,0 e Iset 0,5 → sete pacotes: tipos 209, 210, 211, 212, 199, 200 e 255 (leitura).

## Testes

A montagem dos pacotes de setup tem dois testes (`pacotes_do_setup_seguem_a_ordem_do_contrato` e
`grupo_sem_type_id_nao_envia_vset_e_iset`). O resto do módulo não tem testes automáticos: depende de uma porta
serial real. Ele é validado pelo roteiro manual de paridade, com a fonte conectada (veja a
[especificação da refatoração](../../specs/001-modularize-logics-gui/spec.md#verificação-manual-de-paridade)) e, para o
setup, pelo roteiro da [feature 003](../../specs/003-memory-setup-fields/quickstart.md).
