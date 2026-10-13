# Pesquisa: fonte Noto Sans e rótulos legíveis

**Feature**: [spec.md](./spec.md) | **Plan**: [plan.md](./plan.md)

## R1 — O egui consegue usar uma fonte variável?

**Decisão**: sim; usar o arquivo variável da Noto Sans como está.
**Motivo**: o `epaint` 0.34.3 renderiza com `skrifa` e tem `FontTweak::coords` (`VariationCoords`) para
escolher eixos como `wght`. Sem `coords`, usa a instância padrão da fonte. O eixo `wght` da Noto Sans vai de
400 a 700 com padrão **400** (conferido na tabela `fvar` do arquivo), que é o peso normal que queremos.
**Alternativas**: arquivos estáticos Regular e Bold (dois arquivos de ~500 KB; mais peso no repositório e
sem ganho, porque o `strong()` do egui não troca de peso).

## R2 — De onde vem a fonte e qual é a licença

**Decisão**: copiar `NotoSans[wght].ttf` (versão 2.015, projeto `notofonts/latin-greek-cyrillic`) para
`assets/fonts/NotoSans-VariableFont_wght.ttf`, e baixar o texto da licença (`OFL.txt` do mesmo repositório)
para `assets/fonts/OFL.txt`.
**Motivo**: a licença é a SIL Open Font License 1.1 (campos 13 e 14 da própria fonte), que permite copiar,
distribuir e incluir a fonte em software, desde que a licença acompanhe os arquivos e a fonte não seja
vendida sozinha. A fonte não declara "Reserved Font Name", então copiá-la sem renomear internamente é
permitido. O nome do arquivo segue o padrão do Google Fonts (sem colchetes, que atrapalham em alguns
sistemas e ferramentas).
**Alternativas**: usar a fonte instalada no sistema (viola o pedido: a compilação dependeria da máquina);
subconjunto só com Latin (menor, mas é uma modificação da fonte e foge de "copie a fonte").

## R3 — Como embutir

**Decisão**: `include_bytes!("../../assets/fonts/NotoSans-VariableFont_wght.ttf")` em `gui/fonts.rs`,
entregue ao egui por `FontData::from_static`.
**Motivo**: o arquivo vai para dentro do executável (FR-002, US3-4); não há leitura em tempo de execução,
então não há erro de arquivo ausente a tratar (Princípio I). O caminho relativo é verificado pelo compilador.
**Alternativas**: ler do disco na partida (pode falhar e depende do diretório de trabalho).

## R4 — Ordem das famílias

**Decisão**: inserir `"noto-sans"` na posição 0 de `FontFamily::Proportional`.
**Motivo**: a documentação do `epaint` mostra exatamente isso; os fallbacks (Ubuntu-Light, `NotoEmoji`,
`emoji-icon-font`) ficam atrás e cobrem o que a Noto Sans não tem. Conferido no arquivo: á, ã, ç, Í, °, ·,
•, …, — **existem**; ✓ **não existe** (hoje nenhum texto o usa, e se usar, cai na reserva).
**Alternativas**: trocar todas as famílias (perde os fallbacks; viola FR-008).

## R5 — Largura dos algarismos

**Decisão**: nada a fazer.
**Motivo**: na Noto Sans 2.015, `zero`, `one` e `four` têm o mesmo avanço (572/1000 do corpo): os algarismos
são de largura fixa por padrão. As leituras V/A/W e o tempo não "dançam" (US2-3).

## R6 — Tamanho de 16 px no quadro

**Decisão**: constante `SETUP_FONT_SIZE = 16.0` (`gui::theme`). Rótulos, título e unidades usam
`RichText::size`; o valor (um `DragValue`) usa um `Style::override_font_id` numa sub-área, pois o
`DragValue` não tem `.size()`.
**Motivo**: é o jeito de mudar o texto de um `DragValue` sem mudar o estilo global.
**Alternativas**: mudar o `TextStyle::Button` globalmente (afetaria todos os botões e campos).

## R7 — Cabe na coluna?

**Decisão**: sim; verificar visualmente.
**Motivo**: a coluna de controle tem 330–440 px e o card tem 8 px de margem de cada lado: ≈ 314 px para três
colunas. O valor mais largo, "295.20 W", tem 8 caracteres de ≈ 9 px (572/1000 × 16) mais o espaço, ≈ 80 px
com o preenchimento do campo; três colunas dão ≈ 250 px, mais 28 px de espaços. Sobram ≈ 36 px na largura
mínima. As escalas do gráfico (`CHART_AXIS_COL_W` = 40 px, texto de 10 px, "0.0400" ≈ 31 px) também cabem.

## R8 — Altura

**Decisão**: manter `HEIGHT = 700`.
**Motivo**: no print atual, com a janela redimensionada para 615 px de altura, ainda sobra ≈ 80 px entre o
quadro e o botão de saída. O 16 px acrescenta ≈ 25 px ao quadro e a Noto Sans tem linhas ≈ 10 % mais altas;
700 px comporta com folga. Conferir no quickstart.

## R9 — Custo

**Decisão**: aceitar ≈ +0,7 MB no executável e registrar o tamanho antes e depois.
**Motivo**: o perfil `release` já é `opt-level = "s"` com `strip`; a fonte é um dado, não código. O tempo de
abertura é conferido no quickstart (SC-006).
