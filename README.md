# iSearch CLI™

iSearch CLI™ is a modern, premium, and lightning-fast command-line search assistant.

## Features

- **Beautiful Interactive TUI**: Terminal User Interface with full keyboard & mouse navigation.
- **Cross-Platform**: Support for Windows, macOS, Linux, and Android (via Termux).
- **Integrated Donation System**: Support the project directly from your terminal without leaving the command line!

---

## Installation & Getting Started

To compile and run iSearch CLI™:

```bash
# Clone the repository
git clone https://github.com/erikraft/iSearch-CLI.git
cd iSearch-CLI

# Run the interactive CLI
cargo run
```

---

## Donate

Allowing users to support the project financially directly from their favorite environment is a core pillar of iSearch CLI™. We provide a beautiful, seamless, and fully integrated terminal-based PIX payment system.

### How to open the donation screen

You can launch the donation interface in two different ways:

1. **Directly from your shell**:
   ```bash
   isearch donate
   # Or using cargo run:
   cargo run -- donate
   ```

2. **From inside the interactive CLI**:
   Simply run `donate` when you are already inside the interactive iSearch CLI environment:
   ```text
   iSearch> donate
   ```

### How PIX works

PIX is an instant, secure, and modern payment method created by the Banco Central do Brasil.
1. When you choose an amount (or enter a custom value) and optional message on our donation screen, iSearch CLI™ generates a valid, standard-compliant EMV Co QR Code payload.
2. The terminal displays a **high-quality scannable QR Code** rendered with Unicode block characters.
3. Simply scan the QR code using your favorite bank's mobile app to complete the instant payment safely.
4. You can also copy the PIX payload instantly to your system clipboard by pressing `C` or `Enter` on the copy button.

### Supported platforms

The donation system and clipboard auto-copying are fully optimized and supported natively across:
- **macOS** (utilizing native clipboard APIs or `pbcopy`)
- **Windows** (utilizing PowerShell or system `clip` utilities)
- **Linux** (using `xclip`, `xsel` under X11 or `wl-copy` under Wayland)
- **Android / Termux** (utilizing `termux-clipboard-set` package)

### Privacy considerations

- **Offline-First**: All PIX code formatting, CRC16 CCITT validation, and QR Code generation are done locally on your machine.
- **No telemetry / trackers**: iSearch CLI™ never sends any personal data, session logs, or payment information over the network.
- **Secure payment**: Since the payment is routed directly through standard PIX keys via your trusted banking app, no credit card numbers or sensitive details are exposed or requested.

---

### Terminal Donation Interface Example

When launching the donation flow, you'll be greeted with our professional interactive screen:

```text
╭────────────────────────────────────────────╮
│             Support iSearch CLI™           │
├────────────────────────────────────────────┤
│                                            │
│ Thank you for supporting the project.      │
│                                            │
│ Choose an amount:                          │
│                                            │
│   ● R$ 5                                   │
│   ○ R$ 10                                  │
│   ○ R$ 20                                  │
│   ○ R$ 50                                  │
│   ○ R$ 100                                 │
│   ○ Custom Amount                          │
│                                            │
│ Message (Optional):                        │
│ [ Supporting iSearch CLI™                ] │
│                                            │
│          [ Generate PIX QR Code ]          │
│                                            │
╰────────────────────────────────────────────╯
```

---

## Configuration

iSearch CLI™ respects your preferences. You can customize the default donation options and keys inside a `config.toml` file in your workspace:

```toml
[donation]
pix_key = "11925416678"
currency = "BRL"
default_values = [5, 10, 20, 50, 100]
```

---

## License

MIT License. See [LICENSE](LICENSE) for details.
