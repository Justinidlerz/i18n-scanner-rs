---
'@i18n-scanner-rs/main': patch
---

Fix native scanner crashes caused by TypeScript declarations inside function bodies by transforming TypeScript before minification. Preserve JSX-based key extraction, value imports, and the existing dependency analysis.
