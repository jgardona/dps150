# Contratos: interface e comando serial

**Feature**: [spec.md](./spec.md) | **Plan**: [plan.md](./plan.md)

## 1. Interface (coluna de controle, de cima para baixo)

```text
[ leituras V / A / W ]
[ card Setpoint: V [..] A [..] · Aplicar · Desfazer ]
[ Memória  M1 M2 M3 M4 M5 M6  Salvar ]
┌ quadro de setup ──────────────────────────────┐
│ Setup M2                      • não salvo     │
│  Vset      Iset      OVP*                     │
│  05.00 V   0.500 A   36.00 V                  │
│  OCP*      OPP*      OTP*                     │
│  2.500 A   295.20 W  60 °C                    │
│  [Usar no setpoint] [Desfazer]   gravado ✓    │
│  * vale para o aparelho todo                  │
└───────────────────────────────────────────────┘
[ SAÍDA LIGADA / DESLIGADA ]
```

| Elemento | Regra |
|---|---|
| Título | "Setup Mn", n = memória selecionada; "• não salvo" (cor de aviso) quando há pendência |
| Campos | rótulo na cor da grandeza para V, A, W; os demais em cinza; valor com as casas da tabela; sem dado = "—" desabilitado |
| Campo inválido | borda/texto vermelho e a mensagem do erro embaixo; **Salvar** desabilitado com a dica do primeiro erro |
| **Salvar** (na linha das memórias) | habilitado só conectado, com seis valores válidos; grava no quadro da memória selecionada |
| **Desfazer** | só com pendência; volta aos valores da fonte |
| **Usar no setpoint** | carrega Vset/Iset da memória no card Setpoint (marca "não aplicado"); não envia nada |
| Aviso de troca | com pendência, clicar em outra memória mostra "Descartar as alterações de Mn?" com **Descartar** / **Continuar editando**; nada muda até escolher |
| Estado da gravação | "verificando…" (cinza), "gravado" (verde), "não confirmado" (vermelho, com dica) |
| Sem conexão | campos e botões desabilitados |
| Rodapé | "* vale para o aparelho todo" (texto fraco) |

Invariantes (Princípio III): selecionar, editar, descartar e "Usar no setpoint" **nunca** enviam
comandos; só **Salvar**, **Aplicar** e o botão de saída enviam.

## 2. Comando da GUI para a sessão

```text
AppCommand::SaveMemorySetup(SetupValues)   // group 1..6, vset, iset, ovp, ocp, opp, otp
```

Substitui `AppCommand::SaveProfile(n, v, c)`.

## 3. Pacotes enviados pela sessão (nesta ordem, 100 ms entre eles)

Todos `HEADER_OUTPUT (0xf1)`, `CMD_SET (0xb1)`, payload de 4 bytes `f32` little-endian:

| Ordem | type_id | Valor |
|---|---|---|
| 1 | 209 (`OVP`) | ovp |
| 2 | 210 (`OCP`) | ocp |
| 3 | 211 (`OPP`) | opp |
| 4 | 212 (`OTP`) | otp |
| 5 | 197 + 2·(n−1) | vset do grupo n |
| 6 | 198 + 2·(n−1) | iset do grupo n |
| 7 | 255, `CMD_GET` (`get_all`) | — (releitura) |

Grupo fora de 1..6: nenhum pacote é enviado (e `validate` já impede essa entrada).
Escrita das proteções: **experimental** (não confirmada em aparelho real).

## 4. Máximos (FR-013)

Fonte única: constantes em `logics::memory_setup`, usadas pela validação e pelos campos da GUI,
inclusive os campos V e A do card Setpoint (FR-014).
