# `logics::protocol` — protocolo da fonte

Arquivo: [`src/logics/protocol.rs`](../../src/logics/protocol.rs) · Visão geral: [arquitetura](../arquitetura.md)

## Papel

É o "tradutor" entre os bytes da porta serial e o resto do programa:

- **Para a fonte**: monta os pacotes de comando (pedir dados, definir tensão, ligar a saída…) com o
  checksum correto.
- **Da fonte**: recebe os bytes que chegam, junta os pedaços, acha os pacotes válidos (mesmo com lixo no meio
  ou pacotes partidos em duas leituras) e os converte em `DPSUpdate`.

O **formato do pacote** e a **tabela de campos** (`type_id`) já estão descritos na seção
[Protocolo do README](../../README.md#protocolo); este documento não os repete e explica só as funções.

## O que não faz

Não abre a porta serial, não cria threads e não conhece a interface. Só transforma bytes em bytes e bytes em
`DPSUpdate`. Quem fala com a porta é [`logics::session`](logics-session.md).

## Tipos e constantes

| Item | O que é |
|---|---|
| `commands` | Módulo com as constantes do protocolo: cabeçalhos (`HEADER_INPUT` 0xF0 e `HEADER_OUTPUT` 0xF1), comandos (`CMD_GET`, `CMD_SET`, `CMD_BAUD`, `CMD_SESSION`), os `type_id` dos campos, as velocidades da serial e o limite do buffer (`BUFFER_MAX_SIZE` = 1024) |
| `DPSUpdate` | Uma atualização vinda da fonte. **Todos os campos são `Option`**: um pacote traz só parte deles, e o que não veio fica `None` |
| `DPS150` | O objeto do protocolo. Guarda só o buffer de bytes recebidos que ainda não formaram um pacote |
| `PROTECTION_STATE_LABELS` | Nomes do estado de proteção, pelo índice do byte: "", OVP, OCP, OPP, OTP, LVP, REP |
| `CC_CV_LABELS` | `"CC"` (índice 0) e `"CV"` (índice 1) |

## Funções

### `DPS150::default`

| | |
|---|---|
| **Propósito** | Criar o objeto do protocolo pelo trait `Default`. |
| **Entradas** | nenhuma |
| **Saída** | Um `DPS150` com o buffer vazio. |
| **Efeitos** | nenhum |

**Como funciona**: chama `DPS150::new()`.

### `DPS150::new`

| | |
|---|---|
| **Propósito** | Criar o objeto do protocolo. |
| **Entradas** | nenhuma |
| **Saída** | Um `DPS150` com o buffer de recepção vazio. |
| **Efeitos** | nenhum |

**Como funciona**: só inicializa o buffer com um vetor vazio.

### `DPS150::init_command`

| | |
|---|---|
| **Propósito** | Montar a sequência de comandos que prepara a fonte quando a conexão começa. |
| **Entradas** | nenhuma |
| **Saída** | Uma lista de 8 pacotes prontos para escrever na porta, na ordem. |
| **Efeitos** | nenhum (só monta os bytes; quem os envia é a sessão) |

**Como funciona**: monta, nesta ordem: (1) desbloquear a sessão, (2) fixar a velocidade em 115200 baud,
(3) pedir a temperatura, (4) pedir o modelo, (5) pedir a versão de hardware, (6) pedir a versão de firmware,
(7) **desligar a saída** (`OUTPUT_ENABLE = 0`) e (8) pedir todos os dados (`get_all`). O comando 7 é o motivo
de a conexão sempre desligar a saída da fonte.

### `DPS150::get_protection`

| | |
|---|---|
| **Propósito** | Montar o pedido do estado de proteção (qual proteção, se alguma, disparou). |
| **Entradas** | nenhuma |
| **Saída** | O pacote `GET` do campo de estado de proteção. |
| **Efeitos** | nenhum |

**Como funciona**: monta um `GET` para o campo `PROTECTION_STATE_DATA` com um byte de payload (0). A interface
atual não usa esta função: o estado de proteção já vem dentro da resposta de `get_all`.

### `DPS150::get_all`

| | |
|---|---|
| **Propósito** | Montar o pedido de **todos** os campos de uma vez. É a consulta periódica da sessão. |
| **Entradas** | nenhuma |
| **Saída** | O pacote `GET` do campo `ALL` (255). |
| **Efeitos** | nenhum |

**Como funciona**: monta um `GET` para `ALL` com um byte de payload (0). A fonte responde com um pacote
grande (type_id 255) que traz medições, setpoints, memórias, proteções, acumuladores e estado.

### `DPS150::lock_session`

| | |
|---|---|
| **Propósito** | Montar o comando que bloqueia a sessão, usado ao desconectar. |
| **Entradas** | nenhuma |
| **Saída** | O pacote `SESSION` com payload 0. |
| **Efeitos** | nenhum |

**Como funciona**: é o oposto do primeiro comando de `init_command` (`SESSION` com payload 1, que desbloqueia).

### `DPS150::group_voltage_id`

| | |
|---|---|
| **Propósito** | Descobrir qual campo guarda a **tensão** de um grupo de memória. |
| **Entradas** | `n`: o número do grupo, de 1 a 6. |
| **Saída** | `Some(type_id)` do campo (197, 199, 201, 203, 205 ou 207) ou `None` se `n` está fora de 1..6. |
| **Efeitos** | nenhum |

**Como funciona**: um `match` em `n` devolve a constante `GROUPn_VOLTAGE_SET` correspondente.

### `DPS150::group_current_id`

| | |
|---|---|
| **Propósito** | Descobrir qual campo guarda a **corrente** de um grupo de memória. |
| **Entradas** | `n`: o número do grupo, de 1 a 6. |
| **Saída** | `Some(type_id)` do campo (198, 200, 202, 204, 206 ou 208) ou `None` se `n` está fora de 1..6. |
| **Efeitos** | nenhum |

**Como funciona**: igual a `group_voltage_id`, com as constantes `GROUPn_CURRENT_SET`. A sessão usa as duas
para gravar o Vset/Iset de uma memória (`SaveMemorySetup`).

### `DPS150::enable_output`

| | |
|---|---|
| **Propósito** | Montar o comando que liga ou desliga a saída da fonte. |
| **Entradas** | `enable`: `true` para ligar, `false` para desligar. |
| **Saída** | O pacote `SET` do campo `OUTPUT_ENABLE`, com payload 1 (ligar) ou 0 (desligar). |
| **Efeitos** | nenhum |

**Como funciona**: converte o booleano em 1 ou 0 e monta o `SET` com `build_command`.

### `DPS150::set_float_value`

| | |
|---|---|
| **Propósito** | Montar um comando que grava um número real em um campo (tensão, corrente, uma memória…). |
| **Entradas** | `type_id`: o campo a gravar. `value`: o número. |
| **Saída** | O pacote `SET` com os 4 bytes do número. |
| **Efeitos** | nenhum |

**Como funciona**: converte `value` em 4 bytes **little-endian** (o byte menos significativo primeiro) e monta
o `SET` com `build_command`.

### `DPS150::build_command`

*(interna)*

| | |
|---|---|
| **Propósito** | Montar qualquer pacote de comando, com o tamanho e o checksum. |
| **Entradas** | `c1`: cabeçalho. `c2`: comando. `c3`: campo (`type_id`). `payload`: os bytes de dados. |
| **Saída** | O pacote completo: `[c1, c2, c3, tamanho, payload…, checksum]`. |
| **Efeitos** | nenhum |

**Como funciona**: copia os quatro primeiros bytes, em seguida o payload, e por fim o checksum. O checksum é
`(c3 + tamanho + soma dos bytes do payload) mod 256` (o módulo é `CHECKSUM_MODULUS`, 0x100). Só `c3`, o
tamanho e o payload entram na soma; o cabeçalho e o comando não.

**Exemplo:** ligar a saída é `build_command(0xF1, 0xB1, 219, &[1])`: o tamanho é 1 e o checksum é
`219 + 1 + 1 = 221`. O pacote é `[0xF1, 0xB1, 219, 1, 1, 221]`.

### `DPS150::push_serial_data`

| | |
|---|---|
| **Propósito** | Receber bytes novos da porta e devolver as atualizações completas que já deu para montar. |
| **Entradas** | `data`: os bytes que acabaram de ser lidos (qualquer tamanho). |
| **Saída** | Uma lista de `DPSUpdate`, possivelmente vazia. |
| **Efeitos** | Altera o buffer interno (acrescenta os bytes e remove os pacotes consumidos). |

**Como funciona**: acrescenta `data` ao fim do buffer e chama `process_buffer`. A leitura da porta não
respeita limites de pacote: pode chegar meio pacote, ou três de uma vez. É por isso que existe um buffer
persistente entre as chamadas.

### `DPS150::process_buffer`

*(interna)*

| | |
|---|---|
| **Propósito** | Achar pacotes válidos dentro do buffer, convertê-los e tirá-los do buffer. |
| **Entradas** | nenhuma (usa o buffer interno) |
| **Saída** | As atualizações obtidas nesta passada. |
| **Efeitos** | Remove do buffer os bytes de cada pacote aceito e o lixo que vinha antes dele; pode esvaziar o buffer. |

**Como funciona**: repete até não achar mais nada:

1. Com menos de 6 bytes, para (não cabe nem o menor pacote).
2. Procura, byte a byte, a sequência de cabeçalho `0xF0 0xA1` (`HEADER_INPUT` + `CMD_GET`).
3. Lê o `type_id` e o tamanho. Se o pacote ainda **não chegou inteiro**, devolve o que tem e espera a
   próxima leitura, sem mexer no buffer.
4. Calcula o checksum e compara com o último byte. Se **não bate**, ignora esse cabeçalho (era
   coincidência ou pacote corrompido) e continua procurando no byte seguinte.
5. Se bate, entrega o payload a `parse_data` e remove do buffer o lixo anterior mais o pacote inteiro; então
   volta ao passo 1 para ver se há outro.
6. Se nenhum pacote foi achado e o buffer passa de `BUFFER_MAX_SIZE` (1024 bytes), esvazia tudo: é lixo
   acumulado que nunca formou pacote, e o buffer não pode crescer sem limite.

**Exemplo:** chegam `[0x00, 0x13, 0x37]` e logo depois um pacote de 17 bytes (tipo 195, 12 bytes de
payload). O cabeçalho está no índice 3, o checksum bate, e saem 3 + 17 = 20 bytes do buffer: o lixo e o
pacote. Se, em vez disso, só os 7 primeiros bytes do pacote chegarem, o passo 3 vê que faltam 10 e devolve
lista vazia; quando os 10 restantes chegam, o pacote sai inteiro.

### `DPS150::parse_data`

*(interna)*

| | |
|---|---|
| **Propósito** | Converter o payload de um pacote válido em um `DPSUpdate`, conforme o `type_id`. |
| **Entradas** | `type_id`: o campo que a fonte enviou. `payload`: os bytes de dados do pacote. |
| **Saída** | `Some(DPSUpdate)` com os campos preenchidos, ou `None` se o `type_id` não é conhecido. |
| **Efeitos** | nenhum |

**Como funciona**: um `match` no `type_id` decide como ler o payload, sempre com `read_float` (números de 4
bytes) ou lendo bytes direto:

- campos simples (192 entrada, 193 Vset, 194 Iset, 196 temperatura, 217 capacidade, 218 energia,
  226 e 227 limites): um número no offset 0;
- **195**: três números seguidos, tensão no offset 0, corrente no 4 e potência no 8;
- **219**: um byte; **0 significa saída desligada** (`output_opened = true`, "circuito aberto");
- **220** e **221**: um byte que vira texto, pelo índice (proteção "", OVP… / "CC" ou "CV");
- **222–224**: texto (modelo, hardware, firmware);
- **255** (`ALL`): o pacote grande, lido em offsets fixos (veja a tabela no README). Os floats dos
  offsets 119, 123, 127, 131 e 135 existem, mas ainda não se sabe o que são.

Um campo só vira `Some` se o payload tem bytes suficientes; um payload curto deixa o campo `None` em vez de
dar erro. Um `type_id` desconhecido devolve `None`, e o pacote é descartado sem prejuízo.

**Exemplo:** `parse_data(195, payload)` com `payload` = 12 bytes, os 4 primeiros `[0x00, 0x00, 0xA0, 0x40]`
(5,0 em float), os 4 seguintes 0,25 e os 4 últimos 1,25, devolve um `DPSUpdate` com
`output_voltage = Some(5.0)`, `output_current = Some(0.25)` e `output_power = Some(1.25)`, e o resto `None`.

### `DPS150::read_float`

*(interna)*

| | |
|---|---|
| **Propósito** | Ler um número real de 4 bytes de dentro de um payload. |
| **Entradas** | `payload`: os bytes. `offset`: a posição do primeiro byte do número. |
| **Saída** | `Some(f32)` ou `None` se não há 4 bytes a partir de `offset`. |
| **Efeitos** | nenhum |

**Como funciona**: pega a fatia `payload[offset..offset + 4]`; se ela não existe (payload curto), devolve
`None` em vez de causar pânico. Se existe, converte os 4 bytes em `f32` pela ordem **little-endian**.

**Exemplo:** `[0x00, 0x00, 0xA0, 0x40]` em `offset` 0 → `Some(5.0)`, porque 5,0 em ponto flutuante de 32
bits é `0x40A00000` e o byte menos significativo vem primeiro. `read_float(&[1, 2, 3], 0)` → `None`.

## Testes

Seis testes cobrem este módulo: o comando de ligar a saída (`[0xF1, 0xB1, 219, 1, 1, 221]`), o comando de
gravar um número com seu checksum, a leitura de tensão, corrente e potência (tipo 195), o lixo antes do
cabeçalho, o pacote dividido em duas leituras e o checksum errado sendo descartado.
