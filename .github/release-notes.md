## ClipHistory 1.1.0

### What's new in 1.1.0

- **Image previews in the list**: image entries now show a small 64 px JPEG preview; existing
  images are backfilled in the background.
- **More reliable popup focus** on Linux/X11/XWayland: focus-on-map is configured before the
  first show, GTK `present()` keeps retrying until the popup is focused, and an X11 fallback
  (`_NET_ACTIVE_WINDOW` + `XSetInputFocus`) forces focus when the window manager ignores the
  request.
- **Faster popup opening**: the list now loads at most 100 entries, previews are capped at 300
  characters, the UI refresh starts before the window is mapped, and memoized rows keep keyboard
  navigation smooth.
- **MIT License**: added the `LICENSE` file and license metadata; the license is included in the
  installers.
- **Documentation**: added the license badge/section and notes on regenerating the grayscale
  icons.

Local clipboard history for Windows, Linux and macOS. Nothing leaves your computer.

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

### Novidades na 1.1.0

- **Previews de imagens na lista**: entradas de imagem agora mostram um preview JPEG de 64 px;
  imagens antigas são processadas em background.
- **Foco mais confiável** no popup em Linux/X11/XWayland: o foco ao mapear é configurado antes
  da primeira exibição, o `present()` do GTK continua tentando até o popup focar, e um fallback
  X11 (`_NET_ACTIVE_WINDOW` + `XSetInputFocus`) força o foco quando o window manager ignora o
  pedido.
- **Popup mais rápido**: a lista carrega no máximo 100 entradas, os previews são limitados a 300
  caracteres, a atualização da interface começa antes da janela ser mapeada e as linhas usam
  memoização para manter a navegação por teclado fluida.
- **Licença MIT**: adicionado o arquivo `LICENSE` e os metadados de licença; a licença é
  incluída nos instaladores.
- **Documentação**: badge/seção de licença e nota para regenerar os ícones em tons de cinza.

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
