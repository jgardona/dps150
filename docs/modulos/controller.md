# `controller` — ponto de entrada

Arquivo: [`src/controller.rs`](../../src/controller.rs) · Visão geral: [arquitetura](../arquitetura.md)

## Papel

É o binário `controller`: o programa que você executa com `cargo run --bin controller`. Só abre a janela e
entrega o controle à interface gráfica ([`GuiApp`](gui-mod.md)).

## O que não faz

Não tem regra de negócio nem desenha nada. Todo o resto do programa está nos módulos `logics` e `gui`,
que ele acessa pela biblioteca `dps150`.

## Constantes

| Constante | Valor | Uso |
|---|---|---|
| `WIDTH` | 1000 px | Largura inicial da janela |
| `HEIGHT` | 700 px | Altura inicial da janela (o quadro de setup das memórias precisou de mais espaço vertical) |

## Funções

### `main`

| | |
|---|---|
| **Propósito** | Iniciar a aplicação gráfica. |
| **Entradas** | nenhuma |
| **Saída** | `eframe::Result`: sucesso quando a janela é fechada, ou o erro se o egui não conseguiu iniciar. |
| **Efeitos** | Abre a janela do programa e roda o laço da interface até ela ser fechada. |

**Como funciona**: monta as opções da janela (`NativeOptions`) com o tamanho inicial de `WIDTH × HEIGHT` e chama
`eframe::run_native` com o título "DPS150 Controller" e uma função que cria o `GuiApp::new()`. A partir daí,
o egui chama `GuiApp::ui` a cada quadro. O retorno do `main` é o retorno de `run_native`, então um erro de
inicialização do egui aparece como erro do programa.
