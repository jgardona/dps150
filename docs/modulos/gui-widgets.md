# `gui::widgets` — componentes reaproveitados

Arquivo: [`src/gui/widgets.rs`](../../src/gui/widgets.rs) · Visão geral: [arquitetura](../arquitetura.md)

## Papel

Reúne os pequenos componentes visuais que mais de um painel usa: a **leitura grande** de uma grandeza, a
**moldura** dos blocos, o par **"rótulo valor"**, o **ponto de status** e o auxiliar que recorta uma área em
colunas.

## O que não faz

Não lê nem altera o estado da aplicação: cada componente recebe tudo pronto por parâmetro (textos, cores,
áreas) e só desenha.

## Tipos

| Item | O que é |
|---|---|
| `ReadoutRow` | Uma leitura grande: `unit` (a unidade, "V", "A" ou "W"), `accent` (a cor), `value` (o número já formatado) e `badge` (um selo opcional, "CV" ou "CC") |

## Funções

### `ReadoutRow::ui`

| | |
|---|---|
| **Propósito** | Desenhar uma leitura grande de V, A ou W, implementando o trait `egui::Widget`. |
| **Entradas** | `self`: o `ReadoutRow` com os dados; `ui`: a área onde desenhar. |
| **Saída** | O `Response` do egui, que descreve a interação com o componente. |
| **Efeitos** | nenhum (só desenha) |

**Como funciona**: desenha uma moldura escura, com cantos arredondados, ocupando a largura toda. Dentro, em
linha, o **número** em corpo grande (34 pt) e a **unidade** ao lado (20 pt), ambos na cor `accent`. Se há
`badge`, ele aparece **encostado à direita**, como um retângulo preenchido na cor `accent` com o texto em
preto. Por ser um `Widget`, é usado com `ui.add(ReadoutRow { … })`.

### `card`

| | |
|---|---|
| **Propósito** | Desenhar a moldura padrão dos blocos da interface. |
| **Entradas** | `ui`: a área. `add_contents`: uma função que desenha o conteúdo dentro da moldura. |
| **Saída** | nada |
| **Efeitos** | Os de `add_contents`. |

**Como funciona**: abre uma moldura de fundo cinza, com cantos arredondados e margem interna de 8 px, faz
ela ocupar a largura disponível e chama `add_contents` com a área interna. Assim todos os blocos (setpoint,
gráfico) ficam com o mesmo visual.

### `info_pair`

| | |
|---|---|
| **Propósito** | Escrever um rótulo discreto seguido do valor em destaque, como "Energia 0.00 Wh". |
| **Entradas** | `ui`: a área. `label`: o rótulo. `value`: o valor. |
| **Saída** | nada |
| **Efeitos** | nenhum (só desenha) |

**Como funciona**: escreve dois textos lado a lado, o rótulo com estilo "fraco" (cinza) e o valor em negrito.
Usada nas barras superior e inferior.

### `status_dot`

| | |
|---|---|
| **Propósito** | Desenhar o pontinho colorido de status da conexão. |
| **Entradas** | `ui`: a área. `color`: a cor do ponto. |
| **Saída** | O `Response`, para o chamador poder acrescentar uma dica ao passar o mouse. |
| **Efeitos** | nenhum (só desenha) |

**Como funciona**: reserva um quadrado de 12 × 12 px que reage ao mouse (para a dica funcionar) e desenha
nele um círculo de raio 5 px na cor dada.

### `column_ui`

| | |
|---|---|
| **Propósito** | Criar uma sub-área vertical restrita a um retângulo, para desenhar uma coluna do layout. |
| **Entradas** | `ui`: a área mãe. `rect`: o retângulo da coluna. |
| **Saída** | Uma nova `Ui` que só desenha dentro de `rect`, com os itens empilhados de cima para baixo. |
| **Efeitos** | nenhum |

**Como funciona**: cria uma `Ui` filha limitada a `rect` (`max_rect`) e com o layout "de cima para baixo,
alinhado à esquerda". É a base do layout em duas colunas de [`GuiApp::ui`](gui-mod.md): uma `Ui` para a
coluna de controle e outra para o gráfico.

### `setup_field`

| | |
|---|---|
| **Propósito** | Desenhar um campo do quadro de setup: rótulo pequeno em cima e valor editável embaixo. |
| **Entradas** | `ui`: a área. `field`: o campo. `value`: o valor atual (`None` se a fonte não informou). `accent`: a cor do rótulo (V, A e W). `enabled`: se aceita edição. `invalid`: se está com erro. |
| **Saída** | `Some(novo valor)` quando o usuário altera o campo; `None` caso contrário. |
| **Efeitos** | nenhum além de desenhar; quem grava o valor é quem chama |

**Como funciona**: empilha o rótulo (com `*` quando o campo vale para o aparelho todo) e o valor. O rótulo usa `accent`; se `invalid`, rótulo e valor ficam vermelhos. Sem valor, mostra "—" desabilitado. Com valor, usa um `DragValue` com faixa de 0 até o máximo do campo, **sem cortar** o que o usuário digita (`clamp_existing_to_range(false)`): assim um valor acima do máximo continua visível e a validação o marca em vermelho, em vez de ser trocado em silêncio. As casas decimais e a unidade são as do campo.
