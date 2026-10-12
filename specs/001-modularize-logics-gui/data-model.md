# Data Model: Separação do controlador em módulos `logics` e `gui`

**Feature**: [spec.md](./spec.md) | **Plan**: [plan.md](./plan.md) | **Date**: 2026-10-12

Este documento descreve as entidades de `logics`, os campos e as regras de cada uma. Os tipos já
existem em `src/controller.rs`. A refatoração só os move e define de quem é cada responsabilidade.
As assinaturas públicas estão em [contracts/logics-api.md](./contracts/logics-api.md).

---

## `DPSUpdate` — `logics::protocol` (sem mudança)

O estado da fonte. Uma mensagem pode trazer só parte dos campos; os demais vêm como `None`.

| Grupo | Campos |
|---|---|
| Medições | `output_voltage`, `output_current`, `output_power`, `input_voltage`, `temperature` |
| Setpoint ativo | `vset`, `cset` |
| Memórias | `g1_vset` … `g6_cset` |
| Proteções | `ovp`, `ocp`, `opp`, `otp`, `lvp`, `protection_state` |
| Estado | `cc_cv` (`"CC"`/`"CV"`), `output_opened` (`Some(true)` = saída **desligada**) |
| Acumuladores | `output_capacity` (Ah), `output_energy` (Wh) |
| Identificação | `model_name`, `firmware_version`, `hardware_version` |
| Outros | `brightness`, `volume`, `metering`, `upper_limit_voltage`, `upper_limit_current` |

**Regra (merge)**: ao receber um `DPSUpdate` parcial, cada campo `Some` substitui o valor guardado, e
cada campo `None` mantém o último valor conhecido. Hoje isso é o `merge_state`; o comportamento não
muda.

## `ConnectionState` — `logics::app`

```
Disconnected ──connect()──▶ Connecting ──evento Connected──▶ Connected
     ▲   ▲                      │                               │
     │   └──────disconnect()────┴───────────disconnect()────────┘
     │                          │                               │
     └── (porta vazia) ── Error(msg) ◀──── evento Error ────────┘
```

- `connect()` com a porta selecionada vazia vai direto para `Error("Nenhum dispositivo/porta serial
  disponível para conexão.")`.
- `Error` e `Disconnected` são estados **ociosos**: a lista de portas é atualizada e a escolha de
  porta fica liberada.
- Entrar em `Connecting`, sair por `disconnect()` ou receber `Error` zera a telemetria (`reset_telemetry`).

## `AppState` — `logics::app`

O estado da aplicação, sem nenhum tipo do egui.

| Campo | Tipo | Regra |
|---|---|---|
| `state` | `DPSUpdate` | Último estado conhecido da fonte (merge) |
| `connection` | `ConnectionState` | Veja a máquina de estados acima |
| `events` / `commands` | canais `mpsc` opcionais | Existem só durante a sessão |
| `available_ports` | `Vec<String>` | Atualizada no máximo a cada `PORT_REFRESH_INTERVAL` (1 s), só quando ocioso |
| `selected_port` | `String` | Se sumir da lista, passa a ser a primeira porta disponível, ou vazio |
| `last_port_refresh` | `Option<Instant>` | Controla o intervalo acima |
| `setpoint` | `SetpointEditor` | Veja abaixo |
| `selected_profile` | `u8` (1..=6) | Alvo do "Salvar"; começa em 1 |
| `clock` | `OutputClock` | Veja abaixo |
| `update_ms` | `u64` | Dentro de `UPDATE_MS_RANGE` (100..=1000); começa em 100 |
| `history` | `History` | Veja abaixo |

## `SetpointEditor` — `logics::app`

