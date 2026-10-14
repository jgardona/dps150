# Feature Specification: Ajuste do botão Aplicar na seção Setpoint

**Feature Branch**: `fix/ajuste-botao-aplicar`

**Created**: 2026-10-13

**Status**: Draft

**Input**: User description: "Ajuste do botão Aplicar na sessão de Setpoint. O botão Aplicar está estranho abaixo dos inputs de voltage e current. Seria interessante movê-lo para a frente de current, com um padding ou margin semelhante ao desses componentes. A seta vermelha demonstra para onde ele deve ser movido na imagem em anexo."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Aplicar na mesma linha dos campos de tensão e corrente (Priority: P1)

No card **Setpoint**, o botão **Aplicar** fica sozinho numa linha abaixo dos campos V e A, encostado na borda esquerda, o que dá a impressão de um elemento solto. Ele passa a ficar **na mesma linha dos campos**, logo à direita do campo de corrente (A), com um espaço entre os dois semelhante ao que já existe entre o campo de tensão e o de corrente.

**Why this priority**: é o pedido inteiro, e o ajuste é visível na primeira olhada para o card. Também encurta o card em uma linha (e ela não volta quando o usuário edita), deixando mais espaço para o resto da coluna.

**Independent Test**: abrir o programa e olhar o card Setpoint: V, A e **Aplicar** aparecem numa única linha, na ordem V, A, Aplicar, com o mesmo espaçamento entre os três e sem a linha de botões que existia embaixo.

**Acceptance Scenarios**:

1. **Given** o card Setpoint sem alteração pendente, **When** o usuário o olha, **Then** vê o campo V, o campo A e o botão **Aplicar** na mesma linha, nessa ordem, e nenhuma linha de botões embaixo.
2. **Given** o card na largura mínima da coluna de controle (330 px), **When** o usuário o olha, **Then** os três itens cabem na linha, sem corte e sem sobreposição.
3. **Given** a distância entre o campo V e o campo A, **When** o usuário mede a distância entre o campo A e o botão **Aplicar**, **Then** as duas distâncias são iguais.
4. **Given** o botão **Aplicar** na nova posição, **When** o usuário compara a altura dele com a dos campos, **Then** os três têm a mesma altura e o mesmo alinhamento vertical (o texto "V", "A" e "Aplicar" ficam na mesma linha de base).

---

### User Story 2 - O botão continua se comportando como antes (Priority: P1)

Só a posição muda. O botão continua habilitado apenas com a fonte conectada, continua azul quando há alteração pendente, continua enviando tensão e corrente ao ser clicado (ou ao apertar **Enter** num campo) e continua mostrando a mesma dica.

**Why this priority**: o botão envia valores para uma fonte de bancada real; uma mudança de aparência não pode mudar o que ele faz.

**Independent Test**: com a fonte conectada, editar V e A, clicar em **Aplicar** (e, em outra rodada, apertar **Enter**) e ver os valores chegarem à fonte; sem conexão, ver o botão desabilitado.

**Acceptance Scenarios**:

1. **Given** a fonte conectada e uma alteração pendente, **When** o usuário clica em **Aplicar**, **Then** tensão e corrente são enviadas e o aviso "• não aplicado" some.
2. **Given** o foco no campo V ou A, **When** o usuário aperta **Enter**, **Then** o efeito é o mesmo de clicar em **Aplicar**.
3. **Given** uma alteração pendente, **When** o usuário olha o botão, **Then** ele está azul; sem pendência, está cinza.
4. **Given** a fonte desconectada, **When** o usuário olha o botão, **Then** ele está desabilitado.
5. **Given** o mouse sobre o botão, **When** a dica aparece, **Then** é a mesma de hoje ("Envia Vset/Iset ao aparelho (ou Enter)").

---

### User Story 3 - O botão Desfazer continua acessível (Priority: P2)

O botão **Desfazer** só aparece quando há alteração pendente. Com a mudança de **Aplicar**, ele passa para a **linha do título**, logo depois do aviso "• não aplicado", onde cabe sem apertar os campos e sem fazer o card mudar de altura quando a pendência aparece e some.

**Why this priority**: é consequência do pedido principal. Sem decidir onde ele fica, a mudança poderia escondê-lo ou estourar a largura da linha.

**Independent Test**: editar um campo e conferir que **Desfazer** aparece ao lado de "• não aplicado", que clicar nele volta os campos aos valores da fonte, e que o card, os campos e o que vem abaixo não se mexem nem na hora em que ele aparece nem na hora em que some.

**Acceptance Scenarios**:

1. **Given** uma alteração pendente, **When** o usuário olha o card, **Then** vê **Desfazer** na linha do título, logo depois de "• não aplicado", e a linha V, A, Aplicar não muda de posição.
2. **Given** a alteração pendente descartada ou aplicada, **When** o usuário olha o card, **Then** "• não aplicado" e **Desfazer** desaparecem e a altura do card **não muda**.
3. **Given** **Desfazer** visível, **When** o usuário o clica, **Then** os campos voltam aos valores atuais da fonte, como hoje.

---

### Edge Cases

