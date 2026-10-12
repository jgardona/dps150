# Research: Separação do controlador em módulos `logics` e `gui`

**Feature**: [spec.md](./spec.md) | **Plan**: [plan.md](./plan.md) | **Date**: 2026-10-12

Não havia nenhum "NEEDS CLARIFICATION" no Technical Context. Este documento registra as decisões de
desenho que a refatoração precisa tomar e por que cada uma foi escolhida.

---

## R1. Como os dois binários compartilham `logics`

**Decision**: criar um alvo de biblioteca no próprio pacote (`src/lib.rs`, crate `dps150`) que
declara `pub mod logics;` e `pub mod gui;`. `src/controller.rs` (bin `controller`) e `src/tests.rs`
(bin `tests`) passam a usar `dps150::gui` e `dps150::logics` em vez de `mod dps150;`.

**Rationale**:
- Hoje cada binário inclui `dps150.rs` com `mod dps150;`. O protocolo é compilado duas vezes, e cada
  cópia gera avisos de "never used" para o que o outro binário usa. A referência atual é de 18 avisos
  no `controller` e 31 no `tests`, quase todos desse tipo.
- Numa biblioteca, itens `pub` não geram aviso de código morto. Isso zera esses avisos sem
  `#[allow]` e atende ao FR-011 e ao Princípio I.
- `cargo test` passa a rodar os testes unitários da biblioteca (FR-010) sem abrir janela.
- Continua sendo um único pacote, um único `Cargo.toml`, como diz a Assumption da spec. Não é um crate
  separado num workspace.

**Alternatives considered**:
- *Módulos só dentro do binário* (`mod logics; mod gui;` em `controller.rs`): o `tests` precisaria
  de `#[path = "logics/protocol.rs"] mod protocol;`. Isso é frágil e mantém a compilação duplicada
  e os avisos.
- *Workspace com crates `dps150-logics` e `dps150-gui`*: separação mais forte, mas a spec deixou
  isso fora do escopo e aumenta a configuração sem ganho para um projeto deste tamanho.

## R2. Onde fica o protocolo e com que nome

**Decision**: mover `src/dps150.rs` para `src/logics/protocol.rs` (módulo `logics::protocol`), sem
mudar o conteúdo. A única exceção é o `use std::u8;`, que é obsoleto e gera aviso: ele sai, porque
não é usado.

**Rationale**: a spec assume o protocolo sob `logics` (FR-008). O nome `protocol` diz o que o módulo
faz. `logics::dps150` repetiria o nome do crate (`dps150::logics::dps150`).

**Alternatives considered**: manter `src/dps150.rs` na raiz como módulo irmão de `logics`. Foi
rejeitado porque contraria o FR-001 (toda a lógica em `logics`).

**Impacto na constituição**: o Princípio I cita o caminho `src/dps150.rs`. Depois do merge, é preciso
uma emenda PATCH trocando para `src/logics/protocol.rs`. Não há mudança de sentido.

## R3. Pedir redesenho sem depender de egui

**Decision**: `logics::session` recebe um notificador genérico ao iniciar a conexão:
`notify: impl Fn() + Send + 'static`. A `gui` passa `move || ctx.request_repaint()`.

**Rationale**: a thread serial hoje recebe um `egui::Context` só para chamar `request_repaint()`.
Um closure resolve isso sem trait nova, mantém `logics` livre de egui (FR-004, SC-003) e, nos
testes, aceita um closure vazio ou um contador.

**Alternatives considered**:
- *Trait `Notifier`*: mais cerimônia para um único método.
- *A GUI fazer polling sem notificação*: aumentaria a latência ou o consumo de CPU (Princípio IV).

## R4. Tempo determinístico nas regras

**Decision**: as regras que dependem do relógio recebem `now: Instant` como parâmetro:
`tick(now)`, `current_elapsed(now)`, a amostragem do histórico e a atualização periódica da lista de
portas. Só a `gui` chama `Instant::now()`.

