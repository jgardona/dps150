# Feature Specification: Separação do controlador em módulos `logics` e `gui`

**Feature Branch**: `001-modularize-logics-gui`

**Created**: 2026-10-12

**Status**: Draft

**Input**: User description: "Nessa aplicação e refatoração que estamos aplicando necessitamos seguir as
seguintes especificações: 1. As regras de negócio e lógica do programa precisam estar em um módulo
apartado chamado `logics`. 2. A UI da aplicação deve estar em um módulo apartado chamado `gui`. 3. O
entrypoint da aplicação será o arquivo `src/controller.rs`, no qual estará a função main."

## Contexto

Hoje todo o controlador está num único arquivo de pouco mais de mil linhas: estado da aplicação, regras
(setpoint pendente, memórias, relógio de saída, histórico, cálculo de escalas), sessão de comunicação
com a fonte e desenho da interface. Isso dificulta testar as regras sem abrir a janela, como exige o
Princípio II da constituição, e aumenta o risco de uma mudança visual quebrar o comportamento.

Esta feature é uma **refatoração estrutural**. Os nomes `logics`, `gui` e `src/controller.rs` são
requisitos explícitos do mantenedor e por isso aparecem na especificação. O comportamento visível para
quem opera a fonte NÃO muda.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Operar a fonte exatamente como antes (Priority: P1)

Quem usa o controlador conecta a fonte, ajusta tensão e corrente, usa as memórias, liga e desliga a
saída e acompanha o gráfico, sem notar diferença nenhuma depois da reorganização.

**Why this priority**: uma refatoração que muda o comportamento é uma regressão. Preservar tudo o que
já funciona é a condição para a feature ser aceitável.

