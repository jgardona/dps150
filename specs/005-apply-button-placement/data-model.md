# Modelo de dados: botão Aplicar na linha dos campos

**Feature**: [spec.md](./spec.md) | **Plan**: [plan.md](./plan.md)

Esta feature não tem dados novos nem muda estado: é só layout.

## Constante nova (`gui::theme`)

| Constante | Valor | Uso |
|---|---|---|
| `FIELD_GAP` | `10.0` | Espaço extra entre os grupos da linha do Setpoint: entre o campo V e o rótulo A, e entre o campo A e o botão **Aplicar** |

## Estado existente que o layout lê (sem mudança)

| Item | Onde | Uso na tela |
|---|---|---|
| `setpoint().is_dirty()` | `AppState` | aviso "• não aplicado", cor do botão e aparição do **Desfazer** |
| `is_connected()` | `AppState` | habilita o botão **Aplicar** |
| `apply_setpoints()` / `discard_setpoint_edits()` | `AppState` | ações dos botões (inalteradas) |
