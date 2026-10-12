# Feature Specification: Documentação de arquitetura do software

**Feature Branch**: `002-architecture-docs`

**Created**: 2026-10-12

**Status**: Draft

**Input**: User description: "Essa é uma feature de documentação. 1. Criaremos uma pasta docs para guardar
as documentações em md. 2. README.md terá um link para essas documentações. 3. As documentações serão a
respeito de arquitetura do software: a. Uma listagem com os diagramas dos módulos e explicações do que
eles fazem. b. Cada função documentada de cada módulo e como elas trabalham."

## Contexto

Depois da refatoração da feature 001, o código está dividido nos módulos `logics` (regras, protocolo e
sessão serial) e `gui` (interface), com `src/controller.rs` como ponto de entrada. O README descreve o
uso do controlador e uma visão geral da estrutura, mas não explica como as peças se encaixam nem o que
cada função faz. Quem quiser mexer no código precisa ler tudo para descobrir isso.

Esta feature é **somente de documentação**. Nenhum código de produção é alterado, e o comportamento do
controlador continua o mesmo. A pasta `docs` e o formato Markdown são requisitos explícitos do
mantenedor e por isso aparecem na especificação.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Entender a arquitetura em poucos minutos (Priority: P1)

Uma pessoa que chega ao repositório (mantenedor voltando depois de meses, ou colaborador novo) abre o
README, segue o link para a documentação e, em poucos minutos, entende quais são os módulos, o que cada
um faz, como se relacionam e por onde passam os dados entre a interface e a fonte de bancada.

**Why this priority**: é o valor central da feature. Sem uma visão de conjunto, a documentação de cada
função (US2) fica solta e difícil de usar.

**Independent Test**: dar o README a alguém que não conhece o código e pedir que explique, usando só a
documentação, onde fica a regra do setpoint pendente, onde fica o desenho do gráfico e como um comando
chega à fonte. A pessoa precisa acertar os três sem abrir o código.

**Acceptance Scenarios**:

1. **Given** o README do projeto, **When** a pessoa procura a documentação, **Then** encontra um link
   visível para a pasta de documentação, sem precisar navegar pelos arquivos do repositório.
2. **Given** a documentação de arquitetura, **When** a pessoa abre a visão geral, **Then** vê a lista de
   todos os módulos, um diagrama de como eles se relacionam e uma explicação curta do papel de cada um.
3. **Given** a visão geral, **When** a pessoa segue o caminho de uma ação do usuário (por exemplo,
   "Aplicar" um setpoint) até a fonte, **Then** a documentação mostra cada módulo por onde a ação passa,
   na ordem.

---

### User Story 2 - Saber o que cada função faz e como funciona (Priority: P2)

Quem vai alterar um módulo consulta a documentação dele e encontra todas as funções, com o propósito de
cada uma, o que ela recebe e devolve, quais efeitos tem (por exemplo, enviar um comando à fonte) e como
funciona por dentro, sem precisar decifrar o código.

**Why this priority**: o mantenedor pediu explicitamente que cada função de cada módulo fosse documentada.
Depende da visão geral (US1) para dar contexto.

**Independent Test**: sortear 10 funções do código e conferir que cada uma tem entrada na documentação do
seu módulo, com propósito, entradas, saída, efeitos e funcionamento descritos.

**Acceptance Scenarios**:

1. **Given** qualquer módulo do projeto, **When** a pessoa abre a documentação dele, **Then** existe uma
   entrada para cada função, pública ou privada, daquele módulo.
2. **Given** a entrada de uma função, **When** a pessoa a lê, **Then** encontra o que a função faz, o que
   recebe, o que devolve, os efeitos colaterais e uma explicação de como ela trabalha.
3. **Given** uma função com regra não óbvia (por exemplo, o cálculo da escala do gráfico ou a
   ressincronização do protocolo), **When** a pessoa lê a entrada, **Then** a regra aparece explicada em
   linguagem clara, com um exemplo.

---

### User Story 3 - Manter a documentação confiável ao longo do tempo (Priority: P3)

O mantenedor sabe quando e como atualizar a documentação, e consegue conferir se ela ainda corresponde ao
código.

