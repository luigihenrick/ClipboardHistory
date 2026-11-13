## ClipHistory 1.2.0

### What's new in 1.2.0

- **First popup centered reliably**: the position is calculated from the work area of the monitor
  under the cursor and re-applied once after the window is mapped, so the window manager cannot
  leave the first popup off-center.
- **Pinned items are protected from deletion**: they survive clearing and retention limits, show
  an always-visible pin icon, and follow the normal recency order; copying one moves it to the
  top like any other item.
- **Update check**: at startup, ClipHistory checks GitHub for a newer release and shows the
  version and release notes in Settings, with a button to open the GitHub release.
- **Auto-paste fixed**: after choosing an item, ClipHistory now restores focus to the window
  that was active before the popup and then sends `Ctrl+V` / `⌘V`.
- **Native Wayland**: when `ydotool` and `ydotoold` are installed and running, ClipHistory uses
  them automatically to paste into native Wayland apps, with the correct syntax for both
  ydotool 0.1.x and 1.x.
- If auto-paste still fails on Wayland, configure `ydotool` or use an X11 session; the item
  remains on the clipboard.

Local clipboard history for Windows, Linux and macOS. Clipboard data never leaves your computer;
the only network request is the update check against GitHub.

### Windows

Download the `.msi` installer (or the `-setup.exe` NSIS installer) and run it.

If Windows SmartScreen appears, choose **More info → Run anyway**.

### macOS

Download the `.dmg`, open it and drag **ClipHistory** to **Applications**.

The build is ad-hoc signed but not notarized. If macOS blocks it, right-click the app → **Open**,
or run:

```bash
xattr -dr com.apple.quarantine /Applications/ClipHistory.app
```

### Linux

- **AppImage:** `chmod +x ClipHistory_*.AppImage && ./ClipHistory_*.AppImage`
  If FUSE is unavailable, use `./ClipHistory_*.AppImage --appimage-extract-and-run`.
- **Debian/Ubuntu:** `sudo apt install ./ClipHistory_*_amd64.deb`
- **Fedora:** `sudo dnf install ./ClipHistory-*.rpm`

### Notes

- On Wayland, global shortcuts and auto paste are limited by the desktop. Use the tray icon,
  `--toggle`, or an X11 session. See the README for details.
- **Windows 10** may need the [WebView2 Runtime](https://developer.microsoft.com/microsoft-edge/webview2/)
  (Windows 11 includes it).
- Linux packages use the system WebKitGTK 4.1 (`libwebkit2gtk-4.1-0` on Debian/Ubuntu,
  `webkit2gtk4.1` on Fedora).
- These are unsigned Linux/Windows builds and an ad-hoc signed macOS build from the project's
  GitHub Actions workflow.

<details>
<summary>Português (Brasil)</summary>

### Novidades na 1.2.0

- **Primeiro popup centralizado de forma confiável**: a posição é calculada pela área útil do
  monitor sob o cursor e reaplicada uma vez após a janela ser mapeada, para o window manager não
  deixar o primeiro popup fora do centro.
- **Itens fixados são protegidos contra exclusão**: sobrevivem à limpeza e ao limite de retenção,
  mostram um ícone de pin sempre visível e seguem a ordem normal de uso; copiar um item fixado o
  leva ao topo como qualquer outro.
- **Verificação de atualizações**: no startup, o ClipHistory consulta o GitHub por uma versão
  mais nova e mostra a versão e as notas do release nas Configurações, com um botão para abrir
  o release no GitHub.
- **Colagem automática corrigida**: ao escolher um item, o ClipHistory agora devolve o foco à
  janela que estava ativa antes do popup e só então envia `Ctrl+V` / `⌘V`.
- **Wayland nativo**: quando `ydotool` e `ydotoold` estão instalados e em execução, o ClipHistory
  os usa automaticamente para colar em aplicativos Wayland nativos, com a sintaxe correta para
  ydotool 0.1.x e 1.x.
- Se a colagem automática ainda falhar no Wayland, configure o `ydotool` ou use uma sessão X11;
  o item continua na área de transferência.

### Windows

Baixe o instalador `.msi` (ou o `-setup.exe`) e execute. Se o SmartScreen aparecer, escolha
**Mais informações → Executar assim mesmo**.

### macOS

Baixe o `.dmg`, abra e arraste o **ClipHistory** para **Aplicativos**. O build tem assinatura
ad-hoc, mas não é notarizado; se o macOS bloquear, clique com o botão direito no app → **Abrir**
ou execute:

```bash
xattr -dr com.apple.quarantine /Applications/ClipHistory.app
```

### Linux

- **AppImage:** `chmod +x ClipHistory_*.AppImage && ./ClipHistory_*.AppImage`
  Se o FUSE não estiver disponível, use `./ClipHistory_*.AppImage --appimage-extract-and-run`.
- **Debian/Ubuntu:** `sudo apt install ./ClipHistory_*_amd64.deb`
- **Fedora:** `sudo dnf install ./ClipHistory-*.rpm`

### Observações

- No Wayland, o atalho global e o auto paste dependem do compositor. Use o ícone da bandeja,
  `--toggle` ou uma sessão X11. Veja o README para detalhes.
- **Windows 10** pode precisar do
  [WebView2 Runtime](https://developer.microsoft.com/microsoft-edge/webview2/) (o Windows 11
  já inclui).
- Os pacotes Linux usam o WebKitGTK 4.1 do sistema (`libwebkit2gtk-4.1-0` no Debian/Ubuntu,
  `webkit2gtk4.1` no Fedora).
- Estes são builds sem assinatura no Linux/Windows e um build macOS com assinatura ad-hoc
  gerados pelo workflow do GitHub Actions do projeto.

</details>
