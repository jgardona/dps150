# Pesquisa: campos de configuração das memórias

**Feature**: [spec.md](./spec.md) | **Plan**: [plan.md](./plan.md)

## R1 — O que a fonte guarda por memória

**Decisão**: por memória, só Vset e Iset (type_id 197–208). OVP, OCP, OPP e OTP são do aparelho (209–212).
**Motivo**: é o que o protocolo do projeto lê (`GET ALL`: offsets 28–75 são os seis grupos, 76–91 as
proteções) e grava (`group_voltage_id`/`group_current_id` e as constantes `OVP`…`OTP`).
**Alternativas**: proteções por memória — sem comando conhecido; fica como verificação com a fonte real
(quickstart, passo 6). Se existir, muda só o `SaveMemorySetup` na sessão.

## R2 — Como gravar as proteções

**Decisão**: `CMD_SET` (0xb1) com o type_id da proteção e um `f32` little-endian, igual a Vset/Iset, por
`DPS150::set_float_value(commands::OVP, v)`. Nenhum código de protocolo novo.
**Motivo**: o payload de leitura das proteções é `f32` (`read_float`), e o formato de escrita do aparelho
é o mesmo para todas as grandezas float. OTP também é `f32` na leitura, então é gravada como `f32`.
**Risco**: não confirmado em aparelho real → marcado **experimental** (código e README) e coberto por
roteiro manual e pela conferência de leitura (R5).

## R3 — Ordem das escritas

**Decisão**: gravar OVP, OCP, OPP, OTP e depois, para o grupo, a tensão e a corrente; terminar com um
`GET ALL`, com 100 ms entre pacotes (como a inicialização já faz).
**Motivo**: o grupo de memória não altera a saída atual, então não depende da ordem; as proteções vêm
primeiro para a fonte já ter os limites quando o grupo for usado. O intervalo de 100 ms é o que o
projeto já usa entre comandos seguidos.
**Alternativas**: tudo sem pausa — pode perder pacotes; a fonte tem buffer pequeno.

## R4 — Onde fica a regra de validação

**Decisão**: novo módulo `src/logics/memory_setup.rs`, puro. `AppState` só o usa.
**Motivo**: Princípio II (lógica fora do desenho, testável); `app.rs` já é o maior módulo.
**Alternativas**: dentro de `app.rs` (cresce demais e mistura com canais); dentro de `gui` (viola a
separação).

## R5 — Conferir a gravação (FR-011)

**Decisão**: depois de enviar, o estado fica **"verificando"**. A próxima leitura completa (`GET ALL`, a que
traz as proteções) é comparada com o que foi enviado, com tolerância de meio passo da casa decimal de cada
campo (0,005 V, 0,0005 A, 0,005 W, 0,5 °C). Se bater: **"gravado"**. Se a 3ª leitura seguinte ainda não
bater: **"não confirmado"**. A leitura que chega no meio da gravação pode ser antiga, por isso 3 tentativas.
**Motivo**: o aparelho não responde a um SET com confirmação; só a leitura mostra o que ele guardou.
**Alternativas**: não conferir (viola FR-011); esperar um tempo fixo (frágil com `update_ms` até 1000 ms).

## R6 — Máximos e campos (FR-013)

**Decisão**: constantes nomeadas em `memory_setup` (`VSET_MAX = 36.00`, `ISET_MAX = 8.200`,
`OVP_MAX = 36.10`, `OCP_MAX = 8.200`, `OPP_MAX = 295.20`, `OTP_MAX = 99.0`) e casas decimais por campo.
O card "Setpoint" usa `VSET_MAX`/`ISET_MAX`.
**Motivo**: valores mágicos devem ser constantes (Princípio I); uma só fonte da verdade para a GUI e a
validação.

## R7 — Campo numérico e valor acima do máximo

**Decisão**: `egui::DragValue` com `range(0..=max)` e `clamp_existing_to_range(false)`: arrastar fica na
faixa, mas o que é digitado fica como digitado, e a validação marca o campo em vermelho se passar do
máximo. (Confirmado na documentação do egui 0.34.3, `drag_value.rs`.)
**Motivo**: o cenário US3-1 pede campo marcado como inválido; o corte silencioso esconderia o erro.
**Alternativas**: corte automático — mais simples, mas o usuário não vê o que aconteceu.

## R8 — Espaço na tela

**Decisão**: duas linhas de três campos num `card`; cada campo é rótulo pequeno em cima e valor embaixo.
**Motivo**: é o desenho do visor da fonte (print) e cabe nos ~340 × 130 px do espaço mostrado. A janela
tem 540 px de altura; se não couber, o botão de saída é empurrado — verificar no quickstart e, se for
preciso, aumentar `HEIGHT` em `controller.rs`.

## R9 — Documentação

**Decisão**: o módulo novo ganha `docs/modulos/logics-memory_setup.md`; funções novas/alteradas em
`app`, `session`, `panels` e `widgets` são atualizadas; a conferência `docs_coverage` falha se algo
faltar. A constituição recebe PATCH 1.0.2 (faixa de 30 V / 5 A deixa de ser débito; menção ao monitor
`tests` removida).
