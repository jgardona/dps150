# Quickstart: validar a separação em `logics` e `gui`

**Feature**: [spec.md](./spec.md) | **Plan**: [plan.md](./plan.md)

Este guia comprova, de ponta a ponta, que a refatoração cumpre a spec. Ele não descreve a
implementação; para isso, veja [contracts/logics-api.md](./contracts/logics-api.md) e o futuro
`tasks.md`.

## Pré-requisitos

- Rust 1.88 ou mais novo, `libudev` de desenvolvimento (`sudo dnf install systemd-devel`), usuário
  no grupo `dialout`.
- Fonte IPS3608 ou DPS150 conectada por USB, para os passos manuais.
- **Antes de começar a refatoração**, registre a referência de avisos:

  ```bash
  cargo clippy --all-targets 2>&1 | grep -c '^warning'   # referência registrada: controller 18, tests 31
  ```

## 1. Gates automáticos (sem hardware)

```bash
cargo fmt --check
cargo clippy --all-targets          # esperado: nenhum aviso novo em relação à referência
cargo test --lib                    # esperado: todos os testes de logics passam
cargo build --release --bin controller --bin tests
```

`cargo test --lib` precisa passar numa máquina **sem interface gráfica e sem porta serial**
(SC-002). Para conferir isso, rode dentro de um container ou de uma sessão SSH sem `DISPLAY` nem
`WAYLAND_DISPLAY`.

## 2. Verificações estruturais

```bash
# SC-003 — logics não referencia a biblioteca de interface
grep -rnE 'egui|eframe' src/logics/ && echo "FALHOU" || echo "ok"

# FR-005 — gui não acessa a serial nem o protocolo diretamente
grep -rnE 'serialport|logics::protocol::DPS150' src/gui/ && echo "FALHOU" || echo "ok"

# SC-004 — controller.rs só faz a inicialização
wc -l src/controller.rs              # esperado: ~40 linhas ou menos
grep -nE 'fn main' src/controller.rs # esperado: 1 ocorrência

# FR-009 — o monitor usa o protocolo da biblioteca
grep -n 'mod dps150' src/tests.rs && echo "FALHOU (ainda duplica o protocolo)" || echo "ok"
```

## 3. Paridade de comportamento (com hardware)

```bash
cargo run --release --bin controller
```

Execute o roteiro **"Verificação manual de paridade"** da [spec](./spec.md#verificação-manual-de-paridade)
(9 passos). Todos precisam dar o mesmo resultado de antes da refatoração (SC-001). Um ponto de
atenção é uma diferença pequena e esperada, explicada na pesquisa (R7): uma porta recém-plugada pode
levar até 1 s para aparecer na lista.

Para comparar com o estado anterior, compile o commit base num worktree separado:

```bash
git worktree add ../dps150-base master
(cd ../dps150-base && cargo run --release --bin controller)
```

## 4. Monitor de linha de comando

```bash
cargo run --bin tests
```

Esperado: a mesma saída de antes no terminal (modelo, tensão, corrente, temperatura, versões,
setpoints, energia).

## 5. Fechamento

- [ ] README, seção "Estrutura do código", atualizada (FR-012).
- [ ] Emenda PATCH da constituição trocando `src/dps150.rs` por `src/logics/protocol.rs` (R2).
