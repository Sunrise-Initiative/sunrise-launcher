# Sunrise Launcher

Versão atual **0.2.0**, seguindo Semantic Versioning. O histórico está no [CHANGELOG](CHANGELOG.md). Commits seguem Conventional Commits e versões recebem tags no formato `v0.2.0`.

Launcher desktop local para explorar os sete aplicativos Sunrise Rise e abrir executáveis vinculados por você. A interface tem tema Moderno e Frutiger Aero, cada um com modo claro e escuro. Catálogo, busca e biblioteca funcionam offline.

## Executar no Windows x64

Abra `target/release/sunrise-launcher.exe` após compilar. No diretório do projeto, com Rust MSVC e Visual Studio Build Tools com suporte a C++:

```powershell
cargo run --locked
cargo build --release --locked
```

O arquivo `rust-toolchain.toml` seleciona o Rust MSVC para este projeto. A janela usa egui, eframe 0.36.2, wgpu e AccessKit.

## Usar

- Em **Explorar**, procure por nome ou categoria, filtre o catálogo ou alterne entre cartões e lista.
- Abra os **Detalhes** e escolha **Vincular executável** para incluir um arquivo local na Biblioteca. Escolha somente um executável de origem confiável.
- Na **Biblioteca**, abra, substitua ou remova o vínculo. Remover um vínculo preserva o arquivo.
- Em **Configurações**, escolha aparência e modo. No Windows, as preferências ficam em `%APPDATA%\Sunrise Launcher Local\settings.json`. Se o arquivo estiver inválido, feche o launcher, faça uma cópia e mova o arquivo para recuperar as preferências padrão.
- No tema **Frutiger Aero**, alterne entre claro e escuro pelo botão no topo da janela. A troca de aparência usa uma transição de 240 ms.
- Ative **Reduzir movimento** em **Configurações** para manter as imagens estáticas e aplicar os realces e a mudança de modo imediatamente.
- **Atualizações** leva ao roadmap do projeto. Os sete produtos estavam marcados como planejados na recriação. A edição local trabalha com vínculos escolhidos manualmente. Distribuição e atualização autenticadas aguardam pacotes oficiais.

## Verificar

```powershell
cargo fmt --check
cargo test --locked
cargo clippy --locked --all-targets --all-features -- -D warnings
```

Código próprio sob GPL-3.0-only. O Frutiger Aero usa a paisagem e as capas do mockup aprovado e ícones Oxygen © KDE sob GNU LGPL 3 ou posterior. As demais capas e a marca vieram do Sunrise Website. As fontes dos recursos e o escopo da observação do ArtCraft estão em [docs/implementation.md](docs/implementation.md).