**Independent Test**: executar o roteiro de verificação manual completo (seção "Verificação manual de
paridade") com a fonte conectada e comparar com o comportamento atual documentado no README.

**Acceptance Scenarios**:

1. **Given** a fonte conectada a uma porta USB, **When** o usuário seleciona a porta e clica em
   "Conectar", **Then** o status fica verde, modelo, firmware, entrada e temperatura aparecem, e a
   saída é desligada, como hoje.
2. **Given** a fonte conectada, **When** o usuário edita Vset/Iset, **Then** aparece "não aplicado" e
   nada é enviado até "Aplicar" ou Enter.
3. **Given** a fonte conectada, **When** o usuário clica em M2, **Then** os campos recebem os valores de
   M2 sem enviar nada; **When** clica em "Salvar", **Then** os valores dos campos são gravados em M2.
4. **Given** a saída desligada, **When** o usuário clica no botão de saída, **Then** a saída liga, o
   botão mostra "SAÍDA LIGADA" e o tempo começa a contar.
5. **Given** a fonte conectada, **When** o usuário muda o intervalo de Update, troca a cor de uma
   grandeza ou clica em "Limpar", **Then** o gráfico reage como hoje.
6. **Given** a fonte conectada, **When** o usuário clica em "Desconectar", **Then** a sessão é
   bloqueada, a porta é liberada e os dados são zerados.

---

### User Story 2 - Regras do programa verificáveis sem interface (Priority: P2)

O mantenedor consegue verificar as regras do programa (setpoint pendente, carga e gravação de memória,
relógio de saída, histórico limitado, cálculo das escalas do gráfico, junção das atualizações
parciais da fonte) com testes automáticos, sem abrir a janela e sem a fonte conectada.

**Why this priority**: é o ganho que justifica a refatoração e o que destrava o Princípio II da
constituição. Depende da US1 estar preservada.

**Independent Test**: rodar a suíte de testes automáticos numa máquina sem interface gráfica e sem
porta serial, e ver as regras exercitadas passarem.

**Acceptance Scenarios**:

1. **Given** o módulo `logics`, **When** a suíte de testes roda sem interface gráfica disponível,
   **Then** os testes das regras executam e passam.
2. **Given** uma edição pendente de setpoint, **When** chega uma atualização da fonte com outros
   valores, **Then** um teste comprova que a edição pendente é preservada.
3. **Given** um histórico cheio, **When** chega uma nova amostra, **Then** um teste comprova que a mais
   antiga é descartada e o tamanho não passa do limite.

---

### User Story 3 - Estrutura de código previsível (Priority: P3)

O mantenedor sabe onde fazer cada tipo de mudança: regra de negócio em `logics`, aparência e interação
em `gui`, inicialização da aplicação em `src/controller.rs`.

**Why this priority**: melhora a manutenção de longo prazo, mas não entrega comportamento novo.

**Independent Test**: revisar a árvore de código e as dependências entre módulos. `gui` usa `logics`,
`logics` não usa `gui`, e `src/controller.rs` contém só a inicialização.

**Acceptance Scenarios**:

1. **Given** o código refatorado, **When** se procura qualquer referência à biblioteca de interface em
   `logics`, **Then** nenhuma é encontrada.
2. **Given** o código refatorado, **When** se abre `src/controller.rs`, **Then** ele contém a função
   `main` e apenas a configuração da janela e a ligação entre `gui` e `logics`.

---

### Edge Cases

- **Erro ao abrir a porta** (porta inexistente, ocupada ou sem permissão): a mensagem de erro continua
  aparecendo na barra superior, e a interface volta ao estado desconectado sem travar.
- **Fonte desconectada fisicamente durante a sessão**: o comportamento é o mesmo de hoje. Nenhuma
  regressão é introduzida (melhorias nesse caso ficam fora do escopo).
- **Nenhuma porta disponível ao clicar em "Conectar"**: continua mostrando "Nenhum dispositivo/porta
  serial disponível para conexão."
- **Atualizações parciais da fonte** (só alguns campos presentes): campos ausentes mantêm o último
  valor conhecido, como hoje.
- **Aviso visual de redesenho**: a sessão de comunicação continua pedindo redesenho da tela ao
  receber dados, mesmo sem depender diretamente da biblioteca de interface.
- **Monitor de linha de comando (`tests`)**: continua compilando e funcionando como antes.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Todas as regras de negócio e a lógica do programa DEVEM ficar num módulo próprio chamado
  `logics`. Isso inclui: estado da aplicação e da conexão, setpoint e edição pendente, memórias M1–M6,
  relógio de tempo de saída, histórico de amostras e seus limites, intervalo de leitura, cálculo das
  escalas do gráfico, formatação de durações, junção de atualizações parciais e a sessão de
  comunicação com a fonte.
- **FR-002**: Toda a interface com o usuário DEVE ficar num módulo próprio chamado `gui`: layout,
  barras, leituras, card de setpoint, memórias, botão de saída, gráfico, legenda, seletor de cor,
  cores e dimensões visuais.
- **FR-003**: O ponto de entrada da aplicação DEVE ser `src/controller.rs`, com a função `main`. Esse
  arquivo DEVE se limitar a configurar a janela e iniciar a aplicação, sem regras de negócio nem
  desenho de interface.
- **FR-004**: O módulo `logics` NÃO DEVE depender do módulo `gui` nem da biblioteca de interface. A
  dependência é só no sentido `gui` → `logics`.
- **FR-005**: O módulo `gui` NÃO DEVE conter regras de negócio. Ele lê o estado, apresenta e repassa as
  ações do usuário ao `logics`.
- **FR-006**: A sessão de comunicação com a fonte DEVE continuar rodando fora da thread da interface e
  DEVE continuar sinalizando à interface quando houver dados novos para redesenhar.
- **FR-007**: O comportamento visível DEVE permanecer idêntico ao atual: textos, cores, layout,
  limites dos campos, sequência de inicialização (incluindo desligar a saída ao conectar), intervalo
  de leitura, tamanho do histórico, escalas e ticks do gráfico.
- **FR-008**: O protocolo de comunicação com a fonte (montagem e interpretação de pacotes) DEVE
  continuar isolado e sem E/S, conforme o Princípio I da constituição, ficando sob a responsabilidade
  de `logics`.
- **FR-009**: O monitor de linha de comando (binário `tests`) DEVE continuar compilando e funcionando,
  reaproveitando o protocolo sem duplicá-lo.
- **FR-010**: As regras de `logics` citadas na US2 DEVEM ter testes automáticos executáveis sem
  interface gráfica e sem porta serial.
- **FR-011**: A refatoração NÃO DEVE introduzir avisos novos de compilação ou de lint (Princípio I).
- **FR-012**: O README DEVE ser atualizado na seção de estrutura do código para refletir os módulos
  `logics`, `gui` e o papel de `src/controller.rs`.

### Key Entities

- **Estado da aplicação**: o último estado conhecido da fonte (medições, setpoints, memórias, modo
  CC/CV, saída, acumuladores, identificação), o estado da conexão e as preferências da sessão
  (intervalo de leitura, cores).
- **Setpoint em edição**: os valores de tensão e corrente sendo editados, com a indicação de alteração
  pendente e o grupo de memória selecionado.
- **Histórico de amostras**: sequência limitada de tensão, corrente e potência usada pelo gráfico.
- **Sessão de comunicação**: a conversa com a fonte numa porta serial. Recebe comandos (definir
  valores, ligar e desligar saída, salvar memória, mudar intervalo, desconectar) e devolve eventos
  (dados recebidos, conectado, erro).
- **Escala de eixo**: topo e casas decimais de cada grandeza no gráfico, calculados a partir do maior
  valor visível.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: 100% dos passos do roteiro de verificação manual de paridade produzem o mesmo resultado
  antes e depois da refatoração.
- **SC-002**: As regras listadas na US2 (no mínimo setpoint pendente, carga de memória, relógio de
  saída, limite do histórico, escala de eixo e junção de atualizações) têm testes automáticos que
  passam numa máquina sem interface gráfica e sem a fonte conectada.
- **SC-003**: Zero referências à biblioteca de interface dentro de `logics`.
- **SC-004**: `src/controller.rs` tem no máximo cerca de 40 linhas e nenhuma regra de negócio ou
  desenho de interface.
- **SC-005**: Zero avisos novos de compilação e de lint em relação ao estado anterior à refatoração.
- **SC-006**: O tempo entre clicar em "Aplicar" e o comando chegar à fonte continua dentro de um ciclo
  de leitura, e a interface continua sem travar durante a conexão (Princípio IV).

## Verificação manual de paridade

Executar antes e depois da refatoração, com a fonte conectada:

1. Abrir o app desconectado: a porta aparece na lista, e Aplicar, Salvar e o botão de saída estão
   desabilitados.
2. Conectar: o status fica amarelo e depois verde, e as informações do aparelho aparecem.
3. Editar V e A: aparece "não aplicado". Desfazer volta aos valores da fonte. Aplicar envia os valores.
4. Passar o mouse sobre M1–M6, carregar M2 e conferir que nada foi enviado. Aplicar e conferir na
   fonte. Salvar num grupo e conferir na fonte.
5. Ligar a saída: o botão fica verde, o selo CV/CC aparece e o tempo conta. Desligar: o tempo congela.
6. Gráfico: as linhas avançam; o Update de 100 a 1000 ms muda a janela; Limpar zera; trocar cor e
   Padrão funcionam.
7. Desconectar: os dados zeram e a porta é liberada.
8. Tentar conectar numa porta inválida ou ocupada: a mensagem de erro aparece na barra superior.
9. Rodar o monitor `tests` e conferir a saída no terminal.

## Assumptions

- O protocolo da fonte (`dps150`) é considerado parte da lógica do programa e passa a ficar sob
  `logics`, como submódulo, sem perder o isolamento exigido pelo Princípio I.
- A sessão de comunicação (thread serial) é lógica do programa e fica em `logics`. O aviso de
  "redesenhe a tela" passa a ser uma notificação genérica fornecida pela `gui` ao iniciar a sessão.
- `logics` e `gui` são módulos do mesmo pacote. Separar em crates distintos fica fora do escopo.
- O intervalo de leitura fica no estado da aplicação, em `logics`. As cores escolhidas para as
  grandezas, as cores padrão e a paleta visual são preferências de apresentação e ficam em `gui`
  (veja research.md, R5; isso corrige uma versão anterior desta suposição, que punha as cores em
  `logics` e contrariava o FR-004).
- Nenhuma feature nova entra nesta refatoração. Os débitos já registrados na constituição (faixa
  de 30 V / 5 A, texto "Update" em inglês, proteções não exibidas) continuam fora do escopo.
- O binário `tests` mantém o nome e o caminho atuais.
