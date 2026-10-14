# Pesquisa: botão Aplicar na linha dos campos

**Feature**: [spec.md](./spec.md) | **Plan**: [plan.md](./plan.md)

## R1 — Como o espaço entre V e A é feito hoje

**Decisão**: reaproveitar exatamente a mesma construção.
**Motivo**: em `setpoint_card`, o campo V é seguido de `ui.add_space(10.0)` e do rótulo "A". Num `horizontal`,
o egui ainda põe o espaçamento padrão entre widgets (8 px) antes e depois do `add_space`, o que dá 8 + 10 + 8 =
26 px entre o campo V e o rótulo A. Usar `ui.add_space(FIELD_GAP)` entre o campo A e o botão repete a mesma conta
e dá 26 px também, de modo que as duas distâncias são iguais por construção (FR-002, SC-002).
**Alternativas**: `ui.spacing_mut().item_spacing.x` maior (afetaria os rótulos junto dos campos);
alinhar o botão à direita do card (a spec pede "à frente de current", não na borda).

## R2 — Cabe na largura mínima da coluna?

**Decisão**: sim; confirmar com um teste que mede o texto, em vez de confiar na conta.
**Motivo**: a coluna de controle tem 330 px (mínimo), o card tira 8 px de cada lado: 314 px. A linha é: "V" +
campo "36.00 V" + 26 + "A" + campo "8.200 A" + 26 + botão "Aplicar". Em Noto Sans de 14 px: ≈ 74 + 26 + 75 + 26 +
58 ≈ 259 px, com ≈ 55 px de folga. O teste mede as larguras reais, acrescenta o preenchimento de cada widget
(botão e campos: 2 × 4 px) e o espaçamento padrão entre widgets (8 px) e exige que o total caiba em 314 px, com
os valores máximos (36,00 V e 8,200 A).
**Alternativas**: só o roteiro visual (não pega regressão futura, por exemplo uma fonte maior).

## R3 — Onde fica o Desfazer

**Decisão** (escolhida pelo usuário): na linha do título, logo depois de "• não aplicado", só com pendência.
**Motivo**: essa linha já troca de conteúdo com a pendência (o aviso aparece e some), então acrescentar o botão
não muda a altura do card nem desloca nada (SC-003). Larguras: "Setpoint" (≈ 55 px) + "• não aplicado" (≈ 100 px)
+ **Desfazer** (≈ 65 px) + espaços (2 × 8 px) ≈ 236 px, dentro dos 314 px da coluna mínima.
**Alternativas descartadas**: linha própria abaixo dos campos (o card cresceria uma linha a cada edição e
deslocaria o que vem abaixo); na linha dos campos, depois do **Aplicar** (V, A, Aplicar e Desfazer somam ≈ 332 px,
mais que os 314 px disponíveis, e a quebra seria imprevisível).

## R4 — Alinhamento vertical e altura

**Decisão**: manter `ui.horizontal` (alinhamento central) e o `egui::Button` padrão.
**Motivo**: o `DragValue` e o `Button` têm a mesma altura mínima (`interact_size.y`) e o `horizontal` centraliza
verticalmente os itens da linha; o texto dos três fica na mesma linha de base (FR-003). Não há altura própria
para ajustar.

## R5 — Enter e o clique

**Decisão**: manter a lógica atual, dentro da mesma linha.
**Motivo**: `enter_pressed` é calculado a partir da perda de foco dos campos V e A, na mesma `horizontal`; o
botão passa a ser desenhado depois do campo A, na mesma linha, e o `if clicked || (enter_pressed && conectado)`
fica igual (FR-005). Nenhuma chamada a `AppState` muda.