- Na largura mínima da coluna (330 px), com os valores máximos (36,00 V e 8,200 A) e com o texto do campo mais largo, os três itens da linha DEVEM caber sem quebra de linha.
- O aviso "• não aplicado" e o botão **Desfazer** aparecem e somem na linha do título sem deslocar a linha de campos nem o que vem abaixo (a altura do card é a mesma com e sem pendência).
- Na largura mínima da coluna (330 px), a linha do título com "Setpoint", "• não aplicado" e **Desfazer** cabe sem quebrar.
- Ao digitar num campo (foco) e apertar **Enter** com a fonte desconectada, nada é enviado, como hoje.
- Redimensionar a janela: a linha continua alinhada à esquerda, sem esticar o botão.
- O botão não pode ficar menor que a área clicável confortável (a mesma de hoje).

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: O botão **Aplicar** DEVE ficar na mesma linha dos campos V e A do card Setpoint, imediatamente à direita do campo A, e NÃO DEVE haver mais uma linha só de botões abaixo dos campos, com ou sem alteração pendente.
- **FR-002**: A distância entre o campo A e o botão **Aplicar** DEVE ser igual à distância entre o campo V e o campo A.
- **FR-003**: O botão DEVE ter a mesma altura e o mesmo alinhamento vertical dos campos V e A.
- **FR-004**: Os três itens (V, A, **Aplicar**) DEVEM caber na linha na largura mínima da coluna de controle, com os valores máximos nos campos, sem corte nem quebra.
- **FR-005**: O comportamento do botão NÃO DEVE mudar: habilitado só com a fonte conectada; azul com alteração pendente e cinza sem ela; mesma dica; envia tensão e corrente ao clique; **Enter** num campo tem o mesmo efeito.
- **FR-006**: O botão **Desfazer** DEVE continuar aparecendo só quando há alteração pendente e DEVE ficar na linha do título do card, logo depois do aviso "• não aplicado". A aparição e o sumiço dele NÃO DEVEM mudar a altura do card nem deslocar nenhum outro elemento.
- **FR-007**: Nada além da posição, do espaçamento e do alinhamento DEVE mudar no card Setpoint: textos, cores, limites, regras de edição e o restante da coluna de controle ficam como estão.
- **FR-008**: O README e a documentação de `docs/` DEVEM ser atualizados na mesma entrega, onde descrevem a posição do botão **Aplicar** (README: seção Setpoint; `docs/`: a entrada de `setpoint_card`), com a conferência de documentação passando.
- **FR-009**: O botão **Desfazer** DEVE começar na mesma posição horizontal (eixo x) do botão **Aplicar**, na linha de baixo, em qualquer largura dos campos, desde que o aviso "• não aplicado" termine antes dessa posição; se terminar depois, o **Desfazer** vem logo depois do aviso. (Pedido do usuário depois da primeira entrega.)
- **FR-010**: O botão **Aplicar** DEVE ter a mesma largura do botão **Desfazer** (a do texto mais largo dos dois, mais o preenchimento padrão), e o texto de cada um DEVE ficar centralizado. A linha V, A e **Aplicar** continua cabendo na coluna mínima. (Pedido do usuário depois da primeira entrega.)

### Key Entities

- **Card Setpoint**: o bloco com o título, os campos de tensão (V) e corrente (A), o botão **Aplicar**, o botão **Desfazer** (quando há pendência) e a linha de memórias.
- **Linha de campos**: V, A e **Aplicar**, em sequência, com espaçamento uniforme.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Os três itens (V, A, **Aplicar**) aparecem em **1 linha** em 100% das aberturas do programa, na janela inicial e na largura mínima da coluna.
- **SC-002**: A diferença entre a distância V→A e a distância A→**Aplicar** é de no máximo 1 px.
- **SC-003**: O card Setpoint fica **uma linha mais baixo** (cerca de 30 px a menos) do que hoje, e tem a **mesma altura** com e sem alteração pendente (0 px de diferença).
- **SC-004**: Nenhuma ação do botão muda: todos os testes existentes continuam passando sem alteração de resultado, e a verificação manual com a fonte (editar, aplicar, **Enter**, desfazer) tem o mesmo resultado de antes.
- **SC-005**: Numa amostra de avaliadores, ao olhar o card pela primeira vez, ninguém aponta o botão como "solto" ou "fora do lugar" (a observação original que motivou o ajuste).

## Assumptions

- A posição pedida é a da seta do print: na linha dos campos, à direita do campo A. A ordem fica V, A, **Aplicar**.
- O "padding ou margin semelhante" é o espaço que já separa o campo V do campo A; o botão usa o mesmo valor, sem criar um novo.
- **Desfazer** (que o pedido não cita) vai para a linha do título, depois de "• não aplicado", só quando há pendência. Decisão do usuário, escolhida entre duas alternativas: a outra (linha própria abaixo dos campos) faria o card crescer enquanto se edita, e quatro itens na linha dos campos não caberiam na largura mínima.
- O botão mantém o tamanho e o texto atuais; só a posição muda.
- O botão **Desfazer** mantém o texto e o tamanho de hoje.
- Fora do escopo: mudar cores, textos, limites ou regras do card; reorganizar a linha de memórias; mudar o quadro de setup.
- Segue a constituição: nenhum envio implícito (Princípio III: só o clique em **Aplicar**, ou **Enter**, envia), textos em pt-BR e README atualizado na mesma entrega.
