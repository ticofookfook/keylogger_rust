# Capturador de Teclado em Rust

Este projeto é um capturador de teclas (keylogger) simples implementado em Rust. Ele captura os eventos de pressionamento de teclas e grava os caracteres correspondentes em um arquivo de texto.

## Como Funciona

1. **Captura de Eventos**:
   - O código utiliza a biblioteca `winapi` para registrar um hook global no teclado, permitindo que ele capture eventos de teclas pressionadas em todo o sistema.

2. **Thread de Captura**:
   - O programa cria uma thread que escuta os eventos de teclado. Quando uma tecla é pressionada, o código obtém seu código virtual (VK code) e o envia para uma fila usando um canal (`mpsc`).

3. **Thread de Escrita**:
   - Outra thread é responsável por receber os códigos das teclas da fila e convertê-los em caracteres. Os caracteres são escritos em um arquivo chamado `capturar.txt`.

4. **Códigos de Tecla**:
   - O código converte os códigos virtuais das teclas em caracteres legíveis, incluindo letras (A-Z), números (0-9), espaços e a tecla Enter, que é convertida em uma nova linha.

## Dependências

O projeto utiliza as seguintes dependências:

- `winapi`: Para interagir com a API do Windows.
- `lazy_static`: Para permitir a inicialização de variáveis globais de forma segura.

## Como Compilar e Executar

1. Certifique-se de ter o Rust e o Cargo instalados.
2. Clone o repositório ou crie um novo projeto com o código fornecido.
3. Adicione as seguintes dependências ao seu `Cargo.toml`:

   ```toml
   [dependencies]
   winapi = { version = "0.3", features = ["winuser", "libloaderapi"] }
   lazy_static = "1.4"
