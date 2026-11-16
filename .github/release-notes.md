### What's new

- **Release page link fixed**: opening the GitHub release from Settings now works from the AppImage.
  The opener uses the host desktop tools with a clean environment and reports failures instead of
  failing silently.
- **Diagnostics panel**: Settings now includes system, monitor, database, update and popup-focus
  information, plus recent logs that can be copied or opened directly. Focus timings show how long
  the window, webview and search field take to become ready.
- **Quality and test coverage**: added Rust tests for diagnostics, logging and update payloads,
  formatting/clippy checks, an AppImage smoke test in CI, and release asset verification before
  publishing.
- **Release pipeline fixed**: the AppImage smoke test now finds the bundle under
  `src-tauri/target/release/bundle`, and the release workflow verifies all required installers
  before publishing.

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

- **Link do release corrigido**: abrir o release no GitHub pelas Configurações agora funciona no
  AppImage. O opener usa as ferramentas do sistema com um ambiente limpo e reporta falhas em vez
  de falhar silenciosamente.
- **Painel de diagnóstico**: as Configurações agora incluem informações do sistema, monitores,
  banco, atualização e tempos de foco do popup, além de logs recentes que podem ser copiados ou
  abertos diretamente. Os tempos mostram quanto a janela, o webview e o campo de busca levam para
  ficar prontos.
- **Qualidade e cobertura de testes**: adicionados testes Rust para diagnóstico, logging e payload
  de atualização, verificação de formatação/clippy, teste de fumaça do AppImage no CI e validação
  dos assets antes de publicar.
- **Pipeline de release corrigido**: o teste de fumaça do AppImage agora encontra o bundle em
  `src-tauri/target/release/bundle`, e o workflow valida todos os instaladores necessários antes
  de publicar.

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
