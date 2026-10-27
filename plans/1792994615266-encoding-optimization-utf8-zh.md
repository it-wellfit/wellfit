# Encoding Optimization Plan — UTF-8 Traditional/Simplified Chinese Compatible

## Goal
確保工作區所有檔案採用 UTF-8 編碼，同時支援繁體中文（Big5 超集）與簡體中文（GB2312/GBK 超集），零亂碼，LF 換行，二進位檔案保護。

## Current State
- 無根層 `.editorconfig`
- `ekpi-platform/.gitattributes`：`* text=auto`，無 `encoding=utf-8` 宣告
- `Wellfit/.gitattributes`：`* text=auto eol=lf`，無 `encoding=utf-8` 宣告
- 中文檔案（AGENTS.md、GEMINI.md、WORKSPACE_CHARTER.md）目前為 UTF-8，但無統一強制規範
- 兩個獨立 Rust 專案（ekpi-platform、Wellfit）與 SyncMesh，各自 `.gitattributes`

## Target State
1. 根層 `.editorconfig`：宣告 `charset = utf-8`，`end_of_line = lf`，涵蓋所有文字檔副檔名；二進位檔案標記 `binary`
2. `ekpi-platform/.gitattributes`：新增 `* text=auto encoding=utf-8` 與 `eol=lf` 對應規則
3. `Wellfit/.gitattributes`：新增 `* text=auto encoding=utf-8` 與 `eol=lf` 對應規則；保留現有 CR-only fixtures 保護
4. 同步檢查現有中文檔案是否為純 UTF-8（無 BOM 爭議，UTF-8 without BOM 為跨平台標準）

## Files to Create
- `C:\Users\stephanie.huang\Desktop\新人訓練\Projects\2026\Dev\.editorconfig`

## Files to Edit
- `C:\Users\stephanie.huang\Desktop\新人訓練\Projects\2026\Dev\ekpi-platform\.gitattributes`
- `C:\Users\stephanie.huang\Desktop\新人訓練\Projects\2026\Dev\Wellfit\.gitattributes`

## Validation
- 讀取各關鍵中文檔案（AGENTS.md、GEMINI.md、WORKSPACE_CHARTER.md）確認無亂碼
- 確認 `.editorconfig` 與 `.gitattributes` 兩者一致
- 在支援 EditorConfig 的編輯器（VS Code 等）中開啟中文檔案，確認正確顯示繁簡體

## Rollback
- 刪除 `.editorconfig` 並還原兩個 `.gitattributes` 為原內容即可