**Why this priority**: documentação que envelhece sem aviso perde a confiança. É menos urgente que
escrevê-la, mas evita que ela fique obsoleta.

**Independent Test**: renomear ou adicionar uma função de teste e ver que a conferência indica a
documentação como desatualizada.

**Acceptance Scenarios**:

1. **Given** a pasta de documentação, **When** a pessoa procura a regra de atualização, **Then** encontra
   uma orientação curta dizendo quando a documentação deve ser atualizada junto com o código.
2. **Given** uma função nova, renomeada ou removida no código, **When** o mantenedor faz a conferência de
   cobertura, **Then** ela aponta o que falta ou sobra na documentação.

---

### Edge Cases

- **Função privada ou interna**: também entra na documentação, marcada como interna, porque o pedido é
  documentar cada função de cada módulo.
- **Função de teste**: não entra na documentação de funções. O que os testes cobrem aparece como
  descrição geral por módulo.
- **Código que muda depois da documentação**: a regra de atualização e a conferência (US3) existem para
  detectar essa divergência.
- **Módulo sem funções próprias** (apenas declara submódulos): documentado só na visão geral.
- **Links quebrados**: todo link entre documentos e para arquivos de código deve apontar para algo que
  existe.
- **Leitor sem ferramenta de diagramas**: os diagramas devem ser legíveis diretamente no navegador do
  repositório, sem instalar nada.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: O repositório DEVE ter uma pasta `docs` na raiz, contendo toda a documentação desta feature
  em arquivos Markdown.
- **FR-002**: O `README.md` DEVE ter um link visível para a documentação em `docs`, em local fácil de
  achar (por exemplo, perto do início ou na seção de estrutura do código).
- **FR-003**: A pasta `docs` DEVE ter um documento de entrada (índice) que liste todos os outros
  documentos, com uma linha explicando o conteúdo de cada um.
- **FR-004**: A documentação DEVE ter uma visão geral da arquitetura com a **lista de todos os módulos**
  do projeto: `logics`, `gui` e o ponto de entrada, incluindo seus submódulos.
- **FR-005**: A visão geral DEVE conter **diagramas** que mostrem: (a) a relação entre os módulos e o
  sentido das dependências; (b) o caminho dos dados entre a fonte, a sessão serial, o estado da aplicação
  e a interface; (c) o caminho de uma ação do usuário até a fonte.
- **FR-006**: Cada módulo DEVE ter uma explicação do que faz, de qual é a sua responsabilidade e do que
  ele **não** faz.
- **FR-007**: Cada módulo DEVE ter um documento (ou seção) próprio, listando **todas as funções** dele,
  públicas e privadas, excluindo os testes.
- **FR-008**: Cada função DEVE ser documentada com: propósito, entradas, saída, efeitos colaterais (quando
  houver) e explicação de **como ela funciona**.
- **FR-009**: As funções com regra não óbvia DEVEM trazer a regra descrita passo a passo, com pelo menos um
  exemplo concreto (valores de entrada e resultado).
- **FR-010**: Os diagramas DEVEM ser escritos em texto dentro dos arquivos Markdown (sem imagens binárias
  geradas por ferramentas externas) e DEVEM ser exibidos corretamente pelo visualizador de Markdown do
  repositório.
- **FR-011**: Os tipos de dados centrais (estado da aplicação, estado da conexão, editor de setpoint,
  relógio de saída, histórico, escala de eixo, comandos e eventos da sessão) DEVEM ser descritos, com a
  máquina de estados da conexão representada em diagrama.
- **FR-012**: A documentação DEVE descrever o ciclo de vida da sessão serial: abertura da porta,
  inicialização, laço de leitura e comandos, e encerramento.
- **FR-013**: Todo link da documentação (entre documentos, para o README e para arquivos de código) DEVE
  apontar para um destino existente.
- **FR-014**: A pasta `docs` DEVE ter uma orientação curta de manutenção: quando atualizar a documentação
  e como conferir a cobertura das funções.
- **FR-015**: DEVE existir uma conferência repetível da cobertura que compare as funções do código com as
  funções documentadas e aponte as que faltam ou sobram.
