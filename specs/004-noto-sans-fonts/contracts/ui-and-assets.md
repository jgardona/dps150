# Contratos: aparência e arquivos

**Feature**: [spec.md](./spec.md) | **Plan**: [plan.md](./plan.md)

## 1. Fonte da interface

| Região | Fonte | Tamanho |
|---|---|---|
| Todos os textos | Noto Sans (peso 400), com as fontes de reserva do egui atrás | o de hoje, **exceto** o quadro de setup |
| Quadro de setup: título "Setup Mn" | Noto Sans | **16 px** (`SETUP_FONT_SIZE`) |
| Quadro de setup: rótulos (Vset, Iset, OVP\*, OCP\*, OPP\*, OTP\*) | Noto Sans | **16 px** |
| Quadro de setup: valores e unidades (`DragValue`) | Noto Sans | **16 px** |
| Quadro de setup: botões, mensagens de erro, aviso de troca, estado da gravação e rodapé | Noto Sans | o de hoje |
| Linha M1–M6 e Salvar, leituras, escalas, legenda, barras | Noto Sans | o de hoje |

O destaque ("strong") continua sendo a cor mais forte do egui; nenhum texto muda de cor ou de peso por causa
desta feature.

## 2. Instalação da fonte

- `gui::install_fonts(ctx: &egui::Context)` instala as definições de `fonts::definitions()` uma vez.
- `controller::main` a chama dentro do construtor da janela, **antes** de criar o `GuiApp` e antes do
  primeiro quadro, para nenhum texto ser desenhado com a fonte antiga.

## 3. Arquivos

- `assets/fonts/NotoSans-VariableFont_wght.ttf` e `assets/fonts/OFL.txt` ficam versionados no repositório.
- O arquivo da fonte é incluído no executável em tempo de compilação. O programa **não** lê a fonte do
  disco nem do sistema em tempo de execução.
- O README informa: nome da fonte, versão, origem (`notofonts/latin-greek-cyrillic`) e licença (SIL OFL 1.1),
  com o caminho do texto da licença.
