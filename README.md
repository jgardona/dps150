# DPS150 Protocolo de Comunicação

Havia comprado a fonte IPS3608, da FNIRSI, pois havia visto em um review e me interessou. É um ótima fonte, que conta uma saída de 36V e 8A.
Como sou usuário de sistemas baseados no linux, não pude utilizar o aplicativo que acompanha esse dispositivo, pois ele é um softaware for win. Isso me motivou a procurar pelo protocolo de comunicação usado. Descobri que é uma variante do mestre/escravo como o **Modbus**, mas simplificado, usado para controlar fontes de bancada digitais(como a série RD ou DPS da RuiDeng/Hangzhou Ruideng).

A linguagem utilizada é **Rust**. Cheguei a escrever o mesmo usando c++23, mas o código ficou melhor e menor usando a primeira linguagem citada.

``Esse é um projeto experimental.``