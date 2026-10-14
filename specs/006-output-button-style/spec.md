# Feature Specification: Estilo do botão de saída

**Feature Branch**: `fix/ajustes-botao-saida`

**Created**: 2026-10-14

**Status**: Draft

**Input**: User description: "Ajustes no botão saída. No botão saída de tensão (ligada/desligada) precisamos arredondar as suas bordas. Podemos usar um raio (se não me engano essa é a unidade) de 5px. Também no botão, podemos usar um margin bottom de 10px."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Bordas arredondadas no botão de saída (Priority: P1)

O botão grande **SAÍDA LIGADA / SAÍDA DESLIGADA** tem hoje cantos quase retos, diferentes dos cards da interface, que são arredondados. Os quatro cantos do botão passam a ter um **raio de 5 px**, nos dois estados (ligada e desligada) e também desabilitado (sem conexão).

**Why this priority**: é o pedido principal, visível na primeira olhada para a coluna de controle, e dá ao botão o mesmo acabamento do resto da interface.

**Independent Test**: abrir o programa e olhar o botão de saída desconectado (cinza, desabilitado), e depois conectado, ligando e desligando a saída: nos três casos os quatro cantos aparecem arredondados do mesmo jeito.

**Acceptance Scenarios**:

1. **Given** o programa aberto sem conexão, **When** o usuário olha o botão de saída, **Then** os quatro cantos aparecem arredondados com raio de 5 px.
2. **Given** a fonte conectada e a saída desligada, **When** o usuário olha o botão, **Then** o raio é de 5 px.
3. **Given** a saída ligada (botão verde), **When** o usuário olha o botão, **Then** o raio é de 5 px, igual ao do estado desligado.
4. **Given** o mouse sobre o botão ou o botão pressionado, **When** o usuário o olha, **Then** o contorno, o brilho e o preenchimento seguem o mesmo raio de 5 px, sem cantos retos aparecendo por baixo.

---

### User Story 2 - Margem inferior de 10 px (Priority: P1)

Hoje o botão de saída encosta na barra inferior (a que mostra capacidade, energia e tempo), como mostra o print. Ele passa a ter uma **margem de 10 px entre a borda de baixo do botão e a barra inferior**.

**Why this priority**: é o segundo pedido e resolve o botão "colado" na barra. É independente da US1.

**Independent Test**: abrir o programa e conferir que há um espaço de 10 px, livre de qualquer elemento, entre o fim do botão e o início da barra inferior, em qualquer tamanho de janela.

**Acceptance Scenarios**:

1. **Given** a janela no tamanho inicial, **When** o usuário olha a base da coluna de controle, **Then** há 10 px de espaço entre o botão de saída e a barra inferior.
2. **Given** a janela maximizada ou reduzida, **When** o usuário olha a mesma região, **Then** o espaço continua sendo de 10 px.
3. **Given** a margem de 10 px aplicada, **When** o usuário olha a altura e a largura do botão, **Then** elas continuam as de hoje (56 px de altura, toda a largura da coluna).
4. **Given** a coluna de controle na altura mínima útil, **When** o botão sobe 10 px, **Then** ele não sobrepõe o quadro de setup nem outro elemento.

---

### Edge Cases

- A margem NÃO pode empurrar o botão para cima de outros elementos quando a janela é baixa: o botão e a margem têm prioridade na base, e o que estiver acima rola ou fica coberto como hoje (o comportamento atual da coluna não muda).
- O raio de 5 px vale também para o botão desabilitado, que usa cores acinzentadas.
- O texto "SAÍDA LIGADA"/"SAÍDA DESLIGADA" continua centralizado e inteiro, sem corte pelos cantos arredondados.
- Mudar entre ligada e desligada não pode mudar o tamanho nem a posição do botão.
- Em telas de densidade alta (escala da interface), as medidas continuam sendo 5 px e 10 px lógicos.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Os quatro cantos do botão de saída DEVEM ter raio de **5 px**, nos estados ligada, desligada e desabilitado.
- **FR-002**: Em foco, mouse por cima e pressionado, o botão DEVE manter o mesmo raio de 5 px em todos os elementos desenhados (preenchimento, contorno e realce).
- **FR-003**: Entre a borda de baixo do botão de saída e a barra inferior DEVE haver uma margem de **10 px**, em qualquer tamanho de janela.
- **FR-004**: A altura (56 px) e a largura (toda a coluna) do botão NÃO DEVEM mudar, e o texto DEVE continuar centralizado e inteiro.
- **FR-005**: O comportamento do botão NÃO DEVE mudar: habilitado só com a fonte conectada, mesmo texto, mesma dica, e o clique liga ou desliga a saída como hoje.
- **FR-006**: Nada além do raio e da margem inferior DEVE mudar na coluna de controle: cores, textos, outros botões e a ordem dos elementos ficam como estão.
- **FR-007**: O README e a documentação de `docs/` DEVEM ser atualizados na mesma entrega, onde descrevem o botão de saída (README: seção "Ligar e desligar a saída"; `docs/`: a entrada de `output_button` e a tabela de constantes de `theme`), com a conferência de documentação passando.

### Key Entities

- **Botão de saída**: o botão grande no fim da coluna de controle que mostra o estado da saída e a liga ou desliga.
- **Margem inferior**: o espaço livre entre o botão e a barra inferior.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Os quatro cantos do botão têm raio de 5 px em 100% dos estados (desabilitado, desligada, ligada, mouse por cima e pressionado).
- **SC-002**: O espaço entre o botão e a barra inferior é de 10 px (tolerância de 1 px) na janela inicial, maximizada e reduzida.
- **SC-003**: O botão mantém 56 px de altura e a largura da coluna: diferença de 0 px em relação a hoje.
- **SC-004**: Nenhum comportamento muda: todos os testes existentes continuam passando sem alteração de resultado, e ligar/desligar a saída com a fonte tem o mesmo resultado de antes.
- **SC-005**: Numa amostra de avaliadores, ninguém aponta o botão como "colado na barra" nem como "de cantos diferentes do resto" (as observações que motivaram o ajuste).

## Assumptions

- A unidade citada é o **pixel lógico**; "raio" é o raio de arredondamento dos cantos (você acertou: é o termo certo).
- A margem de 10 px é só **inferior** (entre o botão e a barra de baixo). Margens laterais e superior não foram pedidas e ficam como estão (o botão ocupa toda a largura da coluna e o espaço acima dele não muda).
- O raio vale igual para ligada, desligada e desabilitado, e para todos os estados de interação.
- Fora do escopo: mudar cores, textos, altura, largura ou tamanho de fonte do botão; mudar outros botões; arredondar outros elementos.
- Segue a constituição: nenhum envio implícito (Princípio III: só o clique liga/desliga), textos em pt-BR e README atualizado na mesma entrega.
