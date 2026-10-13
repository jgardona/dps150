# Feature Specification: Fonte Noto Sans e rótulos legíveis nas memórias

**Feature Branch**: `feature/label-correcao`

**Created**: 2026-10-13

**Status**: Draft

**Input**: User description: "Melhorar as fontes da UI. 1. A fonte padrão da aplicação será Noto Sans. 2. Copiar a fonte Noto Sans para o projeto, em assets, para que sempre possamos compilar a aplicação com ela. 3. Os tamanhos das fontes das memórias são muito pequenos; o tamanho pode ser 16px."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Ler os campos das memórias sem esforço (Priority: P1)

No quadro de setup das memórias, os rótulos (Vset, Iset, OVP, OCP, OPP, OTP) e os valores aparecem pequenos demais, como mostra o print: o usuário precisa se aproximar da tela para ler. Os textos do quadro passam a ter **16 px**.

**Why this priority**: é o problema que o usuário vê todo dia, e é independente da troca de fonte: melhora a leitura mesmo com a fonte antiga.

**Independent Test**: abrir o programa, olhar o quadro de setup e conferir que rótulos, valores, unidades e o título têm o mesmo tamanho legível (16 px), sem cortes nem sobreposição, inclusive com os valores máximos (36,00 V, 8,200 A, 295,20 W, 99 °C).

**Acceptance Scenarios**:

1. **Given** o quadro de setup da memória selecionada, **When** o usuário o olha, **Then** rótulos, valores e unidades dos seis campos aparecem com 16 px.
2. **Given** os seis valores máximos digitados (36,00 V, 8,200 A, 36,10 V, 8,200 A, 295,20 W, 99 °C), **When** o usuário os olha, **Then** todos aparecem inteiros, sem corte e sem se sobrepor ao campo vizinho.
3. **Given** a janela no tamanho inicial, **When** o quadro cresce com o texto maior, **Then** o botão de saída e a barra inferior continuam visíveis.
4. **Given** um campo com erro, **When** o usuário o olha, **Then** continua legível em vermelho, no mesmo tamanho.

---

### User Story 2 - Toda a interface na fonte Noto Sans (Priority: P1)

Todos os textos da interface (leituras, setpoint, memórias, gráfico, barras) passam a usar a **Noto Sans**, em vez da fonte padrão da biblioteca de interface. Acentos e símbolos usados hoje (á, ã, ç, °, ·, •, …, —, ✓ quando houver) aparecem corretamente.

**Why this priority**: é o pedido principal de identidade visual e de legibilidade. É independente da US1: pode ser entregue sozinha.

**Independent Test**: abrir o programa e conferir, texto por texto, que a fonte é a Noto Sans (letras e algarismos têm o desenho dela) e que nenhum caractere aparece como quadradinho.

**Acceptance Scenarios**:

1. **Given** o programa aberto, **When** o usuário olha qualquer região da tela, **Then** todos os textos usam a Noto Sans.
2. **Given** textos com acentos e símbolos ("não aplicado", "SAÍDA DESLIGADA", "°C", "—", "…", "•"), **When** aparecem, **Then** todos os caracteres são desenhados, sem quadradinho nem lacuna.
3. **Given** valores numéricos que mudam a cada leitura (leituras V, A, W e o tempo), **When** mudam, **Then** o texto não "dança" para os lados mais do que hoje.
4. **Given** o gráfico com escalas e legenda, **When** é desenhado, **Then** os rótulos usam a mesma fonte e continuam alinhados com as colunas das escalas.

---

### User Story 3 - Compilar e rodar sem depender da fonte instalada no sistema (Priority: P1)

A fonte Noto Sans vem **dentro do projeto** (pasta `assets/`), junto com a licença dela. Quem clona o repositório compila e roda o programa com a mesma aparência, em qualquer máquina, sem instalar fonte nenhuma.

**Why this priority**: sem isso, a US2 só funciona na máquina do autor. É o motivo do item 2 do pedido.

**Independent Test**: numa máquina (ou conta) sem a Noto Sans instalada, clonar, compilar e executar: a interface aparece em Noto Sans. Depois, renomear a fonte do sistema (ou conferir que ela não é consultada) e ver que nada muda.

**Acceptance Scenarios**:

1. **Given** um repositório recém-clonado, **When** o usuário compila com `cargo build`, **Then** a compilação termina sem pedir nenhuma fonte externa.
2. **Given** um computador sem a Noto Sans instalada, **When** o programa é executado, **Then** a interface usa a Noto Sans.
3. **Given** o repositório, **When** o mantenedor procura a licença da fonte, **Then** a encontra ao lado dos arquivos da fonte, e o README informa a origem e a licença.
4. **Given** o binário `release` gerado, **When** o usuário o copia sozinho para outra pasta ou máquina, **Then** ele continua mostrando a Noto Sans (a fonte vai dentro do executável).

---

### Edge Cases

