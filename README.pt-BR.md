**Idiomas:** [English](README.md) · **Português (Brasil)**

# ClipHistory

<!-- Substitua %user% pelo seu usuário/proprietário do GitHub. -->

[![Testes](https://img.shields.io/github/actions/workflow/status/%user%/ClipboardHistory/ci.yml?branch=main&label=testes&logo=github)](https://github.com/%user%/ClipboardHistory/actions/workflows/ci.yml)
[![Release](https://img.shields.io/github/actions/workflow/status/%user%/ClipboardHistory/release.yml?label=release&logo=github)](https://github.com/%user%/ClipboardHistory/actions/workflows/release.yml)
[![Última versão](https://img.shields.io/github/v/release/%user%/ClipboardHistory?display_name=tag&sort=semver&label=%C3%BAltima%20vers%C3%A3o&logo=github)](https://github.com/%user%/ClipboardHistory/releases/latest)
[![Licença: MIT](https://img.shields.io/badge/licen%C3%A7a-MIT-000000.svg)](LICENSE)

Histórico da área de transferência para **Windows, Linux e macOS**, feito com
[Tauri 2](https://tauri.app) + React + TypeScript.

Copie qualquer coisa em qualquer aplicativo e recupere depois com um atalho global: um popup
mostra os últimos itens, com busca, itens fixados e suporte a imagens. Tudo fica salvo
localmente em SQLite — nada sai do seu computador.

## Funcionalidades

- **Captura automática** de textos e imagens copiados.
- **Auto paste**: ao escolher um item, simula `Ctrl+V` / `⌘V` no aplicativo anterior
  (ligado por padrão, configurável).
- **Atalho global configurável** (padrão `Ctrl+Shift+V` / `⌘ ⇧ V`), funcionando com o app em
  segundo plano.
- **Popup rápido** com busca instantânea, navegação por teclado e cópia com `Enter`.
- **Itens fixados** que não são removidos ao limpar o histórico nem ao atingir o limite.
- **Limite configurável** de itens não fixados (10 a 500).
- **Iniciar com o sistema** (login/autostart) configurável.
- **Ícone na bandeja** com acesso ao histórico, às configurações e à opção de sair.
- **Instância única**: abrir o app de novo apenas mostra o popup.
- Interface em inglês (padrão) e português; tema claro/escuro seguindo o sistema e janela sem
  moldura.

## Download

Os instaladores prontos da versão mais recente ficam na página de
[Releases](https://github.com/%user%/ClipboardHistory/releases/latest):

- **Windows:** `ClipHistory_*_x64_en-US.msi` (ou `ClipHistory_*_x64-setup.exe`) — execute o
  instalador. Se o SmartScreen aparecer, escolha **Mais informações → Executar assim mesmo**.
- **macOS:** `ClipHistory_*_universal.dmg` — abra o DMG e arraste o ClipHistory para Aplicativos.
  O build tem assinatura ad-hoc, mas não é notarizado; se o macOS bloquear, clique com o botão
  direito → **Abrir** ou execute
  `xattr -dr com.apple.quarantine /Applications/ClipHistory.app`.
- **Linux:** `ClipHistory_*_amd64.AppImage` (`chmod +x` e execute), `ClipHistory_*_amd64.deb`
  (`sudo apt install ./ClipHistory_*_amd64.deb`) ou `ClipHistory-*.rpm`
  (`sudo dnf install ./ClipHistory-*.rpm`). O AppImage usa o WebKitGTK 4.1 do sistema; se não
  abrir, instale `libwebkit2gtk-4.1-0` (Debian/Ubuntu) ou `webkit2gtk4.1` (Fedora).

> No Wayland, atalhos globais e auto paste dependem do compositor. Veja as
> [observações sobre Wayland](#o-atalho-global-não-funciona-no-wayland).

## Stack

| Camada          | Tecnologia                                                                 |
| --------------- | -------------------------------------------------------------------------- |
| Shell desktop   | Tauri 2 (Rust)                                                             |
| Interface       | React 19 + TypeScript + Vite                                               |
| Banco de dados  | SQLite via `rusqlite` (bundled), modo WAL                                  |
| Clipboard       | `tauri-plugin-clipboard-manager` (leitura por polling em thread dedicada)  |
| Atalho global   | `tauri-plugin-global-shortcut`                                             |
| Auto paste      | `enigo` (simulação de `Ctrl+V` / `⌘V`)                                     |
| Autostart       | `tauri-plugin-autostart`                                                   |
| Instância única | `tauri-plugin-single-instance`                                             |
| Imagens         | `image` (PNG)                                                              |

## Como funciona

- Uma **thread em Rust** verifica a área de transferência a cada **700 ms** (o plugin é
  bloqueante e não deve rodar na thread da interface). Texto tem prioridade; se não houver
  texto, o conteúdo é lido como imagem RGBA.
- Cada item é identificado por **SHA-256**. Copiar o mesmo conteúdo de novo não cria duplicata:
  o item existente é movido para o topo.
- **Imagens** são gravadas como PNG no diretório de dados do app; a lista mostra apenas
  metadados leves, sem carregar thumbnails.
- O popup e a janela de configurações são janelas Tauri separadas. O atalho global alterna o
  popup; perder o foco esconde (configurável), e `Esc` também fecha.
- Com **auto paste** ligado, ao escolher um item o popup se esconde, o foco volta ao aplicativo
  anterior e a combinação de colar é simulada cerca de 250 ms depois. No macOS o app inteiro é
  ocultado para devolver o foco.
- Configurações ficam em `settings.json` (diretório de configuração do app) e são aplicadas
  imediatamente ao salvar — inclusive o re-registro do atalho e o autostart.

## Pré-requisitos

- **Node.js 20.19+** e npm.
- **Rust 1.90+** (`rustup` recomendado).
- Dependências de sistema do Tauri para o seu SO.

### Linux (Debian/Ubuntu)

```bash
sudo apt update
sudo apt install libwebkit2gtk-4.1-dev build-essential curl wget file \
  libxdo-dev libssl-dev libayatana-appindicator3-dev librsvg2-dev libxkbcommon-dev libfuse2
```

Fedora:

```bash
sudo dnf install webkit2gtk4.1-devel openssl-devel curl wget file \
  libappindicator-gtk3-devel librsvg2-devel libxkbcommon-devel
sudo dnf group install "C Development Tools and Libraries"
```

Arch:

```bash
sudo pacman -S webkit2gtk-4.1 base-devel curl wget file openssl \
  appmenu-gtk-module libappindicator-gtk3 librsvg libxkbcommon
```

### macOS

```bash
xcode-select --install
```

### Windows

- [Microsoft C++ Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/)
- WebView2 (já incluído no Windows 11; instalável no Windows 10)

Instale o Rust, caso ainda não tenha:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

## Rodando

```bash
npm install
npm run tauri dev
```

Build de produção (gera instaladores em `src-tauri/target/release/bundle/`):

```bash
npm run tauri build
```

Apenas validar a interface (TypeScript + Vite):

```bash
npm run build
```

### Pré-visualização da interface sem Rust

Durante o desenvolvimento, `npm run dev` também abre a interface em um navegador comum usando
dados de exemplo (o backend Rust não é iniciado). O popup aparece por padrão; para ver a tela
de configurações:

```text
http://localhost:1420/?window=settings
```

Esse modo existe apenas em desenvolvimento (`import.meta.env.DEV`) e não entra no app final.

### CI

O workflow em `.github/workflows/ci.yml` roda os testes/type-check/build do frontend e o
`cargo check`/`cargo test` do backend em Windows, Linux e macOS a cada push/PR — útil para
validar o Rust sem precisar das dependências de sistema na máquina local.

### Testes

```bash
npm test                                          # frontend (Vitest)
cargo test --manifest-path src-tauri/Cargo.toml   # backend (Rust)
```

A suíte cobre os dicionários de tradução (paridade de chaves/placeholders), formatação de datas,
parsing/formatação de atalhos, a política de retentativa de foco, decisões de blur/foco, o
roteamento de `--toggle`, a configuração da janela do popup (`focus`, `center`, `visible`) e a
configuração de release/versão. O CI também compila o projeto, o que evita regressões como o
erro de tempo de vida do closure no `setup`.

### Publicando uma release

O workflow `.github/workflows/release.yml` compila e publica os instaladores. Ele roda quando
uma tag começando com `v` é enviada, ou manualmente em **Actions → Release → Run workflow**.

Para publicar uma versão:

1. Atualize a versão em `package.json` e `src-tauri/tauri.conf.json`.
2. Faça o commit, crie a tag e envie:

   ```bash
   git add -A
   git commit -m "Release v1.0.1"
   git tag v1.0.1
   git push origin main --tags
   ```

O workflow confere se a tag corresponde à versão em `src-tauri/tauri.conf.json`, compila Linux
(AppImage, `.deb`, `.rpm`), Windows (`.msi` e `-setup.exe`) e macOS (`.dmg` universal), envia os
pacotes para um release em rascunho e só publica quando todas as plataformas terminam com
sucesso. Tags com hífen (por exemplo, `v0.2.0-beta.1`) são marcadas como pré-release.

> Se o workflow não conseguir criar o release, permita escrita em
> **Settings → Actions → General → Workflow permissions** (“Read and write permissions”).

Os artefatos não usam um certificado de assinatura confiável: Linux e Windows ficam sem
assinatura, e o macOS usa assinatura ad-hoc (`"signingIdentity": "-"`), sem notarização. Para
assinar e notarizar, adicione os secrets habituais do Tauri (`APPLE_CERTIFICATE`,
`APPLE_CERTIFICATE_PASSWORD`, `APPLE_SIGNING_IDENTITY`, `APPLE_ID`, `APPLE_PASSWORD`,
`APPLE_TEAM_ID`) e passe-os para o step `tauri-action` no `release.yml`;
`APPLE_SIGNING_IDENTITY` substitui a identidade ad-hoc.

O ícone do app é o SVG transparente em tons de cinza em `assets/icon.svg`. Depois de alterá-lo,
regenere os ícones das plataformas com:

```bash
npx tauri icon assets/icon.svg
```


## Solução de problemas

### `failed to run 'cargo metadata' ... No such file or directory`

O Tauri CLI não encontrou o `cargo`. Instale o Rust (se ainda não instalou) e carregue o
ambiente no terminal atual — ou simplesmente abra um terminal novo:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"
cargo --version   # deve mostrar 1.90 ou superior
```

Se o `cargo` estiver em `~/.cargo/bin` mas o comando continuar falhando, confira com
`echo $PATH` e adicione ao seu shell (`~/.bashrc`, `~/.zshrc`, etc.):

```bash
export PATH="$HOME/.cargo/bin:$PATH"
```

### `linker 'cc' not found`, `pkg-config` ou `webkit2gtk-4.1 was not found`

O Rust sozinho não compila as dependências nativas do Tauri: é preciso um compilador C e as
bibliotecas de desenvolvimento do SO (GTK/WebKitGTK no Linux). Instale os pacotes da seção
[Pré-requisitos](#pré-requisitos) — no Ubuntu/Debian, por exemplo:

```bash
sudo apt install build-essential pkg-config libwebkit2gtk-4.1-dev \
  libayatana-appindicator3-dev librsvg2-dev libxkbcommon-dev
```

Se você estiver rodando dentro de um Flatpak/container **sem compilador e sem `sudo`**, o build
não funciona nesse sandbox: execute `npm run tauri dev` em um terminal do sistema hospedeiro
(fora do sandbox) ou em um container/VM com as dependências acima. O frontend ainda pode ser
desenvolvido isoladamente com `npm run dev`.

### `Couldn't get a key from code: Backquote` (Linux)

Essa mensagem vem do `tao` (camada de janelas do Tauri) e aparece em builds de desenvolvimento
quando o layout do teclado usa *dead keys* (por exemplo, ABNT2): o evento da combinação
Ctrl + acento grave é descartado antes de chegar à interface, então o botão **Gravar atalho**
não consegue capturá-la.

Use o campo **Ou digite o atalho** em Configurações e informe
`CommandOrControl+Backquote` (o preset ao lado define a mesma combinação), depois salve. O
atalho em si é registrado pelo sistema normalmente. Se aparecer `Unable to find keycode`,
escolha outra tecla para o atalho.

### O atalho global não funciona no Wayland

O Tauri usa captura de teclas **X11**. Em sessões **Wayland** (padrão em GNOME/Zorin
recentes), o atalho pode até ser registrado via XWayland, mas não recebe as teclas
pressionadas em aplicativos Wayland nativos. Confira a sessão com:

```bash
echo $XDG_SESSION_TYPE
```

Se a saída for `wayland`, use uma destas alternativas:

1. Clique no ícone da bandeja → **Abrir histórico**.
2. Crie um atalho personalizado no sistema executando o app com `--toggle`:
   - **GNOME/Zorin:** Configurações → Teclado → Atalhos personalizados → Adicionar.
   - **KDE:** Configurações do Sistema → Atalhos → Comando/URL.
   - Use o caminho mostrado em **Configurações → Atalho global**, no aviso “Wayland
     detectado”, seguido de `--toggle`:
     ```bash
     "/caminho/para/cliphistory" --toggle
     ```
   - Mantenha **Iniciar com o sistema** ligado. O `--toggle` reutiliza a instância em execução
     ou inicia o app se ele ainda não estiver rodando.
   - Para testar o popup sem depender do atalho, rode o mesmo comando no terminal.
   - Em desenvolvimento, o caminho aponta para o binário de debug e depende do
     `npm run dev`; para uso diário, use o build de produção.
3. Para usar o atalho interno e a colagem automática, entre em uma sessão **X11** (no Zorin,
   escolha “Zorin Desktop on Xorg” na tela de login).

> O Wayland nativo não permite que apps posicionem janelas top-level, e os window managers
> X11/XWayland podem sobrescrever a posição de qualquer forma, então o popup é sempre exibido
> **centralizado na tela**. Quando o XWayland está disponível o ClipHistory o usa
> automaticamente; para forçar Wayland nativo, inicie com `GDK_BACKEND=wayland` (aí o compositor
> controla a posição).
>
> Se o log de inicialização mostrar `gdk_backend=Some("wayland")`, há um
> `GDK_BACKEND=wayland` explícito no ambiente — remova-o (`unset GDK_BACKEND`) e reinicie. Se o
> log mostrar `display=None` ou `xwayland=false`, o XWayland não está acessível. A janela de
> Configurações informa qual desses casos se aplica.

### O popup abre mas não recebe foco

Depois de exibir o popup, o ClipHistory continua pedindo foco a cada 150 ms por cerca de 4,5
segundos (com o `present()` do GTK no Linux), para que janelas mapeadas com atraso ainda
recebam o teclado. Eventos de perda de foco recebidos antes do primeiro foco são ignorados, para
que um mapeamento atrasado não feche o popup cedo demais. Se mesmo assim ele não focar:

1. Garanta que só há uma instância rodando (`pkill -f cliphistory`, depois inicie de novo). Um
   build antigo pode continuar respondendo ao `--toggle` pelo plugin de instância única.
2. Rode um build de debug e confira no terminal `cliphistory: popup focused` ou
   `cliphistory: popup never gained focus`.
3. No Wayland o app usa XWayland automaticamente quando disponível; `GDK_BACKEND=wayland`
   força Wayland nativo, onde o compositor pode se recusar a dar foco a uma janela que não foi
   ativada pelo usuário.

### O build de desenvolvimento parece ignorar as mudanças

Se **Iniciar com o sistema** foi ativado enquanto rodava um build de desenvolvimento, uma
instância antiga do app pode já estar em execução. O plugin de instância única encaminha a nova
execução para ela e encerra, então o código recém-compilado não entra em uso. Saia da instância
pela bandeja (ou `pkill -f cliphistory`), desative **Iniciar com o sistema** durante o
desenvolvimento e inicie de novo. No terminal do `tauri dev` deve aparecer o log da nova
instância:

```text
cliphistory: initialized (session=..., gdk_backend=...)
```

Se essa linha não aparecer, o processo com que você está interagindo é uma instância antiga.



## Usando

1. Na primeira execução, a janela de **Configurações** abre automaticamente.
2. Grave um atalho (ou mantenha o padrão; se a tecla não for capturada, digite o atalho
   manualmente) e clique em **Salvar alterações**.
3. Copie textos ou imagens normalmente.
4. Pressione o atalho em qualquer aplicativo para abrir o histórico.
5. Use `↑`/`↓` para navegar, `Enter` ou clique para copiar de volta, `Esc` para fechar.
6. Com o **auto paste** ligado, o item volta para a área de transferência e é colado
   automaticamente no aplicativo anterior. Desligado, basta colar com `Ctrl+V` / `⌘V`.

O idioma da interface pode ser trocado no topo da janela de Configurações. Inglês é o padrão;
**Português** muda o app (e o menu da bandeja) para português imediatamente.


## Onde ficam os dados

Tudo é local, no diretório de dados do app definido pelo identificador
`com.%user%.cliphistory`:

| SO      | Banco e imagens                                                    | Configurações                                   |
| ------- | ------------------------------------------------------------------ | ----------------------------------------------- |
| Linux   | `~/.local/share/com.%user%.cliphistory/`                            | `~/.config/com.%user%.cliphistory/settings.json` |
| macOS   | `~/Library/Application Support/com.%user%.cliphistory/`             | idem                                            |
| Windows | `%APPDATA%\com.%user%.cliphistory\`                                 | idem                                            |

O banco é `cliphistory.db`; as imagens ficam em `images/`.

> **Privacidade:** o histórico é salvo em texto claro no seu disco. Evite copiar senhas e dados
> sensíveis, ou limpe o histórico depois. Não há sincronização nem envio para a nuvem.

## Atalhos e detalhes de plataforma

- O atalho padrão é `CommandOrControl+Shift+V`, ou seja, `Ctrl+Shift+V` no Windows/Linux e
  `⌘ ⇧ V` no macOS. Ele pode ser trocado em Configurações. Por ser global, ele tem prioridade
  sobre o atalho de mesmo nome dos outros apps (por exemplo, “colar sem formatação” no
  navegador); escolha outra combinação se isso incomodar.
- **macOS:** o app usa `macos-private-api` para a janela transparente do popup. Isso não é
  compatível com distribuição pela Mac App Store. Para a **colagem automática**, autorize o
  ClipHistory em **Ajustes do Sistema → Privacidade e Segurança → Acessibilidade**; sem isso os
  eventos de teclado são ignorados pelo sistema.
- **Linux:** atalhos globais e colagem automática funcionam em sessões **X11**. Em **Wayland**,
  o sistema pode bloquear o atalho e a simulação de teclado não chega a aplicativos Wayland
  nativos; use o ícone da bandeja ou um atalho personalizado do sistema executando o binário
  com `--toggle`. Algumas áreas de trabalho precisam do pacote `libayatana-appindicator3` para
  o ícone aparecer. Em layouts com *dead keys* (por exemplo, ABNT2), a gravação de atalhos como
  Ctrl + acento grave pode ser descartada pela camada de janelas — use o campo de atalho manual
  nas Configurações. No Wayland o popup é sempre exibido **centralizado na tela**; quando o
  XWayland está disponível o app o usa automaticamente, e `GDK_BACKEND=wayland` força Wayland
  nativo.
- **Windows:** o popup não aparece na barra de tarefas (`skipTaskbar`), apenas na bandeja. A
  colagem automática não consegue injetar teclas em janelas elevadas (executadas como
  administrador) a menos que o ClipHistory também esteja elevado.
- **AppImage:** o autostart aponta para o caminho do AppImage; mover o arquivo exige reativar a
  opção.

## Limitações conhecidas (v1)

- A captura é feita por polling: conteúdos muito efêmeros (copiados e sobrescritos em menos de
  ~1 s) podem passar batido.
- Somente **texto puro** e **imagens raster** são capturados. HTML formatado, RTF e listas de
  arquivos são ignorados.
- A colagem automática pode falhar se o aplicativo de destino demorar a recuperar o foco, em
  sessões Wayland, no macOS sem permissão de Acessibilidade e em janelas elevadas no Windows.
  Nesses casos o item continua na área de transferência.
- Sem criptografia, categorias, sincronização ou exportação.
- Não há teste automatizado do backend nesta versão; o lado Rust precisa ser compilado com as
  dependências de sistema instaladas.

## Estrutura

```text
.
├── index.html
├── package.json
├── vite.config.ts
├── tsconfig.json
├── README.md                    # documentação em inglês (padrão)
├── README.pt-BR.md              # documentação em português
├── assets/
│   └── icon.svg                 # fonte dos ícones
├── src/                         # frontend React
│   ├── main.tsx                 # escolhe a tela pela label da janela
│   ├── App.tsx
│   ├── types.ts
│   ├── styles.css
│   ├── components/
│   │   ├── HotkeyRecorder.tsx   # gravação da combinação de teclas
│   │   └── Icons.tsx
│   ├── i18n/                    # inglês (padrão) + português
│   │   ├── dictionaries.ts      # tabelas de tradução
│   │   ├── translate.ts
│   │   ├── errors.ts            # tradução de erros do backend
│   │   └── index.tsx            # provider e hook useI18n
│   ├── lib/
│   │   ├── ipc.ts               # wrappers dos comandos Tauri
│   │   ├── events.ts            # listen do Tauri + eventos no preview
│   │   ├── mock.ts              # dados falsos do preview no navegador
│   │   ├── format.ts
│   │   └── shortcut.ts          # aceleradores ↔ teclas
│   └── windows/
│       ├── Popup.tsx
│       └── Settings.tsx
└── src-tauri/
    ├── Cargo.toml
    ├── tauri.conf.json
    ├── capabilities/default.json
    ├── icons/
    └── src/
        ├── main.rs
        ├── lib.rs               # plugins, estado, janelas, comandos
        ├── commands.rs          # IPC (CRUD, copiar, salvar config.)
        ├── clipboard_watcher.rs # polling + deduplicação
        ├── db.rs                # SQLite, imagens, retenção
        ├── models.rs            # Settings, Entry, Stats
        ├── paste.rs             # simulação de Ctrl+V / ⌘V (enigo)
        ├── popup.rs             # mostrar/esconder janelas
        ├── settings.rs          # persistência de configurações
        └── tray.rs              # menu da bandeja
```

## Comandos IPC

| Comando            | Descrição                                             |
| ------------------ | ----------------------------------------------------- |
| `get_entries`      | Lista itens (com busca opcional).                     |
| `get_settings`     | Lê as configurações atuais.                           |
| `save_settings`    | Valida, salva, re-registra o atalho e aplica autostart.|
| `get_stats`        | Totais de itens, fixados e imagens.                   |
| `shortcut_status`  | Informa se o atalho está realmente registrado.        |
| `copy_entry`       | Copia o item, fecha o popup e, se configurado, simula colar. |
| `toggle_pin`       | Fixa/desafixa um item.                                |
| `delete_entry`     | Remove um item (e o arquivo de imagem).               |
| `clear_history`    | Remove todos os itens não fixados.                    |
| `hide_popup`       | Esconde o popup.                                      |
| `open_settings`    | Abre a janela de configurações.                       |
| `hide_settings`    | Esconde a janela de configurações.                    |

## Roadmap

- Ajuste fino do atraso da colagem automática (apps que demoram a recuperar o foco).
- Suporte a HTML/RTF e listas de arquivos.
- Metadados de origem (aplicativo de onde veio a cópia).
- Criptografia do banco e modo "não guardar senhas".
- Mais idiomas na interface.
- Migrações versionadas do banco e escrita atômica das configurações.
- Testes de integração do backend e CI de build multiplataforma.

## Licença

Distribuído sob a [Licença MIT](LICENSE).

Copyright © 2026 Luigi Henrick.

---

Feito com Tauri 2, React 19 e TypeScript.
