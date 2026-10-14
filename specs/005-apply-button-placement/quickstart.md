# Quickstart: validar o botão Aplicar na linha dos campos

**Feature**: [spec.md](./spec.md) | **Plan**: [plan.md](./plan.md)

## Pré-requisitos

- Rust 1.88+ e `libudev` de desenvolvimento.
- Para o roteiro visual: ambiente gráfico (a fonte conectada só é necessária para o passo 4).

## 1. Gates automáticos

```bash
cargo fmt --check
cargo clippy --all-targets            # esperado: 0 avisos
cargo test                            # esperado: lib + docs_coverage passam
cargo build --release --bin controller
```

Testes novos esperados (sem janela), medidos em Noto Sans de 14 px: a linha V + A + **Aplicar**, com os valores
máximos (36,00 V e 8,200 A), e a linha do título com "Setpoint", "• não aplicado" e **Desfazer** cabem, cada
uma, nos 314 px úteis da coluna mínima.

## 2. Roteiro visual (US1 e US3)

1. Abrir o programa. No card Setpoint: V, A e **Aplicar** na mesma linha, nessa ordem; nenhuma linha de botões
   abaixo (SC-001, SC-003).
2. O espaço entre o campo A e o botão parece igual ao espaço entre o campo V e o "A" (SC-002).
3. Os três têm a mesma altura e o texto na mesma linha de base.
4. Estreitar a janela até a coluna mínima (330 px) com 36,00 V e 8,200 A digitados: nada quebra nem corta.
5. Digitar um valor: na linha do título aparecem "• não aplicado" e o **Desfazer**, e o botão **Aplicar** fica
   azul; **nada se move** (a linha dos campos, a linha de memórias e o quadro de setup ficam onde estavam).
6. O **Desfazer** começa na mesma posição horizontal do **Aplicar** (a borda esquerda dos dois botões fica na mesma vertical), inclusive ao digitar valores de larguras diferentes (ex.: 1,00 V e 36,00 V).
7. Os botões **Aplicar** e **Desfazer** têm a mesma largura, com o texto centralizado em cada um.
8. **Desfazer**: os campos voltam aos valores da fonte, o aviso e o botão somem, e nada se move de novo (SC-003).
9. Sem conexão: o botão **Aplicar** aparece desabilitado.

## 3. Comportamento do botão (US2, com a fonte conectada)

1. Editar V e A e clicar em **Aplicar**: a fonte passa a ter os valores; "• não aplicado" some.
2. Editar V e apertar **Enter** no campo: mesmo efeito.
3. Passar o mouse sobre o botão: dica "Envia Vset/Iset ao aparelho (ou Enter)".
4. Sem alteração pendente, o botão está cinza e continua habilitado (conectado).

## 4. Documentação (FR-008)

```bash
cargo test --test docs_coverage
grep -n "Aplicar" README.md docs/modulos/gui-panels.md      # a posição nova está descrita
```

---

## Registro da execução (2026-10-13)

| Verificação | Resultado |
|---|---|
| Linha de base (T001), antes das mudanças | `fmt` ok, `clippy` 0 avisos, 57 testes da biblioteca + 19 de `docs_coverage` |
| `cargo fmt --check` | ok |
| `cargo clippy --all-targets` | 0 avisos |
| `cargo test` | 60 testes da biblioteca + 19 de `docs_coverage`, todos passando |
| `cargo build --release --bin controller` | ok |
| Varredura da posição antiga dos botões em `README.md` e `docs/` | só resta a frase que descreve a posição nova |

**Conferência do comportamento do botão (T008):** por `git diff`, na parte do **Aplicar** mudaram só a posição (agora
na linha dos campos, depois de `add_space(FIELD_GAP)`) e a remoção da `ui.horizontal` antiga. Texto, cores
(`ACTION_COLOR` com pendência, cinza sem), `add_enabled(app.is_connected(), …)`, dica e a lógica
`clicked || (enter_pressed && app.is_connected())` são idênticos. Os testes `aplicar_envia_os_valores_dos_campos` e
`aplicar_envia_os_maximos_do_setpoint` passam sem alteração.

**Medidas dos testes sem janela** (Noto Sans, corpo padrão, estilo padrão do egui):

| Linha | Largura | Limite (coluna mínima) |
|---|---|---|
| V + A + **Aplicar**, com 36,00 V e 8,200 A | ≈ 255 px | 330 px |
| Título com "• não aplicado" e **Desfazer** | ≈ 231 px | 330 px |

**Achado durante a implementação:** a linha do título só com o texto era ≈ 0,3 px mais baixa que a com o botão
**Desfazer** (22,72 px contra 23 px), o que faria o card dar um pequeno salto ao editar. O teste de altura pegou
isso e a linha ganhou a altura mínima de um botão (arredondada para cima); agora as duas alturas são iguais.

**Não verificado neste ambiente** (precisa de tela e da fonte; tarefas T009 e T016):

- o alinhamento e o aspecto da linha V, A, **Aplicar** (mesma altura, texto na mesma linha de base) e os espaços
  iguais, que a construção garante mas ninguém viu;
- o comportamento com a fonte conectada: clique, **Enter**, dica, cores e botão desabilitado sem conexão;
- que nada se move na tela ao aparecer e sumir a pendência (o teste cobre a altura da linha do título, não o
  resto da coluna).
