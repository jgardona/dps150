# Feature Specification: Campos de configuração das memórias

**Feature Branch**: `feature/memorias-melhorias`

**Created**: 2026-10-12

**Status**: Draft

**Input**: User description: "Melhoria no comportamento de configurações das memórias. Adicionar um setup, com campos de vset, iset, ovp, ocp, opp, otp no espaço vazio da coluna de controle, abaixo da linha das memórias. Os campos seriam editáveis. Ao apertar o botão Salvar, esses dados seriam aplicados na fonte."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Ver os seis valores da memória selecionada (Priority: P1)

Hoje, para saber o que há numa memória, o usuário precisa passar o mouse sobre M1…M6 e só vê tensão e corrente. O espaço vazio abaixo da linha das memórias passa a mostrar um quadro de **setup** com os seis valores de configuração da memória selecionada, no mesmo arranjo do visor da fonte: **Vset**, **Iset**, **OVP**, **OCP**, **OPP** e **OTP**, cada um com a sua unidade (V, A, V, A, W, °C).

**Why this priority**: é a base das outras histórias. Mesmo sem edição, ver todos os valores de uma memória de uma vez já ajuda a conferir a fonte sem olhar o visor dela.

**Independent Test**: conectar a fonte, clicar em M1…M6 e conferir que o quadro mostra os seis valores da memória clicada, iguais ao que o visor da fonte mostra.

**Acceptance Scenarios**:

1. **Given** a fonte conectada e a memória M1 selecionada, **When** o usuário olha o quadro de setup, **Then** vê Vset, Iset, OVP, OCP, OPP e OTP, com unidade e na mesma ordem do visor da fonte (duas linhas de três campos).
2. **Given** o quadro aberto em M1, **When** o usuário clica em M3, **Then** o quadro passa a mostrar os valores de M3 e nenhum comando é enviado à fonte.
3. **Given** a fonte desconectada, **When** o usuário olha o quadro, **Then** os campos aparecem desabilitados e sem valores inventados.

---

### User Story 2 - Editar os campos e salvar na fonte (Priority: P1)

O usuário edita qualquer um dos seis campos do quadro e, ao clicar em **Salvar**, todos os valores do quadro são enviados à fonte e gravados na memória selecionada.

**Why this priority**: é o pedido central: configurar a memória inteira pela tela, sem usar os botões do aparelho.

**Independent Test**: com a fonte conectada, selecionar M2, mudar os seis campos, clicar em **Salvar**, e conferir no visor da fonte (e ao recarregar M2 no programa) que os seis valores foram gravados.

**Acceptance Scenarios**:

1. **Given** a memória M2 selecionada e a fonte conectada, **When** o usuário altera Vset para 5,00 V e Iset para 0,500 A e clica em **Salvar**, **Then** a fonte passa a ter 5,00 V e 0,500 A gravados em M2.
2. **Given** o usuário alterou OVP, OCP, OPP e OTP, **When** clica em **Salvar**, **Then** a fonte passa a usar esses limites de proteção.
3. **Given** campos editados e ainda não salvos, **When** o usuário olha o quadro, **Then** vê o aviso de que há alteração pendente, e os campos não são sobrescritos pelos valores que chegam da fonte.
4. **Given** campos editados e não salvos, **When** o usuário clica em outra memória, **Then** o quadro mostra os valores da outra memória (a edição pendente é descartada, e o usuário é avisado disso antes de perder o que digitou).
5. **Given** a fonte desconectada, **When** o usuário olha o botão **Salvar**, **Then** ele está desabilitado.

---

### User Story 3 - Não aceitar valores perigosos ou sem sentido (Priority: P2)

Os campos só aceitam valores dentro da faixa do aparelho, e a combinação deve fazer sentido: o Vset não pode passar do OVP, e o Iset não pode passar do OCP.

**Why this priority**: a fonte alimenta circuitos reais. Gravar um Vset acima do OVP, por exemplo, faria a fonte desarmar logo ao usar a memória.

**Independent Test**: digitar Vset maior que OVP e conferir que o programa avisa o problema e não envia nada ao clicar em **Salvar**.

**Acceptance Scenarios**:

