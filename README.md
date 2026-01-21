# GitHub Sponsors Discord Webhook

A small Vercel function to receive GitHub Sponsors webhook events and forward new sponsorship notifications to a Discord channel via webhook.

## ⚙️ Environment Variables

- `GITHUB_SECRET`: github webhook secret
- `DISCORD_WEBHOOK_URL`: discord webhook url

## 🛠️ Building

To build the project, ensure you have Rust installed. Then run:

```bash
cargo build --release
```

## 📜 License

GitHub Sponsors Discord Webhook is licensed under the GPLv3 license. Please see the [license file](LICENSE) for more information.
[tl;dr](https://www.tldrlegal.com/license/gnu-general-public-license-v3-gpl-3) you may copy, distribute and modify GitHub Sponsors Discord Webhook as long as you track changes/dates in source files.
Any modifications to GitHub Sponsors Discord Webhook must also be made available under the GPL,
along with build & install instructions.
