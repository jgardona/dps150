# Modelo de dados: campos de configuração das memórias

**Feature**: [spec.md](./spec.md) | **Plan**: [plan.md](./plan.md)

## SetupField (enum, `logics::memory_setup`)

Os seis campos, na ordem do visor da fonte.

| Campo | Rótulo | Unidade | Máximo | Casas | Valor lido de |
|---|---|---|---|---|---|
| `Vset` | Vset | V | 36,00 | 2 | `g{n}_vset` |
| `Iset` | Iset | A | 8,200 | 3 | `g{n}_cset` |
| `Ovp` | OVP | V | 36,10 | 2 | `ovp` |
| `Ocp` | OCP | A | 8,200 | 3 | `ocp` |
| `Opp` | OPP | W | 295,20 | 2 | `opp` |
| `Otp` | OTP | °C | 99 | 0 | `otp` |

Mínimo de todos: 0. `Ovp`, `Ocp`, `Opp` e `Otp` valem para o aparelho (não por memória).

## MemorySetup (edição em curso)

| Campo | Tipo | Regra |
|---|---|---|
| `values` | seis `Option<f32>` | `None` = a fonte ainda não informou ("—") |
| `dirty` | bool | `true` desde a primeira alteração do usuário até salvar ou descartar |

Transições:

- **Sem pendência** (`dirty = false`): acompanha a fonte (memória selecionada + proteções).
- **Edição** → `dirty = true`: deixa de acompanhar a fonte.
- **Salvar válido** → envia, `dirty = false`, estado "verificando".
- **Descartar** → `dirty = false` e volta a acompanhar a fonte.
- **Trocar de memória** com `dirty = true` → pede confirmação (`pending_switch`); sem pendência, troca direto.

## SetupError

| Variante | Quando | Mensagem (pt-BR) |
|---|---|---|
| `Missing(field)` | valor `None` ou não finito | "<campo>: valor ausente" |
| `AboveMax(field)` | valor > máximo, ou negativo | "<campo>: máximo de <máx> <un>" |
| `VsetAboveOvp` | Vset > OVP | "Vset não pode passar do OVP" |
| `IsetAboveOcp` | Iset > OCP | "Iset não pode passar do OCP" |

`validate()` devolve `Ok(SetupValues)` ou todos os erros de uma vez (um por campo, para marcá-los).

## SetupValues (o que é enviado)

`group: u8` (1..6), `vset`, `iset`, `ovp`, `ocp`, `opp`, `otp` (todos `f32`). É o conteúdo de
`AppCommand::SaveMemorySetup`.

## SaveStatus (em `AppState`)

| Estado | Significado | Sai quando |
|---|---|---|
| `Idle` | nada a conferir | — |
| `Verifying { expected, reads_left }` | enviado; esperando leitura | leitura confere → `Confirmed`; `reads_left` chega a 0 → `NotConfirmed` |
| `Confirmed` | a fonte guardou o que foi enviado | nova edição, troca de memória ou novo Salvar |
| `NotConfirmed` | a leitura não bateu | idem |

## Relações

`AppState` → um `MemorySetup`, um `selected_profile` (já existe), um `SaveStatus` e um
`pending_switch: Option<u8>`. Os valores base vêm de `DPSUpdate` (já existe).
