# Keylogger em Rust

## Descrição
Este projeto é um keylogger avançado desenvolvido em Rust que utiliza a API do Windows para capturar e registrar todas as teclas pressionadas pelo usuário. O programa foi criado exclusivamente para fins educacionais e de pesquisa em segurança da informação.

## Características

- **Mapeamento completo de teclas**: Suporte para caracteres especiais, teclas de função e modificadores
- **Rastreamento de janelas**: Registra o título da janela ativa durante a captura de teclas
- **Registro com timestamp**: Adiciona data e hora a cada evento para análise cronológica
- **Sistema de buffer inteligente**: Minimiza operações de I/O de disco para melhor desempenho
- **Suporte a teclas modificadoras**: Interpretação correta de Shift, Ctrl, Alt e Caps Lock
- **Encerramento seguro**: Manipulação adequada do encerramento com suporte para Ctrl+C
- **Tratamento de erros robusto**: Gerenciamento adequado de erros em todo o código
- **Eficiência de memória**: Uso otimizado de recursos do sistema

## Requisitos

- Rust 1.50 ou superior
- Sistema operacional Windows (7/8/10/11)
- Privilégios de administrador (para instalação dos hooks de teclado)

## Dependências

```toml
[dependencies]
winapi = { version = "0.3", features = ["winuser", "minwindef", "libloaderapi", "processthreadsapi"] }
lazy_static = "1.4"
chrono = "0.4"
ctrlc = "3.2"
```

## Instalação

1. Clone este repositório:
```
git clone https://github.com/seu-usuario/keylogger_rust.git
cd keylogger_rust
```

2. Compile o projeto:
```
cargo build --release
```

3. Execute o programa:
```
cargo run --release
```

## Uso

Após a execução, o programa irá:
1. Iniciar a captura de teclas em segundo plano
2. Registrar as teclas pressionadas em um arquivo chamado `keylog.txt`
3. Organizar os logs por aplicativo/janela com carimbos de data/hora

Para encerrar o keylogger, pressione Ctrl+C no terminal onde ele foi iniciado.

## Formato do Log

O arquivo de log será formatado da seguinte maneira:

```
--- Logging started at 2025-03-19 14:30:45 ---

[2025-03-19 14:30:50] Window: Navegador Chrome
google.com[ENTER]

[2025-03-19 14:31:15] Window: Microsoft Word
Este é um texto de exemplo[SHIFT].[ENTER]
```

## Aviso Legal

**IMPORTANTE**: Este software foi desenvolvido EXCLUSIVAMENTE para fins educacionais e de pesquisa em segurança da informação. O uso deste programa para capturar dados de teclado sem o consentimento explícito do usuário é ilegal em muitas jurisdições e antiético em todas as circunstâncias.

O autor não assume responsabilidade pelo uso indevido deste software. Ao utilizar este código, você concorda em:

1. Só executar este software em sistemas em que você tenha permissão explícita
2. Informar todos os usuários que suas entradas de teclado estão sendo registradas
3. Cumprir todas as leis e regulamentos aplicáveis sobre privacidade e segurança de dados

O uso deste software para espionagem, coleta não autorizada de informações, roubo de dados ou qualquer outra atividade maliciosa é expressamente desencorajado e pode resultar em consequências legais severas.

## Licença

Este projeto está licenciado sob a [MIT License](LICENSE).
