# Pesquisa: estilo do botão de saída

**Feature**: [spec.md](./spec.md) | **Plan**: [plan.md](./plan.md)

## R1 — Como arredondar só este botão

**Decisão**: `egui::Button::new(...).corner_radius(OUTPUT_BUTTON_RADIUS)`, com `OUTPUT_BUTTON_RADIUS: u8 = 5`.
**Motivo**: no egui 0.34.3, `Button::corner_radius` (conferido em `widgets/button.rs`) guarda o raio e, ao
desenhar, o aplica ao `Frame` do botão (`frame.corner_radius(...)`), seja qual for o estado visual (desabilitado,
normal, mouse por cima, pressionado). O `CornerRadius` aceita `u8` em pontos lógicos, então 5 já é 5 px lógicos
(FR-001, FR-002).
**Alternativas**: mudar `visuals.widgets.*.corner_radius` no estilo global (afetaria todos os botões; fora do
escopo, FR-006); desenhar o botão à mão com `painter.rect` (perde foco, hover e acessibilidade).

## R2 — Como dar a margem de baixo

**Decisão**: em `output_button`, antes de criar o botão, `ui.add_space(OUTPUT_BOTTOM_MARGIN)`.
**Motivo**: `control_column` desenha o botão num `with_layout(bottom_up)`, em que o primeiro item fica encostado
na base e os seguintes empilham para cima. Um espaço desenhado antes do botão o sobe.
**Correção feita na implementação**: o plano previa descontar `item_spacing.y` (3 px), supondo que o egui
somasse o espaçamento entre itens depois do espaço. O teste da folga mostrou o contrário: com o desconto a folga
dava 7 px, e sem ele dá os 10 px pedidos. O egui não acrescenta o espaçamento depois desse espaço, neste layout.
**Alternativas**: `Frame` com `outer_margin` (adiciona uma moldura só para isso); alterar o painel inferior
(muda outra região).

## R3 — A barra inferior tem margem própria?

**Decisão**: a margem de 10 px é a distância entre o botão e o **fim da área da coluna**, que é onde a barra inferior
começa.
**Motivo**: `GuiApp::ui` desenha o painel inferior (`info_bar`) dentro do painel central e só depois mede a área que
sobra (`available_rect_before_wrap`) para as colunas; logo, o fundo da coluna encosta no começo da barra. O painel
inferior pode ter margem interna (do estilo do egui), que fica *dentro* da barra e não entra na conta. Como o
pedido é "10 px entre o botão e a barra", o roteiro visual confirma o resultado (quickstart, passo 2); se a barra
tiver uma folga própria visível acima do texto, o número de 10 px segue valendo para o botão.

## R4 — Como testar sem janela

**Decisão**: dois testes em `src/gui/panels.rs`, com um `egui::Context` de teste:

1. **Raio**: rodar `output_button` (desabilitado, que é o estado de um `AppState::new()`) com `ctx.run_ui`, pegar as
   formas desenhadas (`FullOutput::shapes`) e conferir que existe um retângulo preenchido com a cor do botão cujo
   `corner_radius` é `CornerRadius::same(5)`.
2. **Folga de baixo**: rodar `output_button` dentro de `with_layout(bottom_up)` e conferir que
   `ui.max_rect().bottom() - resposta.rect.bottom()` é 10 ± 0,5 px.

**Motivo**: Princípio II (regra visual medida sem olhos). Os estados "ligada" e "mouse por cima" usam o mesmo
caminho de código (o raio é uma só chamada, antes do `fill`), e o teste de estado ligado exigiria um `AppState`
conectado, que só existe dentro de `logics`; o roteiro visual cobre esses estados.
**Alternativas**: só roteiro visual (não pega regressão).

## R5 — A margem muda a altura do botão ou o layout acima?

**Decisão**: não.
**Motivo**: a altura (56 px) e a largura (toda a coluna) vêm de `add_sized`, que não muda. O botão só sobe 10 px
dentro da coluna; o trecho de cima (leituras, setpoint, setup) é desenhado de cima para baixo e não depende dele.
