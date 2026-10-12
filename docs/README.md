# Documentação do controlador DPS150/IPS3608

Esta pasta explica **como o software é organizado por dentro**: quais são os módulos, o que cada um faz,
como os dados e as ações circulam entre eles e, para cada função, o que ela faz e como funciona.

Para **usar** o controlador (compilar, conectar, operar a fonte), veja o [README do projeto](../README.md).

## Documentos

| Documento | Conteúdo |
|---|---|
| [arquitetura.md](arquitetura.md) | Visão geral: lista de módulos e o papel de cada um, diagramas (dependências, fluxo de dados, ação do usuário até a fonte, sessão serial, estado da conexão, ciclo por quadro), tipos centrais e regras de projeto |
| [manutencao.md](manutencao.md) | Quando atualizar a documentação, como escrever uma entrada de função, como rodar a conferência automática e como ler a falha dela |

### Módulos

Um documento por arquivo-fonte, com o papel do módulo, o que ele não faz, os tipos dele e **todas as funções**
explicadas (propósito, entradas, saída, efeitos e como funcionam).

| Documento | Arquivo | Conteúdo |
|---|---|---|
| [controller.md](modulos/controller.md) | `src/controller.rs` | O ponto de entrada (`main`) |
| [gui-mod.md](modulos/gui-mod.md) | `src/gui/mod.rs` | `GuiApp`, o ciclo de cada quadro e o layout em colunas (inclui as constantes de `theme`) |
| [gui-panels.md](modulos/gui-panels.md) | `src/gui/panels.rs` | Barra superior, coluna de controle e barra inferior |
| [gui-chart.md](modulos/gui-chart.md) | `src/gui/chart.rs` | Painel do gráfico: legenda, seletor de cor, séries e escalas |
| [gui-widgets.md](modulos/gui-widgets.md) | `src/gui/widgets.rs` | Componentes reaproveitados pelos painéis |
| [logics-app.md](modulos/logics-app.md) | `src/logics/app.rs` | `AppState`: estado e regras da aplicação |
| [logics-protocol.md](modulos/logics-protocol.md) | `src/logics/protocol.rs` | Protocolo da fonte: comandos, checksum e leitura do fluxo serial |
| [logics-session.md](modulos/logics-session.md) | `src/logics/session.rs` | A thread da sessão serial e os canais de comandos e eventos |
| [logics-history.md](modulos/logics-history.md) | `src/logics/history.rs` | O histórico limitado de amostras do gráfico |
| [logics-memory_setup.md](modulos/logics-memory_setup.md) | `src/logics/memory_setup.rs` | Os seis campos de setup das memórias: máximos, edição pendente e validação |
| [logics-scale.md](modulos/logics-scale.md) | `src/logics/scale.rs` | O cálculo das escalas dos eixos |
| [logics-format.md](modulos/logics-format.md) | `src/logics/format.rs` | A formatação de durações |

## Ordem de leitura

- **Quero entender a arquitetura**: leia [arquitetura.md](arquitetura.md), do começo ao fim. Em poucos
  minutos você sabe onde fica cada responsabilidade.
- **Vou alterar um módulo**: comece pela tabela de módulos em [arquitetura.md](arquitetura.md#módulos)
  e vá ao documento do módulo que vai mexer, na lista acima.
- **Vou manter a documentação**: leia [manutencao.md](manutencao.md). A conferência automática
  (`cargo test --test docs_coverage`) avisa quando uma função do código fica sem entrada, ou o contrário.
