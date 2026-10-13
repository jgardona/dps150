# Quickstart: validar a fonte Noto Sans e o quadro de 16 px

**Feature**: [spec.md](./spec.md) | **Plan**: [plan.md](./plan.md)

## Pré-requisitos

- Rust 1.88+ e `libudev` de desenvolvimento (`sudo dnf install systemd-devel`).
- Para o roteiro visual: ambiente gráfico (a fonte conectada ajuda, mas não é necessária).

## 1. Gates automáticos

```bash
cargo fmt --check
cargo clippy --all-targets            # esperado: 0 avisos
cargo test                            # esperado: lib + docs_coverage passam
cargo build --release --bin controller
ls -l target/release/controller       # registrar o tamanho (antes e depois da feature)
```

Testes novos esperados (sem janela): `noto-sans` é a primeira fonte de `Proportional`; as fontes de reserva
continuam; `Monospace` não muda; os bytes embutidos têm a assinatura TrueType e mais de 100 KB.

## 2. Compilar sem a fonte no sistema (US3, SC-005)

1. `fc-list | grep -i "Noto Sans"` mostra se a fonte está instalada. Para simular uma máquina sem ela,
   rode o programa com um ambiente de fontes vazio:

   ```bash
   FONTCONFIG_FILE=/dev/null ./target/release/controller
   ```

2. A interface continua em Noto Sans (o `G` de "Aguardando", os algarismos arredondados e o `1` com a
   bandeira são os sinais da Noto Sans).
3. Copiar só o binário `release` para outra pasta e executar: mesma aparência (US3-4).
4. `ls assets/fonts` mostra a fonte e o `OFL.txt`; o README cita origem e licença (US3-3).

## 3. Roteiro visual (US1 e US2)

1. Abrir o programa (sem a fonte conectada) e olhar cada região: leituras, setpoint, memórias, gráfico,
   barras e a barra superior. Todas em Noto Sans (SC-001).
2. Provocar mensagens fixas com acentos: "não aplicado", "SAÍDA DESLIGADA", "Descartar as alterações…",
   "verificando…", "não confirmado", e o erro "Nenhum dispositivo/porta serial disponível para conexão."
   (Conectar sem porta). Nenhum quadradinho (SC-002).
3. Conectar a uma fonte (ou simular) e digitar nos seis campos do setup os máximos: 36,00 V, 8,200 A,
   36,10 V, 8,200 A, 295,20 W e 99 °C. Todos inteiros, sem corte nem sobreposição (SC-004).
4. Estreitar a janela até a largura mínima da coluna de controle (330 px): os campos continuam inteiros.
5. Conferir que o quadro de setup tem texto de 16 px e o botão de saída e a barra inferior seguem visíveis
   na janela inicial (700 px de altura). Se não, aumentar `HEIGHT` em `src/controller.rs` e documentar.
6. Legibilidade (SC-003): a ≈ 50 cm da tela, ler os seis rótulos e valores sem se aproximar.
7. Escalas do gráfico: os números das três colunas (V, A, W) não se encostam nem passam da coluna.
8. Um campo com erro (ex.: Vset 36,50) continua legível em vermelho.

## 4. Tempo de abertura (SC-006)

```bash
time ./target/release/controller   # fechar a janela assim que aparecer; comparar com a versão anterior
```

Registrar os dois tempos. O aumento esperado é de poucos milissegundos.

## 5. Documentação (FR-012)

```bash
cargo test --test docs_coverage
grep -rn "gui-fonts" docs/ README.md        # o módulo novo está listado
```

Conferir: README (assets, fonte, licença, tamanho do setup), `docs/modulos/gui-fonts.md` e os documentos
alterados.

---

## Registro da execução (2026-10-13)

Nesta máquina o `libudev` está instalado, então os gates rodaram direto no projeto.

| Verificação | Resultado |
|---|---|
| Linha de base (T001), antes das mudanças | `clippy` 0 avisos; 50 testes da biblioteca + 19 de `docs_coverage`; binário `release` de **10 987 928 bytes** |
| `cargo fmt --check` | ok |
| `cargo clippy --all-targets` | 0 avisos |
| `cargo test` | 57 testes da biblioteca + 19 de `docs_coverage`, todos passando |
| `cargo build --release --bin controller` | ok; binário de **11 702 296 bytes** (+714 368 bytes, ≈ 0,68 MB) |
| A fonte inteira está dentro do binário | sim (busca dos 712 460 bytes do `.ttf` no executável: encontrados) |
| Varredura de texto da fonte antiga | só restam "Ubuntu-Light" como fonte de reserva (correto) e "Debian/Ubuntu" no README (sistema operacional) |

Testes automáticos novos que sustentam o roteiro visual:

- a Noto Sans é de fato usada (o mesmo texto mede largura diferente com e sem ela), o que também prova que o egui
  consegue ler o arquivo variável embutido;
- os seis valores máximos (36,00 V, 8,200 A, 36,10 V, 8,200 A, 295,20 W e 99 °C), em 16 px, cabem na largura
  mínima da coluna de controle (330 px), contando preenchimento, espaços e margens.

**Não verificado neste ambiente** (precisa de olhos; seção 3 do roteiro, tarefas T018, T021 e T022):

- nenhum quadradinho nos textos com acento e símbolos, e o aspecto geral da interface em Noto Sans;
- o encaixe vertical (botão de saída e barra inferior visíveis na janela de 700 px);
- as escalas do gráfico com a fonte nova (colunas de 40 px, texto de 9 e 10 px);
- a execução sem a Noto Sans no sistema (`FONTCONFIG_FILE=/dev/null`) e do binário copiado sozinho;
- o tempo de abertura da janela.

Uma tentativa de abrir a janela e capturar a tela falhou (a captura não funcionou nesta sessão), então nada visual foi
conferido por mim.
