# Contrato: estilo do botão de saída

**Feature**: [spec.md](./spec.md) | **Plan**: [plan.md](./plan.md)

## Antes e depois

```text
Antes                                   Depois
┌────────────────────────────┐          ╭────────────────────────────╮
│      SAÍDA DESLIGADA       │          │      SAÍDA DESLIGADA       │
└────────────────────────────┘          ╰────────────────────────────╯
 (encostado na barra inferior)           ↕ 10 px livres
────────────────────────────            ────────────────────────────
 Capacidade  Energia  Tempo              Capacidade  Energia  Tempo
```

## Regras

| Elemento | Regra |
|---|---|
| Raio | 5 px lógicos nos quatro cantos, em todos os estados (desabilitado, desligada, ligada, mouse por cima, pressionado) |
| Margem de baixo | 10 px entre a borda de baixo do botão e o fim da área da coluna de controle (onde começa a barra inferior), em qualquer tamanho de janela |
| Tamanho | 56 px de altura e toda a largura da coluna (como hoje) |
| Texto, cores, dica | inalterados ("SAÍDA LIGADA" verde com texto preto; "SAÍDA DESLIGADA" cinza com texto branco) |
| Comportamento | habilitado só com a fonte conectada; o clique liga ou desliga a saída (`toggle_output`) |

Invariante (Princípio III): nenhum comando é enviado por esta mudança; só o clique do botão continua ligando e
desligando a saída.
