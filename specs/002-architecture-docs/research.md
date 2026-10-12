# Research: Documentação de arquitetura do software

**Feature**: [spec.md](./spec.md) | **Plan**: [plan.md](./plan.md) | **Date**: 2026-10-12

Não havia nenhum "NEEDS CLARIFICATION" no Technical Context. Este documento registra as decisões que a
documentação precisa tomar, e dois fatos do repositório que mudam o escopo.

---

## R0. Fatos do repositório que ajustam a spec

Medidos em 2026-10-12, no código depois do merge da feature 001 (PR #2):

| Fato | Consequência |
|---|---|
| `src/tests.rs` foi **removido** no commit `8ecd93f` ("Tests.rs removido") | O monitor de linha de comando não existe mais. A spec o listava entre os módulos (Assumptions e FR-004); isso foi corrigido junto com este plano |
| `Cargo.toml` **ainda declara** `[[bin]] name = "tests" path = "src/tests.rs"` | `cargo test`, `cargo build` e `cargo clippy --all-targets` falham com "can't find bin `tests`". Só `cargo run --bin controller` e `cargo check --bin controller` funcionam. Ver R9 |
| O README ainda cita `src/tests.rs` e `cargo run --bin tests` em 6 lugares | São links quebrados hoje (FR-013, SC-005). A documentação desta feature precisa corrigi-los |
| Há **88 funções** fora de testes em 11 arquivos (maior: `logics/app.rs`, com 41) | Define o tamanho da entrega e justifica uma conferência automática (R4) |
| Não há ferramenta de renderização de Mermaid na máquina (`mmdc` ausente; há `node`/`npx`, sem rede garantida) | A renderização dos diagramas é conferida no visualizador do repositório, não localmente (R2) |

---

## R1. Estrutura da pasta `docs`

**Decision**:

```text
docs/
├── README.md              # índice: lista os documentos e o que cada um tem
├── arquitetura.md         # visão geral: módulos, diagramas, fluxos, tipos centrais
├── manutencao.md          # quando atualizar e como conferir a cobertura
└── modulos/               # um documento por arquivo-fonte com funções
    ├── controller.md
    ├── gui-mod.md   gui-panels.md   gui-chart.md   gui-widgets.md
    └── logics-app.md  logics-protocol.md  logics-session.md
        logics-history.md  logics-scale.md  logics-format.md
```

Os módulos sem funções (`lib.rs`, `logics/mod.rs`, `gui/theme.rs`) são descritos em `arquitetura.md`.

**Rationale**:
- Um documento por arquivo-fonte espelha `src/` e deixa óbvio onde procurar (`src/logics/app.rs` →
  `docs/modulos/logics-app.md`). Isso também deixa a regra de conferência (R4) simples: o nome do
  documento vem do caminho do arquivo.
- Separar `arquitetura.md` (visão de conjunto, US1) de `modulos/` (detalhe por função, US2) permite ler
  a visão geral em poucos minutos sem passar pelas 88 entradas.
- `docs/README.md` (e não `docs/index.md`) é o arquivo que o GitHub mostra ao abrir a pasta.

**Alternatives considered**:
- *Um arquivo único gigante*: o `logics-app` sozinho tem 41 funções; um arquivo só ficaria longo demais
  para navegar e dificultaria revisão.
- *Um documento por camada* (`logics.md`, `gui.md`): mistura `app` (41 funções) com cinco módulos
  menores, e o checker teria de procurar a função dentro de um arquivo grande.
- *`rustdoc` (`cargo doc`) em vez de Markdown*: o pedido é explicitamente "docs em md", com
  explicações de funcionamento e diagramas; `rustdoc` não substitui isso. (Comentários `///` no código
  ficam fora do escopo, FR-017.)

## R2. Formato dos diagramas

**Decision**: Mermaid em blocos ` ```mermaid `, usando só três tipos: `flowchart` (módulos, dados),
`sequenceDiagram` (ação do usuário até a fonte, ciclo de vida da sessão) e `stateDiagram-v2` (estado da
conexão). ASCII nos casos de uma linha (ex.: `controller → gui → logics`).

**Rationale**: o GitHub renderiza Mermaid no próprio Markdown, sem imagem binária nem ferramenta extra
(FR-010, SC-007). Os três tipos escolhidos são os mais estáveis e cobrem tudo o que a spec pede (FR-005,
FR-011, FR-012).

**Regras para evitar erro de renderização**: identificadores de nó sem espaço, rótulos com
caracteres especiais (`()`, `:`, `/`, `>`) entre aspas, no máximo ~12 nós por diagrama, e cada
diagrama com título em texto logo acima.

**Alternatives considered**:
- *Imagens PNG/SVG geradas*: viola o FR-010 e exige ferramenta de geração.
- *ASCII em tudo*: legível em qualquer lugar, mas ruim para sequências e máquinas de estado.
- *PlantUML*: o GitHub não renderiza.

**Verificação**: sem renderizador local (R0), a conferência é visual no visualizador do repositório
(ou na prévia do editor com suporte a Mermaid). O quickstart inclui o passo e a lista dos 6 diagramas
esperados.

## R3. Convenção da entrada de função

**Decision**: cada função tem um título `###` cujo texto é **o nome qualificado entre crases**,
seguido de uma ficha fixa:

```markdown
### `Tipo::nome`

*(interna)*          ← só para funções privadas

| | |
|---|---|
| **Propósito** | uma frase |
| **Entradas** | parâmetros e o que significam |
| **Saída** | valor devolvido (ou "nada") |
| **Efeitos** | envia comando à fonte, altera estado, E/S… (ou "nenhum") |

**Como funciona**: passos numerados ou parágrafo; regra não óbvia vem com exemplo (valores → resultado).
```

Nome qualificado: `Tipo::método` para métodos (inclusive de traits implementadas, usando o tipo
que implementa, ex.: `GuiApp::ui`), e só `nome` para funções livres (ex.: `format_hhmmss`).

**Rationale**:
- Título com crases é inequívoco para o leitor e para a conferência automática (R4).
- A ficha fixa cumpre o FR-008 (propósito, entradas, saída, efeitos, funcionamento) e deixa fácil
  achar o que se procura.
- O nome qualificado evita colisão entre métodos homônimos (`new`, `tick`, `ui`, `default`).

**Alternatives considered**: lista de bullets sem título (não dá âncora nem conferência); tabela única por
módulo (ruim para explicações longas).

**Funções com regra não óbvia** (FR-009 — exemplo concreto obrigatório): `AxisScale::for_peak`,
`decimals_for`, `DPS150::process_buffer`, `DPS150::parse_data`, `DPS150::build_command` (checksum),
`History::record_if_due`, `OutputClock::tick`, `AppState::sync_edit_buffers`, `AppState::merge`,
`AppState::refresh_ports_if_due`, `AppState::load_profile`, `session_main`, `format_window`,
`draw_history_chart`, `GuiApp::ui` (cálculo das colunas).

## R4. Conferência de cobertura

**Decision**: implementar como **teste de integração em Rust**, `tests/docs_coverage.rs`, usando só a
biblioteca padrão. Ele:

1. percorre `src/**/*.rs`, ignora o bloco `#[cfg(test)]` de cada arquivo e lista as funções com nome
   qualificado (regra em R3);
2. lê `docs/modulos/<slug>.md` de cada arquivo com funções (`slug` = caminho sem `src/` e sem `.rs`,
   com `/` trocado por `-`; `mod.rs` vira `<pasta>-mod`) e extrai os títulos `### \`…\``;
3. falha apontando as funções **faltando** (no código, sem entrada) e as **sobrando** (entrada sem
   função), e documentos de módulo ausentes;
4. um segundo teste confere os links relativos em `README.md` e `docs/**/*.md`: cada destino
   precisa existir (âncoras `#…` não são verificadas).

**Rationale**:
- Roda no gate que já existe (`cargo test`, constituição, Fluxo de Desenvolvimento), então a conferência
  acontece sem ninguém lembrar dela. Atende ao FR-015 ("repetível") e ao US3.
- Não introduz Python, Node nem dependência nova (Restrições Técnicas da constituição): é só `std`.
- Fica fora do build do controlador: é um alvo de teste separado em `tests/`, não em `src/`, então
  respeita o FR-017 (nenhum código de produção alterado).
- A leitura de arquivos é rápida (menos de 1 s) e não precisa de interface gráfica nem de porta serial.

**Como o scanner identifica funções** (varredura por linha, sem analisador completo):
- mantém uma pilha de contextos por profundidade de chaves: `impl [<…>] [Trait for] Tipo` define o
  `Tipo` (sem genéricos e sem caminho: `impl<'a> egui::Widget for ReadoutRow<'a>` → `ReadoutRow`);
- reconhece `fn nome` (com `pub`, `pub(crate)`) apenas no nível direto do `impl` ou do módulo; funções
  dentro de funções ficam de fora;
- ao ver `#[cfg(test)]`, descarta o item seguinte (bloco `mod tests`) até a chaves fechar.

**Limite conhecido**: a contagem de chaves não trata chaves dentro de literais de texto desbalanceadas
(`"{"`). O código atual não tem nenhuma; para não passar despercebido, o teste verifica que a
profundidade de chaves volta a 0 no fim de cada arquivo e falha com uma mensagem clara se não voltar.

**Testes do próprio scanner** (Princípio II): o analisador é uma função pura sobre `&str`, testada com
trechos de código fixos: método de `impl`, método de trait, `impl` com genéricos, função livre,
`pub(crate)`, função aninhada ignorada e bloco `#[cfg(test)]` ignorado.

**Alternatives considered**:
- *Script Python* (a ideia inicial da spec): menor, mas adiciona uma ferramenta que o projeto não usa e
  fica fora do gate `cargo test`. A Assumption da spec foi ajustada.
- *Script bash com `rg`/`awk`*: difícil de acertar o contexto de `impl` e os blocos de teste.
- *Ferramenta de cobertura de documentação do `rustdoc`*: mede comentários `///` no código, não
  Markdown em `docs/`.
- *Sem conferência*: viola o US3 e o FR-015.

## R5. Fonte da verdade e processo de escrita

**Decision**: a documentação descreve o código **como está no HEAD** (depois do merge da feature 001).
Cada entrada é escrita lendo a função (não o plano de 001), e o texto de "como funciona" cita os passos
reais do código. Em caso de divergência entre `specs/001-*/` e o código, vale o código.

**Rationale**: a feature 001 foi implementada com pequenas diferenças em relação ao contrato (ex.:
`update_ms_mut` virou `update_ms`; `selected_port` foi adicionada). Documentar a partir do plano
reproduziria nomes que não existem.

## R6. Idioma e estilo

**Decision**: pt-BR, no mesmo tom do README. Termos do domínio mantidos como no código e na interface:
tensão, corrente, potência, saída, setpoint, memória (M1–M6), CV/CC. Nomes de tipos e funções em crases.
Cada explicação diz o **porquê** quando há decisão relevante (ex.: por que `now` é passado por
parâmetro; por que a escala usa passos 1-2-5).

**Rationale**: FR-016 e Princípio I da constituição (comentários e mensagens em pt-BR, explicando o
porquê).

## R7. Mudanças no README

**Decision**:
1. Colocar um link para `docs/README.md` logo depois da introdução e do "Sumário", e outro na seção
   "Estrutura do código" (FR-002, SC-004).
2. Remover as referências ao monitor `tests` (R0): a linha da tabela de binários, a seção "Monitor de
   linha de comando", a entrada correspondente no Sumário, a linha do diagrama `tests.rs`, a linha da
   tabela de arquivos e a limitação "Monitor `tests`". Isso é correção de fatos, necessária para o
   FR-013 e o SC-005, e não um conteúdo novo.
3. Na seção "Estrutura do código", trocar a tabela longa de arquivos por um resumo curto que aponte
   para `docs/arquitetura.md`, evitando manter a mesma lista em dois lugares.

**Rationale**: duas listas de arquivos (README e `docs/`) divergiriam com o tempo. O README fica com o
essencial e delega o detalhe à documentação.

**Alternatives considered**: manter a tabela completa no README e repetir em `docs/`: dupla manutenção.

## R8. Escopo de cada documento

| Documento | Conteúdo |
|---|---|
| `docs/README.md` | Lista dos documentos, ordem de leitura sugerida por perfil (quem quer entender / quem vai alterar / quem vai documentar) |
| `docs/arquitetura.md` | Lista de módulos e o papel de cada um; diagrama de dependências; fluxo de dados (fonte → sessão → estado → interface); fluxo de uma ação (Aplicar); ciclo de vida da sessão; estado da conexão; tipos centrais; regras de projeto (o que `logics`/`gui` não fazem) |
| `docs/manutencao.md` | Quando atualizar; como escrever uma entrada (R3); como rodar a conferência; como ler a falha |
| `docs/modulos/*.md` | Papel do módulo; o que **não** faz; lista de funções (R3); testes que o cobrem |

O protocolo de bytes (formato de pacote, `type_id`) **não** é repetido: `docs/modulos/logics-protocol.md`
aponta para a seção "Protocolo" do README e documenta só as funções.

## R9. Pré-requisito: o binário `tests` removido ainda consta no `Cargo.toml`

**Decision**: tratar como **tarefa de pré-requisito** no plano: remover do `Cargo.toml` o bloco
`[[bin]] name = "tests" path = "src/tests.rs"`.

**Rationale**:
- Sem isso, `cargo test` não compila nenhum alvo, nem o novo `tests/docs_coverage.rs`. A conferência de
  cobertura (R4), o gate 3 da constituição e a verificação do quickstart dependem de `cargo test`.
- É configuração de build, não código de produção, então não viola o FR-017. É uma correção de um
  resíduo do commit `8ecd93f`.
- Existe a alternativa de recolocar o `src/tests.rs`, mas o commit mostra que a remoção foi proposital.

**Alternatives considered**:
- *Deixar como está e rodar só `cargo test --test docs_coverage`*: também falha, porque o Cargo resolve
  todos os alvos antes.
- *Pedir ao mantenedor que corrija em outra mudança*: bloqueia esta feature por uma linha. O plano
  sinaliza a mudança para o mantenedor vetar.

**Fora do escopo, a registrar**: o Princípio I da constituição cita o "monitor exploratório `tests`" como
isento de `unwrap`/`panic!`. Com a remoção, essa frase fica sem sentido; uma emenda PATCH (1.0.2) pode
retirá-la. Fica como melhoria futura.
