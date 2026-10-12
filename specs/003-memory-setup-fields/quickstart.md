# Quickstart: validar os campos de configuração das memórias

**Feature**: [spec.md](./spec.md) | **Plan**: [plan.md](./plan.md)

## Pré-requisitos

- Rust 1.88+, `libudev` de desenvolvimento (`sudo dnf install systemd-devel`).
- Para o roteiro manual: a fonte IPS3608/DPS150 conectada (nenhuma carga ligada à saída).

## 1. Gates automáticos

```bash
cargo fmt --check
cargo clippy --all-targets            # esperado: 0 avisos
cargo test                            # esperado: lib + docs_coverage passam
cargo build --release --bin controller
```

Cobertura esperada dos testes novos (sem hardware):

- Cada campo aceita o máximo exato e rejeita o valor logo acima (SC-007).
- Vset > OVP e Iset > OCP são recusados; valor ausente, negativo ou `NaN` também (SC-004).
- Salvar inválido envia **zero** comandos; salvar válido envia um `SaveMemorySetup` (SC-006).
- Selecionar memória, editar, descartar e "Usar no setpoint" enviam zero comandos.
- Edição pendente sobrevive a atualizações da fonte (SC-005).
- Pacotes de OVP/OCP/OPP/OTP e dos grupos batem com as fixtures de bytes.
- Conferência: leitura igual → "gravado"; três leituras diferentes → "não confirmado".
- `docs_coverage` passa com o documento novo.

## 2. Roteiro manual (com a fonte; Princípio II)

1. Conectar. Conferir que o quadro mostra Vset, Iset, OVP, OCP, OPP e OTP iguais ao visor da fonte (US1).
2. Clicar em M1…M6: os valores mudam, o visor da fonte **não** muda (SC-006).
3. Em M2, mudar Vset para 5,00, Iset para 0,500, OVP para 36,00, OCP para 2,500, OPP para 295,20 e OTP para 60;
   ver "• não salvo"; esperar 5 s (a edição não some).
4. Clicar em **Salvar**: o quadro mostra "verificando…" e depois "gravado". Conferir os seis valores no visor.
5. Digitar OVP 36,20 e Vset 36,50: campos em vermelho, **Salvar** desabilitado. Digitar Vset 12 e OVP 10:
   erro "Vset não pode passar do OVP".
6. **Verificar**: as proteções também mudam por memória? Salvar M3 com OVP diferente da M2, voltar à M2 e
   ver o visor. Registrar o resultado (confirma ou corrige a premissa de R1).
7. Com edição pendente, clicar em outra memória: aparece o aviso de descarte.
8. Card Setpoint: aceita 36,00 V e 8,200 A; **Aplicar** envia (a fonte mostra os valores).
9. Janela inicial de 700 px de altura: o botão de saída e o quadro de setup aparecem inteiros (R8).

## 3. Documentação (FR-015, SC-008)

```bash
cargo test --test docs_coverage
grep -rnE '30 V|5 A\b|de 0 a 30|de 0 a 5' README.md docs/    # esperado: sem ocorrência da faixa antiga
```

Conferir também: README (memórias, proteções, limitações, escrita experimental) e a constituição 1.0.2.

---

## Registro da execução (2026-10-12)

Os gates rodaram numa cópia de trabalho sem `libudev` (o ambiente não a tem); no projeto real, use `cargo` direto.

| Verificação | Resultado |
|---|---|
| Linha de base (T001), antes das mudanças | `cargo fmt --check` ok, `clippy` 0 avisos, 24 testes da biblioteca + 19 de `docs_coverage` |
| `cargo fmt --check` | ok |
| `cargo clippy --all-targets` | 0 avisos |
| `cargo test` | 50 testes da biblioteca + 19 de `docs_coverage`, todos passando |
| `cargo build --release --bin controller` | ok |
| Busca da faixa antiga e de nomes removidos em `README.md`, `docs/`, `.specify/memory/` e `src/` | só sobraram falsos positivos ("0,5 A", "monitoramento") |

**Não verificado sem a fonte e sem tela** (roteiro manual, seção 2, tarefa T037):

- a escrita de OVP/OCP/OPP/OTP e se as proteções são por memória (passos 4 e 6);
- o encaixe do quadro na janela: a altura inicial passou de 540 para 700 px por estimativa, não por medição (passo 9).
