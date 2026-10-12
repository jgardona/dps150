# `logics::memory_setup` — campos de setup das memórias

Arquivo: [`src/logics/memory_setup.rs`](../../src/logics/memory_setup.rs) · Visão geral: [arquitetura](../arquitetura.md)

## Papel

É a regra pura do **quadro de setup**: define os seis campos (Vset, Iset, OVP, OCP, OPP e OTP), o máximo,
a unidade e as casas decimais de cada um, guarda os valores em edição com a marca de "alterado, não salvo" e
confere se eles podem ser enviados à fonte.

## O que não faz

Não desenha (a tela é de [`gui::panels`](gui-panels.md)), não envia nada à fonte (isso é de
[`AppState`](logics-app.md) e da [sessão](logics-session.md)) e não depende do egui. Por isso as regras
testam-se sem janela e sem aparelho.

## Constantes

Os máximos de cada campo. São a única fonte desses números: a validação e os campos da interface (inclusive os
campos V e A do card Setpoint) usam estas constantes.

| Constante | Valor | Campo |
|---|---|---|
| `VSET_MAX` | 36,00 V | Vset |
| `ISET_MAX` | 8,200 A | Iset |
| `OVP_MAX` | 36,10 V | OVP |
| `OCP_MAX` | 8,200 A | OCP |
| `OPP_MAX` | 295,20 W | OPP |
| `OTP_MAX` | 99 °C | OTP |

O mínimo de todos é 0.

## Tipos

| Tipo | O que é |
|---|---|
| `SetupField` | Um dos seis campos, na ordem do visor da fonte (`SetupField::ALL`) |
| `SetupValues` | Os seis valores já validados, mais o grupo (1 a 6) de destino; é o que vai para a sessão |
| `SetupError` | Por que não pode salvar: `Missing(campo)`, `AboveMax(campo)`, `VsetAboveOvp` ou `IsetAboveOcp` |
| `MemorySetup` | Os seis valores em edição (`None` = a fonte ainda não informou) e a marca de pendência |

OVP, OCP, OPP e OTP valem para o **aparelho todo**; só Vset e Iset são de cada memória.

## Funções

### `SetupField::label`

| | |
|---|---|
| **Propósito** | Dar o nome do campo para a tela. |
| **Entradas** | nenhuma (usa o campo) |
| **Saída** | Texto fixo: "Vset", "Iset", "OVP", "OCP", "OPP" ou "OTP". |
| **Efeitos** | nenhum |

**Como funciona**: um `match` sobre o campo.

### `SetupField::unit`

| | |
|---|---|
| **Propósito** | Dar a unidade do campo. |
| **Entradas** | nenhuma (usa o campo) |
| **Saída** | "V" (Vset, OVP), "A" (Iset, OCP), "W" (OPP) ou "°C" (OTP). |
| **Efeitos** | nenhum |

**Como funciona**: um `match` sobre o campo.

### `SetupField::max`

| | |
|---|---|
| **Propósito** | Dar o valor máximo aceito pelo campo. |
| **Entradas** | nenhuma (usa o campo) |
| **Saída** | Uma das constantes `VSET_MAX`…`OTP_MAX`. |
| **Efeitos** | nenhum |

**Como funciona**: um `match` que devolve a constante do campo. É a única ligação entre o campo e o seu limite.

### `SetupField::decimals`

| | |
|---|---|
| **Propósito** | Dar quantas casas decimais o campo mostra e confere. |
| **Entradas** | nenhuma (usa o campo) |
| **Saída** | 2 (Vset, OVP, OPP), 3 (Iset, OCP) ou 0 (OTP). |
| **Efeitos** | nenhum |

**Como funciona**: um `match` sobre o campo; os valores são os do visor da fonte.

### `SetupField::is_global`

| | |
|---|---|
| **Propósito** | Dizer se o campo vale para o aparelho todo, e não só para a memória. |
| **Entradas** | nenhuma (usa o campo) |
| **Saída** | `true` para OVP, OCP, OPP e OTP; `false` para Vset e Iset. |
| **Efeitos** | nenhum |

**Como funciona**: `!matches!` sobre Vset e Iset. A tela usa isso para marcar os campos com `*` e avisar.

### `SetupField::tolerance`

| | |
|---|---|
| **Propósito** | Dar a diferença abaixo da qual dois valores são a mesma leitura. |
| **Entradas** | nenhuma (usa o campo) |
| **Saída** | Meia unidade da última casa decimal do campo. |
| **Efeitos** | nenhum |

**Como funciona**: calcula `0,5 / 10^casas`. A fonte guarda e devolve os valores arredondados; sem essa folga, uma gravação correta pareceria diferente.

**Exemplo:** Vset tem 2 casas, então a tolerância é 0,005 V; Iset tem 3, então 0,0005 A; OTP tem 0, então 0,5 °C.

### `SetupField::index`

*(interna)*

| | |
|---|---|
| **Propósito** | Dar a posição do campo no vetor de valores de `MemorySetup`. |
| **Entradas** | nenhuma (usa o campo) |
| **Saída** | Um número de 0 (Vset) a 5 (OTP). |
| **Efeitos** | nenhum |

