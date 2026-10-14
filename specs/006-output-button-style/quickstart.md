# Quickstart: validar o estilo do botão de saída

**Feature**: [spec.md](./spec.md) | **Plan**: [plan.md](./plan.md)

## Pré-requisitos

- Rust 1.88+ e `libudev` de desenvolvimento.
- Para o roteiro visual: ambiente gráfico (a fonte conectada só é necessária para o passo 3).

## 1. Gates automáticos

```bash
cargo fmt --check
cargo clippy --all-targets            # esperado: 0 avisos
cargo test                            # esperado: lib + docs_coverage passam
cargo build --release --bin controller
```

Testes novos esperados (sem janela): o retângulo desenhado do botão (desabilitado) tem `corner_radius` de 5; e a
folga entre o fundo do botão e o fundo da área, em layout de baixo para cima, é de 10 px (± 0,5).

## 2. Roteiro visual (US1 e US2)

1. Abrir o programa sem a fonte: o botão cinza, desabilitado, tem os quatro cantos arredondados (SC-001).
2. Há um espaço livre de 10 px entre o botão e a barra inferior (a que mostra Capacidade, Energia e Tempo)
   (SC-002). Conferir também com a janela maximizada e reduzida.
3. Com a fonte conectada: ligar a saída (botão verde) e desligar; o raio é o mesmo nos dois estados, o botão não
   muda de tamanho nem de posição ao trocar (SC-001, SC-003).
4. Passar o mouse e clicar sem soltar: contorno, brilho e preenchimento seguem o raio; nenhum canto reto aparece
   por baixo.
5. O texto continua centralizado e inteiro; o botão continua com 56 px de altura e a largura da coluna.
6. O botão, ao subir 10 px, não encosta no quadro de setup (janela inicial de 700 px).

## 3. Comportamento (FR-005, com a fonte conectada)

1. Clicar ligando e desligando a saída: o efeito é o de antes.
2. Sem conexão, o botão está desabilitado.
3. A dica continua "Clique para ligar a saída" / "Clique para desligar a saída".

## 4. Documentação (FR-007)

```bash
cargo test --test docs_coverage
grep -n "OUTPUT_BUTTON_RADIUS\|OUTPUT_BOTTOM_MARGIN" docs/modulos/gui-mod.md README.md
```

---

## Registro da execução (2026-10-14)

| Verificação | Resultado |
|---|---|
| Linha de base (T001), antes das mudanças | `fmt` ok, `clippy` 0 avisos, 64 testes da biblioteca + 19 de `docs_coverage` |
| `cargo fmt --check` | ok |
| `cargo clippy --all-targets` | 0 avisos |
| `cargo test` | 66 testes da biblioteca + 19 de `docs_coverage`, todos passando |
| `cargo build --release --bin controller` | ok |
| Varredura de "encostado" em `README.md` e `docs/` | só resta o do selo CV/CC em `gui-widgets.md`, que não tem relação com o botão |

**Ordem dos testes (vermelho antes de verde):** o teste do raio falhou com o raio padrão do egui (`2`) e passou com
`corner_radius(5)`. O teste da folga falhou com `0` (o botão encostava na base da área) e, depois, com `7` na
primeira tentativa de margem.

**Correção em relação ao plano:** o plano previa `add_space(10 - item_spacing.y)`, achando que o egui somaria o
espaçamento entre itens depois do espaço. O teste mostrou que não soma: com o desconto a folga era 7 px. O código
usa `add_space(OUTPUT_BOTTOM_MARGIN)` e a folga medida é 10 px. `research.md` (R2), `plan.md` e `tasks.md` foram
corrigidos.

**Conferência do comportamento (T012):** por `git diff`, no botão mudaram só o raio (`corner_radius`), a margem
(`add_space`) e o retorno (`egui::Response`). Texto, cores, dica, `add_enabled_ui`, `add_sized([w, 56.0], …)` e
`toggle_output` são idênticos; o teste mede 56 px de altura e a largura da área.

**Não verificado neste ambiente** (precisa de tela; tarefa T013):

- os estados "ligada" (verde), mouse por cima e pressionado (o teste só roda o estado desabilitado; o raio é uma
  única chamada, antes de qualquer estado);
- a folga **visual** contra a barra inferior: o teste mede até o fim da área da coluna, e o painel inferior pode ter
  margem interna própria acima do texto;
- que o botão, ao subir 10 px, não encosta no quadro de setup na janela inicial.
