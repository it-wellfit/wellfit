# Knowledge Base Index

This project consumes the cross-project knowledge base at `Projects/2026/Knowledge/`.

- **Location:** `C:\Users\stephanie.huang\Desktop\新人訓練\Projects\2026\Knowledge\`
- **Version control:** Independent git repo (private remote). Initialized 2026-11-06 (Phase 0).
- **Consumption:** CI checks out a pinned SHA (see `vars.KNOWLEDGE_SHA`), never `main`. Dev may use latest `main`.
- **Verification:** `Knowledge/tools/kb-verify.ps1` (exit 0 = pass).
- **Authority:** ADR-0004 (Approved) — single-direction reference (project → KB). KB is read-only for implementers.
- **Pinned SHA:** (to be set after Phase 0 stabilizes)