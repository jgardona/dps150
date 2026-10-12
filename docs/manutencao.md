# Manutenção da documentação

Esta documentação só serve se continuar igual ao código. Para isso existe uma **conferência automática**,
que roda junto com os testes do projeto e falha quando as duas coisas se separam. Esta página explica quando
atualizar, como escrever uma entrada e como ler a falha.

## Quando atualizar

Atualize a documentação **na mesma mudança** que altera o código:

| Mudança no código | O que atualizar |
|---|---|
| **Função nova** (pública ou privada) | Escrever a entrada dela no documento do módulo |
| **Função renomeada** | Renomear o título da entrada e rever o texto |
| **Função removida** | Apagar a entrada |
| **Função que mudou de comportamento** | Rever propósito, entradas, saída, efeitos, "Como funciona" e o exemplo |
| **Módulo novo** (um arquivo `.rs` com funções) | Criar o documento dele (veja [módulo novo](#documentar-um-módulo-novo)) e incluí-lo no índice e na visão geral |
| **Tipo central novo ou alterado** | Rever a tabela "Tipos centrais" em [arquitetura.md](arquitetura.md#tipos-centrais) |
| **Mudança de fluxo, de dependência ou de estados** | Rever os diagramas de [arquitetura.md](arquitetura.md) |

A conferência cobre só a **lista de funções** e os **links**. Ela **não** consegue saber se o texto de uma entrada
ainda está certo depois de uma mudança de comportamento: isso depende de quem altera o código.

## Como escrever uma entrada de função

Cada função tem uma entrada no documento do módulo (`docs/modulos/<módulo>.md`), na ordem em que aparece no
arquivo. O modelo é este:

````markdown
### `AppState::apply_setpoints`

| | |
|---|---|
| **Propósito** | Envia à fonte a tensão e a corrente editadas e encerra a edição pendente. |
| **Entradas** | nenhuma (usa os campos do setpoint) |
| **Saída** | nada |
| **Efeitos** | envia `SetVoltage` e `SetCurrent` pelo canal de comandos; a edição deixa de estar pendente |

**Como funciona**: 1. … 2. …
````

Regras:

- **O título é a chave da conferência.** Uma linha só, no formato `` ### `Nome` ``: exatamente três `#`, um par de
  crases, nada antes nem depois.
- **Funções privadas** levam a linha `*(interna)*` logo abaixo do título.
- **"Como funciona"** descreve os passos reais do código, em pt-BR, e explica o *porquê* quando há uma decisão
  que não é óbvia.
- **Regra não óbvia** (cálculo, ressincronização, escala, limites…) exige **`**Exemplo:**`** com valores de
  entrada e o resultado.
- Os **testes** não têm entrada: o que eles cobrem é resumido na seção "Testes" do documento do módulo.

### O nome do título

| A função é… | Nome do título | Exemplo |
|---|---|---|
| Um método de `impl Tipo` | `Tipo::método` | `AppState::new` |
| Um método de trait (`impl Trait for Tipo`) | `Tipo::método`, com o tipo que **implementa** | `AppState::default`, `GuiApp::ui` |
| Uma função solta do módulo | só o nome | `format_hhmmss` |

Genéricos, lifetimes e caminhos são descartados: `impl<'a> egui::Widget for ReadoutRow<'a>` vira `ReadoutRow`.

## Documentar um módulo novo

1. Crie `docs/modulos/<slug>.md`. O `slug` é o caminho do arquivo **sem `src/` e sem `.rs`**, com `/` trocado por
   `-`:

   | Arquivo | Documento |
   |---|---|
   | `src/logics/app.rs` | `docs/modulos/logics-app.md` |
   | `src/gui/mod.rs` | `docs/modulos/gui-mod.md` |
   | `src/controller.rs` | `docs/modulos/controller.md` |

2. Comece pelo **Papel** e pelo **O que não faz**, depois os **Tipos** do módulo e as **Funções**.
3. Inclua o módulo na tabela de [docs/README.md](README.md), na tabela de módulos e na lista de
   documentação de funções de [arquitetura.md](arquitetura.md) e, se a dependência entre módulos mudou, no
   diagrama de dependências.

Arquivos sem nenhuma função (por exemplo, `lib.rs` e `gui/theme.rs`) **não** precisam de documento próprio: ficam
descritos na visão geral.

## Rodar a conferência

```bash
cargo test --test docs_coverage
```

Ela também roda junto com o `cargo test` completo. Não precisa de interface gráfica nem da fonte conectada, e
termina em menos de um segundo. O código está em [`tests/docs_coverage.rs`](../tests/docs_coverage.rs) e usa só a
biblioteca padrão.

Ela faz duas verificações:

- **`funcoes_documentadas`**: compara, arquivo por arquivo, as funções de `src/**/*.rs` com os títulos de
  `docs/modulos/<slug>.md`.
- **`links_relativos_existem`**: confere se todo link relativo do `README.md` e dos documentos de `docs/`
  aponta para um arquivo que existe. As âncoras (`#algo`) não são verificadas.

## Ler a falha

A mensagem lista um problema por linha, em ordem:

```text
documentação fora de sincronia com o código:
  faltando em docs/modulos/logics-app.md: AppState::apply_setpoints
  sobrando em docs/modulos/logics-app.md: AppState::apply_setpoint
  documento ausente: docs/modulos/gui-chart.md (src/gui/chart.rs tem 2 funções)
```

| Mensagem | Significado | O que fazer |
|---|---|---|
| `faltando em <doc>: <nome>` | Há essa função no código e nenhum título para ela | Escrever a entrada |
| `sobrando em <doc>: <nome>` | Há esse título e nenhuma função com o nome | Renomear o título (se a função foi renomeada) ou apagar a entrada |
| `título duplicado em <doc>: <nome>` | Dois títulos iguais no mesmo documento | Remover o repetido |
| `documento ausente: <doc> (<arquivo> tem N funções)` | Um arquivo com funções não tem documento | Criar o documento (veja [módulo novo](#documentar-um-módulo-novo)) |
| `documento sem módulo: <doc>` | Há um documento, mas o arquivo-fonte não tem mais funções | Apagar ou renomear o documento |
| `link quebrado em <arquivo>:<linha> → <destino>` | O link aponta para algo que não existe | Corrigir o caminho |
| `chaves desbalanceadas …` | O scanner não conseguiu contar as chaves do arquivo `.rs` | Veja os limites abaixo |

Um bom truque ao renomear uma função: mudar o título **antes** de rodar a conferência. Se ela mostrar um par
`faltando` e `sobrando` com nomes parecidos, foi só um nome diferente.

## Limites da conferência

O scanner lê o código linha a linha, sem um analisador completo de Rust. Por isso:

- **Funções aninhadas** (declaradas dentro do corpo de outra função) são ignoradas. Elas são detalhe da função
  que as contém e não precisam de entrada própria.
- **Itens marcados `#[cfg(test)]`** e o bloco `mod tests` são ignorados: testes não entram na documentação.
- Comentários, textos e literais de caractere são desconsiderados na contagem de chaves, então `"{"` em um
  texto não atrapalha. Se mesmo assim a contagem falhar, o erro `chaves desbalanceadas` aponta o arquivo.
- A conferência não lê o **conteúdo** das entradas: ela não sabe se "Como funciona" está correto nem se há
  exemplo. Esse cuidado é de quem faz a mudança e de quem revisa.