- A fonte de reserva: um caractere que a Noto Sans do projeto não tem (por exemplo, um símbolo raro numa mensagem de erro do sistema operacional) ainda DEVE aparecer, usando a fonte de reserva da biblioteca, e não como quadradinho.
- Mensagens de erro vindas do sistema (porta serial) com acentos, em pt-BR: aparecem corretas.
- Janela redimensionada para a largura mínima da coluna de controle: os campos do quadro de setup continuam inteiros (ou a coluna cresce até o mínimo necessário).
- Texto em negrito ("Setup M1", valores em destaque): se a fonte do projeto tiver só o peso normal, o destaque DEVE continuar perceptível (cor ou peso), sem deformar o texto.
- O tamanho de 16 px vale para o quadro de setup; os demais textos mantêm os tamanhos de hoje, para não quebrar o layout (veja Premissas).

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: A fonte padrão de **todos** os textos da interface DEVE ser a Noto Sans.
- **FR-002**: Os arquivos da Noto Sans DEVEM ficar no repositório, em `assets/`, e DEVEM ser incluídos no executável, de modo que compilar e rodar nunca dependa de fonte instalada no sistema.
- **FR-003**: A licença da Noto Sans DEVE ficar no repositório ao lado dos arquivos da fonte, e o README DEVE informar a origem e a licença.
- **FR-004**: Os textos do quadro de setup das memórias (título, rótulos, valores, unidades) DEVEM ter **16 px**.
- **FR-005**: Com o texto de 16 px, os seis campos DEVEM caber inteiros na coluna de controle, inclusive com os valores máximos (36,00 V, 8,200 A, 36,10 V, 8,200 A, 295,20 W, 99 °C), sem corte nem sobreposição.
- **FR-006**: Depois da mudança, o botão de saída e a barra inferior DEVEM continuar visíveis na janela inicial, e o tamanho inicial da janela DEVE ser ajustado se for preciso.
- **FR-007**: Todos os caracteres de pt-BR usados na interface (acentos, ç, °) e os símbolos já usados (•, …, —, ·) DEVEM ser desenhados com a Noto Sans, sem quadradinhos.
- **FR-008**: Um caractere ausente na fonte do projeto DEVE cair numa fonte de reserva, em vez de aparecer como quadradinho.
- **FR-009**: Os textos de destaque (hoje em negrito) DEVEM continuar destacados.
- **FR-010**: A troca de fonte NÃO DEVE mudar o comportamento do programa: nenhum comando, valor, limite ou regra muda.
- **FR-011**: O desempenho NÃO DEVE piorar de forma perceptível: o programa abre em tempo parecido e continua redesenhando só quando precisa; o aumento do executável DEVE ser registrado.
- **FR-012**: O README e a documentação de `docs/` DEVEM ser atualizados na mesma entrega (nova pasta `assets/`, fonte padrão, tamanho do quadro de setup, tamanho inicial da janela e funções novas ou alteradas), com a conferência de documentação passando.

### Key Entities

- **Fonte da interface**: a Noto Sans, com seus arquivos e licença guardados no projeto; é a fonte de todos os textos.
- **Texto do quadro de setup**: título, rótulo, valor e unidade dos seis campos de memória, todos com 16 px.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: 100% dos textos visíveis da interface aparecem em Noto Sans (conferência visual região por região: leituras, setpoint, memórias, gráfico, barras e mensagens).
- **SC-002**: 0 caracteres aparecem como quadradinho nos textos da interface, incluindo todas as mensagens fixas em pt-BR.
- **SC-003**: Uma pessoa lê os seis rótulos e valores do quadro de setup, sem aproximar o rosto da tela, a uma distância normal de uso (cerca de 50 cm), em 100% das tentativas de uma amostra de leitura.
- **SC-004**: Os seis valores máximos cabem inteiros no quadro (0 cortes e 0 sobreposições), na janela inicial e na largura mínima da coluna.
- **SC-005**: Clonar, compilar e executar numa máquina sem a Noto Sans instalada mostra a interface em Noto Sans em 100% das tentativas, sem passos extras.
- **SC-006**: O tempo para a janela aparecer não aumenta mais que 0,5 s em relação à versão anterior, e o aumento do tamanho do executável fica registrado na documentação.
- **SC-007**: Nenhum comportamento muda: todos os testes existentes continuam passando sem alteração de resultado.

## Assumptions

- "Fontes das memórias" é o **quadro de setup das memórias** (o do print: título "Setup Mn", rótulos Vset, Iset, OVP\*, OCP\*, OPP\*, OTP\* e os valores). O tamanho de 16 px vale para todo o texto desse quadro. A linha com os botões M1–M6 e **Salvar** fica no tamanho da interface (não foi citada); se você a quiser em 16 px também, é um ajuste simples.
- Os demais textos mantêm os tamanhos de hoje (por exemplo, leituras de 34 px, botão de saída de 20 px e escalas do gráfico de 12 px). O que muda neles é só a fonte.
- A Noto Sans é distribuída sob a licença SIL Open Font License 1.1, que permite incluí-la no projeto e no executável; o texto da licença acompanha os arquivos.
- A Noto Sans do projeto inclui o peso normal e o negrito; havendo só um peso, o destaque é feito por cor e tamanho.
- O quadro maior exige mais altura da janela; o tamanho inicial pode crescer (hoje 1000 × 700 px) e isso é aceito.
- O tamanho de fonte de 16 px é em unidades da interface (pontos lógicos), o mesmo sistema dos tamanhos já usados.
- Fora do escopo: permitir ao usuário trocar a fonte, escolher outros idiomas e alterar a hierarquia de tamanhos dos demais textos.
- A feature segue a constituição: textos em pt-BR (Princípio III), nenhuma dependência nova sem justificativa (Restrições Técnicas), desempenho mantido (Princípio IV) e README atualizado na mesma entrega.
