### What's new

- **History limit fixed**: the popup now lists every retained entry instead of stopping at 100, so
  increasing the items limit works as expected. Added backend and frontend regression tests for
  limits above 100.
- **Much broader frontend test suite**: the suite now has 94 tests covering the popup, settings,
  hotkey recorder, icons, i18n, IPC, preview backend, focus scheduling, formatting, shortcuts and
  update payload validation.
- **Coverage gates in CI**: the release workflow now enforces at least 90% statements, lines and
  functions (and 80% branches) and publishes the HTML coverage report as a build artifact.
- **Reliability release**: combines the history-limit fix with the expanded test suite and
  coverage gates.

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

- **Limite do histórico corrigido**: o popup agora lista todas as entradas retidas em vez de parar
  em 100, então aumentar o limite de itens funciona como esperado. Foram adicionados testes de
  regressão no backend e no frontend para limites acima de 100.
- **Suíte de testes do frontend muito maior**: agora são 94 testes cobrindo popup, configurações,
  gravador de atalhos, ícones, i18n, IPC, backend de preview, agendamento de foco, formatação,
  atalhos e validação de payloads de atualização.
- **Limites de cobertura no CI**: o workflow de release agora exige pelo menos 90% de statements,
  linhas e funções (e 80% de branches) e publica o relatório HTML de cobertura como artifact.
- **Release de confiabilidade**: combina a correção do limite do histórico com a suíte de testes
  ampliada e os gates de cobertura.

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