| Campo | Regra |
|---|---|
| `vset_input` (V) | Faixa aceita: 0..=30 (limite atual mantido, FR-007) |
| `cset_input` (A) | Faixa aceita: 0..=5 |
| `dirty` | `true` quando o usuário altera um campo ou carrega uma memória; volta a `false` em Aplicar ou Desfazer |
| `editing_v`, `editing_c` | Informados pela `gui` a cada quadro: o campo está em foco ou sendo arrastado |

**Regras**:
- **Sincronização**: depois de cada atualização da fonte, se `!dirty` e o campo não está sendo editado,
  o campo recebe o `vset`/`cset` da fonte.
- **Aplicar**: envia `SetVoltage(vset_input)` e `SetCurrent(cset_input)` e zera `dirty`.
- **Desfazer**: zera `dirty` e sincroniza na hora.
- **Carregar memória `n`**: se a memória tem os dois valores, eles vão para os campos e `dirty = true`.
  Em qualquer caso, `selected_profile = n`. **Nenhum comando é enviado.**
- **Salvar em `n`**: envia `SaveProfile(n, vset_input, cset_input)` e faz `selected_profile = n`.

## `OutputClock` — `logics::app`

| Campo | Regra |
|---|---|
| `run_start: Option<Instant>` | Momento em que a saída foi vista ligada |
| `frozen: Duration` | Último valor quando a saída desligou |

**Transições** (`tick(now)`, com `running = output_opened == Some(false)`):
- desligada → ligada: `run_start = now`, `frozen = 0`
- ligada → desligada: `frozen = now - run_start`, `run_start = None`
- `current_elapsed(now)`: `now - run_start` se ligada; `frozen` se desligada
- `reset_telemetry` zera os dois.

## `Sample` e `History` — `logics::history`

- `Sample { voltage, current, power }` (f32).
- `History`: fila com no máximo `HISTORY_LEN = 300` amostras, mais `last_sample: Option<Instant>`.
- **Amostragem** (`tick(now)`, só em `Connected`): grava uma amostra se ainda não houver nenhuma, ou se
  `now - last_sample >= update_ms`. Campos ausentes da fonte entram como 0.
- **Limite**: com a fila cheia, descarta a mais antiga antes de inserir.
- **Limpar**: esvazia a fila e zera `last_sample`. Também acontece em `reset_telemetry`.
- **Janela visível**: `update_ms × HISTORY_LEN`.

## `AxisScale` — `logics::scale`

- Entrada: `peak` (maior valor visível) e `min_step` (V: 0,1; A: 0,01; W: 0,1).
- `step` = menor valor em {1, 2, 5, 10} × 10ⁿ que seja ≥ `max(peak / CHART_Y_DIVS, min_step)`.
- `max = step × CHART_Y_DIVS` (4); `decimals = decimals_for(step / CHART_Y_SUBDIVS)` (4), até 4 casas.
- Exemplos para teste: pico 3,3 V → topo 4, 2 casas; pico 0 A → topo 0,04, 4 casas; pico 0 W → topo 0,4.

## Sessão serial — `logics::session`

**Comandos** (`AppCommand`, GUI → sessão): `SetVoltage(f32)`, `SetCurrent(f32)`, `EnableOutput(bool)`,
`SaveProfile(u8, f32, f32)`, `SetPollInterval(Duration)`, `Disconnect`.

**Eventos** (`SerialEvent`, sessão → GUI): `Update(Box<DPSUpdate>)`, `Connected`, `Error(String)`.

**Ciclo de vida** (sem mudança de comportamento):
1. abre a porta a 115200 baud; se falhar, envia `Error("Falha ao abrir …")`, notifica e encerra;
2. ativa DTR/RTS e espera 2 s;
3. envia a sequência de inicialização, com 100 ms entre comandos (inclui desligar a saída);
4. envia `Connected` e notifica;
5. em loop: aplica os comandos pendentes, lê a porta, repassa as atualizações (e notifica quando houver
   alguma) e envia `GET ALL` a cada `poll_interval`;
6. em `Disconnect`: envia o bloqueio de sessão e encerra.
