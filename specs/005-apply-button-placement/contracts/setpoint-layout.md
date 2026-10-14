# Contrato: layout do card Setpoint

**Feature**: [spec.md](./spec.md) | **Plan**: [plan.md](./plan.md)

## Antes

```text
Setpoint  • não aplicado
V [05.00 V]   A [0.500 A]
[Aplicar] [Desfazer]
─────────────────────────
Memória  M1 M2 M3 M4 M5 M6  Salvar
```

## Depois

```text
Setpoint  • não aplicado  [Desfazer]     ← aviso e Desfazer só com alteração pendente
V [05.00 V]   A [0.500 A]   [Aplicar]
─────────────────────────
Memória  M1 M2 M3 M4 M5 M6  Salvar
```

## Regras

| Elemento | Regra |
|---|---|
| Linha de campos | `V`, campo V, `FIELD_GAP`, `A`, campo A, `FIELD_GAP`, botão **Aplicar**; alinhada à esquerda; itens centralizados na vertical |
| Espaço A → Aplicar | o mesmo construído entre V e A (8 px + `FIELD_GAP` + 8 px) |
| **Aplicar** | mesmo texto, tamanho natural e dica de hoje; azul com pendência, cinza sem; desabilitado sem conexão; clique ou **Enter** num campo envia tensão e corrente |
| **Desfazer** | só com pendência; na linha do título, logo depois de "• não aplicado"; clique volta aos valores da fonte; aparecer e sumir não muda a altura do card |
| Altura do card | a mesma com e sem pendência; nenhuma linha de botões entre os campos e o separador |
| Largura | a linha cabe nos 314 px úteis da coluna mínima (330 px menos as margens do card), com 36,00 V e 8,200 A nos campos |

Invariante (Princípio III): nenhum comando é enviado por esta mudança; só **Aplicar** e **Enter** continuam
enviando.
