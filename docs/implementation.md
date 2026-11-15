# Recriação do Sunrise Launcher 0.1.0

O vault registra uma implementação anterior. A pasta indicada no histórico estava ausente do computador e a organização no GitHub não tinha um repositório do launcher. O usuário escolheu recriar a edição local neste workspace, com a interface das duas prévias Sunrise. A recriação usa configuração própria para preservar qualquer edição anterior encontrada depois.

## Comportamento e origem

- O ArtCraft Launcher 0.19.2 instalado foi observado no Windows. Foram confirmados cartões, tabela, detalhes, busca disponível na barra, estados de instalação, ações por produto e metadados de release. A análise REA confirmou o PE x64, recursos e importações nativas. A seção de dependências cargo-auditable não estava presente. A instalação e a atualização do alvo ficaram fora do ensaio.
- O site e o vault Sunrise fornecem o catálogo, o conteúdo e a identidade visual. As capas utilizadas vieram dos recursos do próprio site. A implementação segue a política Sunrise de reimplementação independente por observação, com separação limitada de contexto. A pesquisa dispensou código, pseudocódigo e recursos gráficos do ArtCraft.
- Catálogo, busca e filtros funcionam offline. Os sete produtos exibem o estado Planejado. Detalhes oferecem descrição, escopo, página pública e vínculo opcional de um executável escolhido pela pessoa.
- Biblioteca distingue vínculo disponível de caminho indisponível. Abrir exige ação explícita e executa o arquivo diretamente. Remover vínculo preserva o arquivo.
- Preferências persistem tema Moderno ou Aero, claro ou escuro, e visualização em cartões ou lista. Atualizações informa a situação dos releases. Downloads autenticados, instalação e atualização dependem de infraestrutura e pacotes futuros.

## Implementação e aceite

- Catálogo estático com busca por nome, categoria e descrição, incluindo buscas sem acentos. Conferir os sete produtos e a mensagem para resultado vazio.
- Configuração JSON versionada por usuário. Conferir persistência, preservação de JSON inválido, conflito entre instâncias e recusa de caminhos relativos ou de rede.
- UI nativa com egui, eframe, wgpu e AccessKit. Conferir janela redimensionável, teclado, navegação, cartões/lista e as quatro combinações de aparência.
- Verificar com rustfmt, testes, Clippy e build de release. Abrir a janela e testar as funções principais no Windows.

## Revisão Frutiger Aero

Os sete ícones de aplicativos em `assets/aero-icons/` foram gerados para o Sunrise por solicitação do usuário. Os PNGs originais são preservados e as cópias de 192 pixels são incorporadas ao launcher. Os símbolos nos botões são glifos da fonte, combinados com as superfícies de vidro existentes. As páginas de detalhes compartilham o layout responsivo e os mesmos comandos de vínculo local da Biblioteca.

A galeria [Frutiger Aero Archive — Wallpapers](https://frutigeraeroarchive.org/wallpapers) orientou a composição com paisagens, vidro colorido e reflexos. O modo claro combina o parque e o lago do Sunrise Website. O modo escuro usa a fotografia de orvalho do mesmo site. As imagens são incorporadas ao executável para exibição offline.

O destaque do RisePhoto passa para duas faixas nas larguras compactas. A busca fica alinhada à direita, a navegação preserva os rótulos em uma linha e os botões dos cartões mantêm o alinhamento. A verificação cobre contraste dos textos e controles sobre os painéis e geometria do catálogo em três larguras.

## Mockup aprovado do Aero claro

O lago e as sete ilustrações de produtos em `assets/aero-light/` são recortes da imagem aprovada, com SHA-256 `5373edd898588e8eb50cc481620493ffbb0fdec87e123ec3320d23a73fa2b7b2`. A janela, o texto e os controles são desenhados pelo egui. O tema usa as fontes Segoe UI disponíveis no Windows, com a tipografia padrão do egui como alternativa.

Os quatro ícones da navegação são [Oxygen © KDE](https://github.com/KDE/oxygen-icons/tree/940e7e98cabe640f093b925837a0cd8189c5cf79/32x32), sob GNU LGPL 3 ou posterior. Os termos acompanham os arquivos em `assets/aero-light/oxygen/`. O globo veio do Sunrise Website. O ícone de pasta é o `mimetypes/inode-directory.png` da mesma revisão.
