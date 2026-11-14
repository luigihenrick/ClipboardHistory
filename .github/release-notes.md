### What's new

- **Update check fixed**: the manual check no longer depends on the system `curl` binary; it uses
  the webview HTTP stack, so it also works from the AppImage. Settings checks when opened and the
  **Check for updates** button remains available for on-demand checks.
- **AppImage focus fixed**: on cold starts the search field now receives focus reliably, using the
  native window-focus event and retries that cover slow AppImage/WebKit startup.
- **Popup position after idle fixed**: after a monitor wakes up, the popup no longer appears at
  the left edge; zero-sized work areas are ignored and the centered position is reapplied for a
  few seconds.

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

### Novidades

- **Verificação de atualizações corrigida**: a checagem manual não depende mais do binário
  `curl` do sistema; usa a pilha HTTP do webview, então também funciona no AppImage. As
  Configurações verificam ao abrir e o botão **Verificar atualizações** continua disponível.
- **Foco corrigido no AppImage**: em cold starts o campo de busca agora recebe foco de forma
  confiável, usando o evento nativo de foco da janela e retentativas que cobrem a inicialização
  lenta do AppImage/WebKit.
- **Posicionamento após inatividade corrigido**: depois que o monitor acorda, o popup não aparece
  mais na lateral esquerda; áreas úteis com tamanho zero são ignoradas e a posição centralizada é
  reaplicada por alguns segundos.

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