- **FR-016**: Os textos DEVEM estar em pt-BR, no idioma do projeto, e DEVEM explicar o *porquê* das
  decisões relevantes, não só o *quê*.
- **FR-017**: Nenhum código de produção DEVE ser alterado por esta feature. Mudanças em `src/` ficam
  restritas, se houver, a comentários de documentação. A conferência (FR-015) e a correção do
  `Cargo.toml` descrita em Assumptions não são código de produção.

### Key Entities

- **Documento de entrada (índice)**: página inicial da pasta `docs`, com a lista dos documentos.
- **Visão geral da arquitetura**: documento com a lista de módulos, os diagramas e o caminho dos dados e
  das ações.
- **Documento de módulo**: descreve a responsabilidade de um módulo e lista as funções dele.
- **Entrada de função**: registro de uma função, com propósito, entradas, saída, efeitos e funcionamento.
- **Diagrama**: representação em texto de módulos, dados, estados ou sequências.
- **Conferência de cobertura**: verificação que compara o código e a documentação.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Uma pessoa nova no projeto, lendo só a documentação, consegue responder às três perguntas do
  teste independente da US1 (onde fica o setpoint pendente, onde fica o desenho do gráfico e como um
  comando chega à fonte) em até 10 minutos, sem abrir o código.
- **SC-002**: 100% das funções de produção do código (públicas e privadas, sem testes) têm entrada na
  documentação do seu módulo.
- **SC-003**: 100% das entradas de função têm propósito, entradas, saída, efeitos e funcionamento
  preenchidos.
- **SC-004**: O README leva à documentação com **1 clique**, e o índice leva a qualquer documento com 1
  clique.
- **SC-005**: Zero links quebrados na documentação e no README.
- **SC-006**: 100% dos módulos listados na visão geral têm um documento (ou seção) próprio.
- **SC-007**: Os diagramas aparecem renderizados no visualizador de Markdown do repositório, sem
  instalação de nada.
- **SC-008**: A conferência de cobertura, rodada logo após a entrega, não aponta funções faltando nem
  sobrando.
- **SC-009**: Zero linhas de código de produção alteradas pela feature.

## Assumptions

- A pasta `docs` fica na raiz do repositório, ao lado de `src` e `specs`.
- A documentação é escrita em pt-BR, como o README e a constituição do projeto.
- Os diagramas são escritos em texto: diagramas de blocos em Mermaid, que o GitHub renderiza, e ASCII nos
  casos simples. Isso atende ao FR-010 sem imagens binárias.
- "Função" inclui funções livres e métodos (de `impl`). Funções dentro de `#[cfg(test)]` ficam de fora.
- Os módulos documentados são os existentes hoje, depois da feature 001 e do merge do PR #2:
  `controller`, `lib`, `logics` (`protocol`, `session`, `app`, `history`, `scale`, `format`) e `gui`
  (`mod`, `panels`, `chart`, `widgets`, `theme`). O monitor de linha de comando (`src/tests.rs`) foi
  removido no commit `8ecd93f` e **não** é documentado. O protocolo da fonte já é detalhado no README
  (formato de pacote, campos); a documentação de arquitetura faz referência a ele em vez de repeti-lo.
- O `README.md` ainda cita o monitor removido (links para `src/tests.rs` e `cargo run --bin tests`).
  Esses trechos viram links quebrados e são corrigidos por esta feature, para cumprir o FR-013.
- O `Cargo.toml` ainda declara o binário `tests`, o que impede `cargo test` e `cargo build` de
  compilar. Remover esse bloco é um pré-requisito desta feature (veja research.md, R9), porque a
  conferência de cobertura roda dentro de `cargo test`.
- A conferência de cobertura é um teste automático do próprio projeto, executado por `cargo test`, sem
  dependências novas e fora do build do controlador (veja research.md, R4). Isso substitui a ideia
  inicial de um script avulso.
- Esta feature não muda o comportamento do controlador, a interface nem os testes automáticos.
- A documentação de funções descreve o código **como está depois da feature 001**. Se o código mudar
  antes desta feature ser implementada, a documentação segue o código atual.