1. **Given** qualquer OVP válido, **When** o usuário digita Vset 36,50 V (acima do máximo de 36,00 V), **Then** o campo é marcado como inválido, e **Salvar** não envia nada enquanto houver erro.
2. **Given** Vset 12,00 V e OVP 10,00 V, **When** o usuário tenta salvar, **Then** o programa mostra qual campo está errado e por quê, e nada é enviado.
3. **Given** o usuário digita OVP 36,20 V, OCP 8,300 A, OPP 300,00 W ou OTP 100 °C (todos acima do máximo do campo), **When** tenta salvar, **Then** o campo é marcado como inválido e nada é enviado.
4. **Given** os valores máximos de cada campo (Vset 36,00 V, Iset 8,200 A, OVP 36,10 V, OCP 8,200 A, OPP 295,20 W, OTP 99 °C), **When** o usuário clica em **Salvar**, **Then** todos são aceitos e enviados.
5. **Given** um campo vazio ou com texto que não é número, **When** o usuário tenta salvar, **Then** o campo é marcado como inválido e nada é enviado.

---

### Edge Cases

- A fonte ainda não informou os valores da memória (logo depois de conectar): os campos mostram "—" e **Salvar** fica desabilitado até os valores chegarem.
- A conexão cai com alterações pendentes: os campos ficam desabilitados, mas o que foi digitado não se perde até o usuário trocar de memória.
- **Salvar** é clicado e a gravação falha ou a fonte não confirma: o usuário precisa ver que não foi confirmado, e não uma mensagem de sucesso.
- Fonte conectada de outro modelo (faixa menor): a interface aplica os máximos de FR-013 e a fonte pode recusar o valor; o usuário vê que a gravação não foi confirmada (FR-011).
- O usuário salva uma memória enquanto a saída está ligada: os valores salvos não alteram a saída atual; isso só acontece ao carregar e **Aplicar** o setpoint, como hoje.
- OVP/OCP/OPP/OTP salvos valem para o aparelho inteiro, não só para a memória (veja Premissas): o usuário precisa ser avisado disso.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: O sistema DEVE mostrar, no espaço vazio abaixo da linha das memórias, um quadro de setup com seis campos: Vset, Iset, OVP, OCP, OPP e OTP, em duas linhas de três, na mesma ordem do visor da fonte.
- **FR-002**: Cada campo DEVE mostrar a unidade correspondente (V, A, V, A, W, °C) e o rótulo na cor da grandeza quando houver uma (V, A, W).
- **FR-003**: O quadro DEVE mostrar os valores da memória selecionada (M1–M6) e DEVE trocar de conteúdo ao selecionar outra memória, sem enviar comandos à fonte.
- **FR-004**: Os seis campos DEVEM ser editáveis (digitando ou arrastando), como os campos de setpoint.
- **FR-005**: O botão **Salvar** DEVE enviar à fonte os seis valores do quadro e gravá-los na memória selecionada. Nenhuma outra ação (selecionar, editar, carregar) DEVE enviar comandos.
- **FR-006**: O sistema DEVE sinalizar edição pendente no quadro e DEVE preservar o que o usuário digitou contra atualizações vindas da fonte, até salvar ou descartar.
- **FR-007**: O sistema DEVE oferecer como descartar a edição pendente do quadro, voltando aos valores atuais da memória.
- **FR-008**: O sistema DEVE validar os valores antes de enviar: dentro da faixa de cada campo (FR-013), Vset ≤ OVP e Iset ≤ OCP, e todos numéricos. Havendo erro, DEVE indicar o campo e o motivo, e NÃO DEVE enviar nada.
- **FR-009**: Sem conexão, os campos e o botão **Salvar** DEVEM ficar desabilitados. Com a conexão, **Salvar** só DEVE ficar habilitado com todos os seis valores conhecidos e válidos.
- **FR-010**: O sistema DEVE avisar o usuário, de forma visível na interface, que OVP/OCP/OPP/OTP valem para o aparelho inteiro, e não para a memória selecionada, quando esse for o comportamento do aparelho.
- **FR-011**: Depois de salvar, o sistema DEVE reler os valores da fonte e mostrar o que ela de fato guardou, deixando claro se a gravação não foi confirmada.
- **FR-013**: Cada campo DEVE aceitar de 0 até o seguinte máximo, e NÃO DEVE aceitar valor acima dele:

  | Campo | Máximo | Casas decimais |
  |---|---|---|
  | Vset | 36,00 V | 2 |
  | Iset | 8,200 A | 3 |
  | OVP | 36,10 V | 2 |
  | OCP | 8,200 A | 3 |
  | OPP | 295,20 W | 2 |
  | OTP | 99 °C | 0 |

