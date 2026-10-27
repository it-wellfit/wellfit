# Plan: 設定 Web Search (WEBSEARCH) MCP 工具

## 目標

在 `.kilo/kilo.jsonc` 中新增 Web Search MCP 伺服器設定，讓 Kilo 具備免費的網頁搜尋能力。

## 背景

- 目前 `.kilo/kilo.jsonc` 僅設定 `sequentialthinking` MCP 伺服器。
- Google Custom Search API 免費方案每天 100 次查詢，但需要 API Key + Search Engine ID (CX)。
- Brave Search 免費方案每月 2000 次查詢，設定較簡單（只需 BRAVE_API_KEY）。
- Kilo 內建 `websearch` 工具已存在，但 MCP 伺服器可提供更豐富的搜尋能力。

## 決策

1. **主要方案**：使用官方 MCP 伺服器 `@modelcontextprotocol/server-brave-search`（Brave Search 免費方案），設定簡單、維護穩定。
2. **備用方案**：Google Custom Search MCP 伺服器（需 API Key + CX），列為可選設定。
3. **環境變數**：API Key 透過環境變數注入，不寫入版本控制。

## 實作步驟

### Step 1: 修改 `.kilo/kilo.jsonc`

在 `mcp` 區塊新增 `brave-search` 伺服器：

```jsonc
{
  "$schema": "https://app.kilo.ai/config.json",
  "mcp": {
    "sequentialthinking": {
      "type": "local",
      "command": ["npx", "-y", "@modelcontextprotocol/server-sequential-thinking"],
      "enabled": true
    },
    "brave-search": {
      "type": "local",
      "command": ["npx", "-y", "@modelcontextprotocol/server-brave-search"],
      "environment": {
        "BRAVE_API_KEY": "${BRAVE_API_KEY}"
      },
      "enabled": true
    }
  }
}
```

### Step 2: （可選）新增 Google Custom Search MCP 伺服器

若使用者取得 Google Custom Search API Key 與 CX，可追加：

```jsonc
"google-search": {
  "type": "local",
  "command": ["npx", "-y", "mcp-server-google-search"],
  "environment": {
    "GOOGLE_API_KEY": "${GOOGLE_API_KEY}",
    "GOOGLE_CX": "${GOOGLE_CX}"
  },
  "enabled": true
}
```

> ⚠️ Google Custom Search MCP 伺服器為社群維護，需確認套件名稱與版本。

### Step 3: 設定環境變數

使用者需在系統或 `.env` 中設定：

- `BRAVE_API_KEY` — 從 https://brave.com/search/api/ 取得（免費方案每月 2000 次）
- （可選）`GOOGLE_API_KEY` + `GOOGLE_CX` — 從 Google Cloud Console 取得

### Step 4: 驗證

- 重啟 Kilo 後，執行 `/mcps` 確認 brave-search 伺服器已載入。
- 測試搜尋功能，確認回傳結果正常。

## 風險與注意事項

- API Key 不應寫入版本控制，使用環境變數。
- Brave Search 免費方案有速率限制（每月 2000 次）。
- Google Custom Search 社群 MCP 伺服器穩定性不確定，建議優先使用 Brave Search。

## 開放問題

- 使用者是否已有 Brave API Key？若無，需引導註冊。
- 是否需要同時設定 Google Custom Search（需額外 API Key + CX）？
