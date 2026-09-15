---
'@i18n-scanner-rs/main': patch
---

Make scanned key sets independent of dependency traversal order by resolving imports, aliases, and reexports against the complete module graph, including cycles, while preserving namespace and keyProp metadata.
