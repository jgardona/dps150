# Modelo de dados: estilo do botão de saída

**Feature**: [spec.md](./spec.md) | **Plan**: [plan.md](./plan.md)

Esta feature não tem dados novos nem muda estado: é só estilo.

## Constantes novas (`gui::theme`)

| Constante | Tipo | Valor | Uso |
|---|---|---|---|
| `OUTPUT_BUTTON_RADIUS` | `u8` | `5` | Raio dos cantos do botão de saída, em pixels lógicos |
| `OUTPUT_BOTTOM_MARGIN` | `f32` | `10.0` | Folga entre a borda de baixo do botão de saída e a barra inferior |

## Estado existente que a tela lê (sem mudança)

| Item | Onde | Uso na tela |
|---|---|---|
| `output_on()` | `AppState` | texto, cor e dica do botão |
| `is_connected()` | `AppState` | habilita o botão |
| `toggle_output()` | `AppState` | ação do clique (inalterada) |
