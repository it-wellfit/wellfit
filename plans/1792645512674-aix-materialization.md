# Plan: Materialize AIX 思考作業系統

## Goal
Operationalize the AIX methodology into this workspace: an `/aix` collaboration command + five tool templates under `.kilo/`, plus a governance-tracked entry in `2026/Knowledge/`. Covers Personal / Team / Enterprise versions.

## Context (verified)
- `Dev/.kilo/` exists but only has `.gitignore` + `agent-manager.json` (no `command/`, `agent/`, `kilo.json`). Must be created.
- Global `kilo.jsonc` allows read/edit/glob/grep broadly; `kilo.json` not present locally.
- `2026/Knowledge/` governs documentation: ADRs (ADR-0001 source hierarchy, ADR-0003 fixture provenance, ADR-0010 ISO 27001, ADR-0011 compliance naming & crosswalk), source-register, traceability, invariant "no synthetic-as-real".
- Wellfit Rust project is separate and OUT OF SCOPE for AIX (methodology/docs only).

## Design decisions (resolved)
1. Locations — `.kilo/command/aix.md` (command); `.kilo/templates/aix/*.md` (5 templates); `2026/Knowledge/adr/ADR-0012-*.md` + `2026/Knowledge/sources/source-register.md` (governance).
2. Command content — embed 8-dimension collaboration spec, 6 gate questions, output requirements, template references, Personal/Team/Enterprise scaling.
3. Case-study figures marked 假設值/示範性, never real data (ADR-0003).
4. Out of scope — Agent backend, Rust/Wellfit integration, kilo.json creation, committing changes.

## Ordered implementation tasks
1. Read `2026/Knowledge/sources/source-register.md` and `ADR-0011-compliance-naming-and-crosswalk-schema.md` to match naming/format conventions.
2. Create `.kilo/command/aix.md` — `/aix` collaboration command (8 dimensions, 6 gates, output requirements, template references, scaling guidance).
3. Create `.kilo/templates/aix/` five templates:
   - `problem-definition-card.md` (目標、問題、使用者、成功指標、範圍)
   - `evidence-assumptions-table.md` (已知事實、推論、待驗證假設、資料來源)
   - `option-decision-matrix.md` (方案、效益、成本、風險、可逆性、依據)
   - `human-ai-division-risk-table.md` (工作、負責者、權限、核准、停止條件)
   - `verification-learning-record.md` (預期vs實際、錯誤、改善措施、後續責任)
   - `README.md` — Personal/Team/Enterprise scaling + §十 guardrail (avoid bureaucracy).
4. Create `2026/Knowledge/adr/ADR-0012-adopt-aix-thinking-system.md` adopting AIX as governed methodology; register AIX in source-register.
5. Validate — dry-run `/aix` on a low-risk sample task; confirm 8-dimension output + gate answers; confirm no example figures presented as real.
6. Stop — await user go-ahead; this agent writes plan only, no source/doc changes.

## Risks
- Templates become bureaucracy — mitigate via §十 guardrail (simplify when process cost exceeds benefit).
- Example data leakage — mark all case-study figures as 假設值/示範性.

## Open questions
- None. Plan implementation-ready.