**Como funciona**: um `match` sobre o campo; a ordem é a de `SetupField::ALL`.

### `SetupValues::get`

| | |
|---|---|
| **Propósito** | Ler um dos seis valores pelo campo. |
| **Entradas** | `field`: o campo. |
| **Saída** | O valor `f32` do campo. |
| **Efeitos** | nenhum |

**Como funciona**: um `match` que devolve o membro correspondente. Serve para comparar o que foi enviado com o que a fonte devolveu.

### `SetupError::field`

| | |
|---|---|
| **Propósito** | Dizer qual campo marcar na tela por causa do erro. |
| **Entradas** | nenhuma (usa o erro) |
| **Saída** | `Some(campo)`. Para `VsetAboveOvp` é Vset e para `IsetAboveOcp` é Iset. |
| **Efeitos** | nenhum |

**Como funciona**: `Missing` e `AboveMax` já carregam o campo; os dois erros cruzados apontam para o campo da esquerda.

### `SetupError::message`

| | |
|---|---|
| **Propósito** | Montar a mensagem do erro em pt-BR. |
| **Entradas** | nenhuma (usa o erro) |
| **Saída** | Um `String`. |
| **Efeitos** | nenhum |

**Como funciona**: `Missing` diz "OVP: valor ausente". `AboveMax` diz o máximo com as casas e a unidade do campo. Os erros cruzados dizem "Vset não pode passar do OVP" e "Iset não pode passar do OCP".

**Exemplo:** `AboveMax(Ovp)` → "OVP: máximo de 36.10 V"; `AboveMax(Otp)` → "OTP: máximo de 99 °C".

### `MemorySetup::get`

| | |
|---|---|
| **Propósito** | Ler o valor em edição de um campo. |
| **Entradas** | `field`: o campo. |
| **Saída** | `Some(valor)` ou `None` se a fonte ainda não informou. |
| **Efeitos** | nenhum |

**Como funciona**: lê a posição do campo no vetor.

### `MemorySetup::set`

| | |
|---|---|
| **Propósito** | Registrar uma edição do usuário. |
| **Entradas** | `field`: o campo. `value`: o novo valor. |
| **Saída** | nada |
| **Efeitos** | Grava o valor e marca a edição como pendente. |

**Como funciona**: grava `Some(value)` e liga `dirty`. A partir daí o quadro deixa de acompanhar a fonte.

### `MemorySetup::load`

| | |
|---|---|
| **Propósito** | Carregar um valor vindo da fonte. |
| **Entradas** | `field`: o campo. `value`: o valor lido, ou `None`. |
| **Saída** | nada |
| **Efeitos** | Troca o valor sem marcar pendência. |

**Como funciona**: grava o valor e **não** mexe em `dirty`. É o caminho dos dados da fonte, para eles não parecerem uma edição do usuário.

### `MemorySetup::is_dirty`

| | |
|---|---|
| **Propósito** | Informar se há edição pendente. |
| **Entradas** | nenhuma |
| **Saída** | `true` se o usuário alterou algo e ainda não salvou nem descartou. |
| **Efeitos** | nenhum |

**Como funciona**: devolve `dirty`.

### `MemorySetup::clear_dirty`

| | |
|---|---|
| **Propósito** | Encerrar a pendência. |
| **Entradas** | nenhuma |
| **Saída** | nada |
| **Efeitos** | A edição deixa de estar pendente. |

**Como funciona**: desliga `dirty`. Os valores ficam como estão.

### `MemorySetup::validate`

| | |
|---|---|
| **Propósito** | Conferir se os seis valores podem ser enviados à fonte. |
| **Entradas** | `group`: a memória de destino (1 a 6; o chamador garante). |
| **Saída** | `Ok(SetupValues)` com os seis valores e o grupo, ou `Err` com **todos** os erros encontrados. |
| **Efeitos** | nenhum (só confere) |

**Como funciona**: para cada campo, na ordem do visor: ausente ou que não é número (`NaN`, infinito) vira `Missing`; negativo ou acima do máximo vira `AboveMax`. Depois, só se os dois campos envolvidos são válidos, confere Vset ≤ OVP e Iset ≤ OCP (`VsetAboveOvp`, `IsetAboveOcp`). Devolver tudo de uma vez deixa a tela marcar todos os campos errados sem o usuário descobri-los um a um.

**Exemplo:** Vset 12,00 com OVP 10,00 e o resto válido → `Err([VsetAboveOvp])`. Vset 36,50 → `Err([AboveMax(Vset)])`. Vset igual ao OVP (10,00 e 10,00) é aceito.

## Testes

Os testes do módulo cobrem: a tabela de máximos, unidades e casas; as mensagens; a pendência (`set` marca, `load` não); cada máximo exato aceito e o valor logo acima recusado; zero aceito; negativo, `NaN` e ausente recusados; os dois erros cruzados (e a igualdade aceita); vários erros de uma vez; e a tolerância por casa decimal.
