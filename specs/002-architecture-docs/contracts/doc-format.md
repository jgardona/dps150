# Contrato: formato da documentação e da conferência

**Feature**: [spec.md](../spec.md) | **Data model**: [data-model.md](../data-model.md)

Este é o contrato entre quem escreve a documentação e a conferência automática. A conferência só
confia no que está descrito aqui. Se o formato mudar, este arquivo, a conferência e
`docs/manutencao.md` mudam juntos.

---

## 1. Layout de arquivos

```text
README.md                      # tem link para docs/README.md (FR-002)
docs/
├── README.md                  # índice (FR-003)
├── arquitetura.md             # visão geral e diagramas (FR-004..006, 011, 012)
├── manutencao.md              # orientação de manutenção (FR-014)
└── modulos/<slug>.md          # um por arquivo-fonte com funções (FR-007, 008)
tests/
└── docs_coverage.rs           # conferência (FR-015) — só biblioteca padrão
```

`slug` = caminho do `.rs` sem `src/` e sem `.rs`, com `/` trocado por `-`. O mapa completo está no
data model.

## 2. Entrada de função (o que a conferência lê)

Uma entrada começa com um título de nível 3 cujo conteúdo é **só** o nome qualificado entre crases:

```markdown
### `AppState::apply_setpoints`
```

Regras do título (a conferência extrai com a expressão `^### \`([^\`]+)\`\s*$`):

- exatamente `###`, uma linha, um par de crases, sem texto antes ou depois;
- nome qualificado: `Tipo::método` ou `função_livre`;
- único dentro do documento.

O restante da entrada (ficha e "Como funciona") é livre para a conferência, mas segue o modelo abaixo
para o leitor encontrar o que procura:

```markdown
### `AppState::apply_setpoints`

| | |
|---|---|
| **Propósito** | Envia à fonte a tensão e a corrente editadas e encerra a edição pendente. |
| **Entradas** | nenhuma (usa os campos do setpoint) |
| **Saída** | nada |
| **Efeitos** | envia `SetVoltage` e `SetCurrent` pelo canal de comandos; `dirty` passa a falso |

**Como funciona**: 1. … 2. …
```

Funções privadas têm a linha `*(interna)*` logo abaixo do título. Entradas de funções com regra não
óbvia (lista na pesquisa R3) incluem **Exemplo:** com valores de entrada e resultado.

## 3. Diagramas obrigatórios

Todos em `docs/arquitetura.md`, em blocos ` ```mermaid ` com um título de uma linha acima:

| # | Diagrama | Tipo | Mostra |
|---|---|---|---|
| D1 | Dependências entre módulos | `flowchart` | `controller → gui → logics` e as dependências internas de `logics` |
| D2 | Fluxo de dados | `flowchart` | fonte ⇄ thread da sessão ⇄ canais ⇄ `AppState` ⇄ `GuiApp` |
| D3 | Ação do usuário até a fonte | `sequenceDiagram` | clique em "Aplicar" até o comando na porta serial, e a resposta de volta |
| D4 | Ciclo de vida da sessão | `sequenceDiagram` | abrir a porta, DTR/RTS, espera de 2 s, inicialização, laço, desconectar |
| D5 | Estado da conexão | `stateDiagram-v2` | `Disconnected`, `Connecting`, `Connected`, `Error` e as transições |
| D6 | Ciclo por quadro | `flowchart` | o que `GuiApp::ui` faz a cada quadro, em ordem |

Regras: ≤ ~12 nós; rótulos com `()`, `:`, `/` ou `>` entre aspas; nomes do código nos rótulos.

## 4. Conferência de cobertura — `tests/docs_coverage.rs`

**Entrada**: a árvore do repositório (`src/**/*.rs`, `docs/**/*.md`, `README.md`), relida a cada
execução a partir de `CARGO_MANIFEST_DIR`.

**Alvos de teste**:

| Teste | Passa quando |
|---|---|
| `funcoes_documentadas` | o conjunto de funções do código é igual ao conjunto de títulos, módulo a módulo |
| `links_relativos_existem` | todo link relativo de `README.md` e `docs/**/*.md` aponta para um arquivo existente |
| testes do scanner | o analisador de código acerta os casos fixos (ver abaixo) |

**Saída em caso de falha** (mensagem do `panic!`, uma linha por problema, ordenada):

```text
documentação fora de sincronia com o código:
  faltando em docs/modulos/logics-app.md: AppState::apply_setpoints
  sobrando em docs/modulos/logics-app.md: AppState::apply_setpoint
  documento ausente: docs/modulos/gui-chart.md (src/gui/chart.rs tem 2 funções)
  link quebrado em docs/arquitetura.md:42 → modulos/logics-apps.md
```

**Regras do scanner de funções** (função pura sobre o texto de um `.rs`, testada com trechos fixos):

| Entrada | Resultado |
|---|---|
| `impl AppState { pub fn new() … }` | `AppState::new` |
| `impl Default for AppState { fn default() … }` | `AppState::default` |
| `impl<'a> egui::Widget for ReadoutRow<'a> { fn ui(…) … }` | `ReadoutRow::ui` |
| `pub(crate) fn card(…)` no nível do módulo | `card` |
| `fn dentro_de_outra()` aninhada em uma função | ignorada |
| `#[cfg(test)] mod tests { … }` e `#[cfg(test)] fn …` | ignorados |

Falha adicional: a contagem de `{`/`}` não volta a 0 no fim de um arquivo.

**Garantias**: não lê nada fora do repositório, não usa rede nem interface gráfica, não escreve
arquivos e termina em menos de 1 s.

## 5. Contrato do README

`README.md` tem, **fora de blocos de código**, ao menos um link `[…](docs/README.md)`:

1. logo depois da introdução (antes do "Sumário"), e
2. na seção "Estrutura do código".

O README não tem links nem comandos para `src/tests.rs` ou `--bin tests`.