- **FR-014**: Os campos Vset e Iset do card "Setpoint" (saída atual) DEVEM aceitar os mesmos máximos de Vset e Iset (36,00 V e 8,200 A), para que a interface tenha uma só faixa de tensão e corrente. Isso substitui a limitação de 30 V e 5 A hoje registrada no README.
- **FR-015**: Toda a documentação DEVE ser atualizada na mesma entrega: o README (inclusive a seção de memórias, as proteções e as limitações conhecidas), os documentos de `docs/` (visão geral, módulos e funções alteradas, novas ou removidas, com a conferência `docs_coverage` passando) e os registros do projeto que citam as faixas antigas (constituição, débitos de faixa).
- **FR-012**: Todos os textos novos DEVEM estar em pt-BR (com as abreviações técnicas V, A, W, OVP, OCP, OPP, OTP) e o README DEVE descrever o quadro e a nova regra do **Salvar**.

### Key Entities

- **Memória (M1–M6)**: um dos seis grupos da fonte. Tem Vset e Iset próprios.
- **Setup da memória**: o conjunto dos seis valores (Vset, Iset, OVP, OCP, OPP, OTP) que o usuário vê e edita no quadro, mais o estado "pendente" ou "salvo" e os erros de validação de cada campo.
- **Proteções (OVP, OCP, OPP, OTP)**: limites de tensão, corrente, potência e temperatura que fazem a fonte desligar a saída ao serem passados.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: O usuário vê os seis valores de qualquer memória em 1 clique (a seleção da memória), sem passar o mouse nem abrir outra tela.
- **SC-002**: O usuário configura uma memória inteira (seis valores) e salva em menos de 1 minuto, sem usar os botões do aparelho.
- **SC-003**: Depois de **Salvar**, os seis valores mostrados no visor da fonte são iguais aos do quadro em 100% das tentativas válidas.
- **SC-004**: 100% das tentativas de salvar valores inválidos (acima do máximo de FR-013, Vset > OVP, Iset > OCP, texto não numérico) são barradas, com mensagem que diz o campo e o motivo, e nada é enviado à fonte.
- **SC-005**: Nenhuma edição pendente é perdida por atualização automática da fonte em sessões de teste de 10 minutos.
- **SC-007**: Os seis valores máximos de FR-013 são aceitos e gravados, e qualquer valor acima deles é barrado, em 100% das tentativas.
- **SC-008**: Depois da entrega, nenhum trecho do README ou de `docs/` cita a faixa antiga (30 V / 5 A), e a conferência de documentação passa.
- **SC-006**: Selecionar ou carregar memórias, em qualquer quantidade, nunca altera a fonte (zero comandos enviados sem clique em **Salvar**, **Aplicar** ou no botão de saída).

## Assumptions

- Os valores de OVP, OCP, OPP e OTP são lidos hoje como limites **do aparelho**, não de cada memória (o protocolo só tem Vset/Iset por memória). Assume-se que o quadro mostra e grava as proteções do aparelho, e as memórias guardam só Vset/Iset. A interface avisa isso (FR-010). Se testes com a fonte real mostrarem que as proteções também existem por memória, o plano ajusta o comportamento sem mudar o desenho do quadro.
- O quadro ocupa o espaço mostrado em vermelho no print: entre a linha de memórias e o botão de saída, na coluna de controle.
- O quadro mostra o Vset/Iset **salvos** da memória selecionada, e é separado do card "Setpoint" (que continua a controlar a saída atual com **Aplicar**).
- Os máximos de FR-013 são os do modelo IPS3608 e valem como faixa fixa desta feature. Aparelhos de faixa menor (DPS150) podem rejeitar o valor; esse caso fica registrado como limitação conhecida, não tratado aqui.
- O mínimo de todos os campos é 0.
- Escrita das proteções (OVP, OCP, OPP, OTP) ainda não foi confirmada em aparelho real: o envio DEVE ser marcado como experimental no código e no README, e ser validado com a fonte conectada antes do merge (constituição, Princípio II e Restrições Técnicas).
- LVP (subtensão), brilho, volume e medição ficam fora desta feature.
- Os valores das proteções são mostrados nas unidades do visor: V, A, W e °C.
- Esta feature segue a constituição: nenhum envio implícito (Princípio III), regra de validação em lógica testável fora do desenho (Princípio II) e E/S serial somente pela thread da sessão (Princípio IV).