**Rationale**: torna os testes do relógio de saída e do histórico (US2, SC-002) determinísticos,
sem `sleep`.

**Alternatives considered**: *trait `Clock` injetada*. Funcionaria, mas é mais complexa que um
parâmetro.

## R5. O que é estado de lógica e o que é estado de interface

**Decision**:

| Fica em `logics` (`AppState`) | Fica em `gui` (`GuiApp`) |
|---|---|
| Estado da fonte (`DPSUpdate`), conexão e canais | Cores das séries (`[Color32; 3]`) e cores padrão |
| Portas disponíveis e porta selecionada | Paleta de status (OK, aviso, erro, ação) |
| Setpoint em edição, pendência, flags "usuário editando" | Dimensões (largura da coluna, seletor de cor, eixos) |
| Memória selecionada | Widgets (`ReadoutRow`, `card`, `info_pair`, `status_dot`) |
| Relógio de saída | Desenho do gráfico (usa `logics::scale` para os números) |
| Intervalo de leitura e histórico | |

**Rationale**: as cores usam o tipo `Color32` do egui e só servem para a apresentação, então movê-las
para `logics` violaria o FR-004. Isso **corrige uma Assumption da spec**, que colocava as cores em
`logics`. A spec foi atualizada junto com este plano. As flags "usuário editando" ficam em `logics`
porque decidem a regra de sincronização. A `gui` só informa se o campo está em foco ou sendo
arrastado.

## R6. Matemática do gráfico

**Decision**: `AxisScale`, `decimals_for` e as constantes de divisões (`CHART_Y_DIVS`,
`CHART_Y_SUBDIVS`) vão para `logics::scale`. `format_hhmmss` e `format_window` vão para
`logics::format`. O desenho (`draw_history_chart`, grade, rótulos, linhas) fica em `gui::chart`.

**Rationale**: escala e formatação são regras testáveis (SC-002 cita a escala de eixo), e o desenho
é apresentação.

## R7. Enumeração de portas na thread da GUI (débito do Princípio IV)

**Decision**: enquanto não há conexão, a lista de portas passa a ser atualizada **no máximo uma vez
por segundo**, e não mais a cada quadro. Fica em `AppState::refresh_ports_if_due(now)`, com o
intervalo `PORT_REFRESH_INTERVAL = 1 s`.

**Rationale**: hoje `serialport::available_ports()` roda na thread da GUI a cada quadro enquanto não há
conexão, e isso é E/S, ainda que rápida, na thread da interface. A constituição manda corrigir um
débito quando a área é tocada, e aqui ela é movida. Para o usuário, a única diferença é que uma porta
recém-plugada pode levar até 1 s para aparecer, dentro do espírito do FR-007.

**Alternatives considered**: enumerar na thread serial ou numa thread própria. Seria mais complexo,
e a frequência de 1 Hz já tira o custo do caminho de cada quadro.

## R8. Testes

**Decision**: testes unitários `#[cfg(test)]` dentro de cada submódulo de `logics`, rodando com
`cargo test --lib`:
- `protocol`: checksum de `build_command` (via comandos públicos), parsing do type_id 195, layout
  mínimo de `GET ALL`, ressincronização com lixo antes do cabeçalho, pacote fragmentado em duas
  chamadas e checksum inválido descartado.
- `app`: edição pendente preservada contra atualização, `load_profile` sem envio, `merge` mantendo
  campos ausentes, relógio de saída (liga, congela, zera) e limite do histórico.
- `scale` e `format`: valores de topo e casas decimais para picos típicos (0, 3,3 V, 0,001 A), e
  formatação de durações.

Para comprovar que `load_profile` e as edições não enviam comandos, os testes criam um `AppState`
com um canal de comandos de teste e verificam que ele continua vazio.

**Rationale**: atende ao FR-010, ao SC-002 e ao Princípio II sem hardware. As fixtures de protocolo
começam com bytes montados pela própria função de comando. Capturas de um aparelho real ficam como
melhoria futura, registrada no plano.
