# 計畫：REA 為核心的完整免費開源 Windows 本機逆向工具鏈整合

## 目標與約束（來自使用者三個選擇）

- 逆向對象：全類型（原生 EXE/DLL/ELF、Android APK、Web/JS/API、韌體、原始碼與建置產物）
- 能力：全部（AI Agent 自動化、靜態/動態分析、反編譯、功能還原、修補、差異、漏洞稽核、驗證）
- 部署與成本：免費、開源、Windows 本機優先；可搭配付費商業工具但不強制
- 核心定位：REA（morluto/rea）作為 AI Agent × MCP 協調層，不只貼反編譯結果到聊天視窗，而是讓 Agent 呼叫專業工具並攜帶證據

## 已確認的工具現狀（2026-10）

| 層級 | 工具 | 授權 | 狀態 | 備註 |
|------|------|------|------|------|
| Agent/MCP 協調 | morluto/rea | MIT | 2026-04 創立，2,609+ commits，npm `@morluto/rea`，Node.js 22+ | 42 MCP 工具；不自行安裝/更新 Ghidra/IDA/Hopper；需自備分析器；執行時擷取以使用者權限執行 |
| 靜態分析（主） | Ghidra + bethington/ghidra-mcp | Apache-2.0 | Ghidra 12+，JDK 21+，209 MCP 工具 | P-code 模擬、即時偵錯整合、批次操作；推薦社群版 |
| 靜態分析（備） | Rizin + Cutter | LGPL-3.0 / GPLv3 | Windows 7+，原生二進位 | 完全 FOSS 替代方案，Cutter 有 GUI 與 Ghidra 反編譯整合 |
| 商業靜態 | IDA 9.4 + HexRaysSA/ida-mcp | 商業 + MIT MCP | 需 IDA 授權 | 官方 MCP，Headless idalib，stdio/HTTP；非免費，僅做選擇性搭配 |
| 動態插樁 | Frida 17.24.3 | 各項套件不同 | 2026-10-19 釋出，極活躍 | Windows/macOS/Linux/Android/iOS/QNX；Python 繫結 |
| Android | JADX (skylot/jadx) + zinja-coder/jadx-ai-mcp | Apache-2.0 | APK/DEX 反編譯 | JADX-AI-MCP 外掛把 Android 分析接入 MCP/Agent |
| 差異比較 | ghidriff + ghidriff-mcp | - | Ghidra 二進位差異 | 與 REA/Ghidra MCP 搭配做版本差異與修補分析 |
| 驗證層 | 隔離沙箱/VM、雜湊紀錄、測試案例、人工核准、報告 | - | 需自行建置 | AI 推論不等於已驗證行為 |

## 重大決策與風險（必須先處理）

1. **Node.js 與專案「已移除 Node.js」的衝突**：記憶中專案決策為「Node.js 完全移除，統一 Rust/Bun/HTTP」。但 REA 是 npm 套件，需 Node.js 22+（`npx rea-agents setup`）。這是整合計畫最大技術衝突。
   - 建議決策：為 REA 保留最小 Node.js 執行環境（僅供 REA MCP 使用），其餘工具框架（Windmill 等）維持 Rust/Bun/HTTP；或以 Bun 執行 REA（Bun 相容 npm）。此決策需使用者確認，列為未解決開放問題。

2. **IDA 是商業軟體**：與「免費開源」約束衝突。IDA MCP 本身是 MIT 且免費，但需 IDA Pro 授權。計畫中 IDA 僅作為「選擇性搭配/商業升級路徑」，主路徑用 Ghidra + Rizin/Cutter。

3. **安全性**：REA 執行時擷取以目前使用者權限執行目標程式。必須在隔離環境（VM/沙箱/容器）中執行，並加入權限控制與人工核准門檻。

4. **可重現性**：需保留原始檔雜湊、工具版本、輸入、操作紀錄與驗證結果，建立獨立驗證與證據層。

## 分層工具鏈架構

```
AI Agent 工作層：REA (MCP server) <-> Claude Code / Codex / 代理
    ↓ 呼叫 MCP 工具
靜態分析：Ghidra (ghidra-mcp) / Rizin + Cutter / (IDA 選擇性)
動態分析：Frida (插樁/追蹤/除錯)
平台專用：JADX + jadx-ai-mcp (Android)；Binwalk/unblob (韌體/封裝)
差異比較：ghidriff-mcp (版本差異/修補分析)
獨立驗證與證據層：隔離執行 / 雜湊與版本紀錄 / 測試案例 / 可重現分析 / 人工核准 / 完整報告
```

## 實作步驟（有序任務清單）

1. **環境準備（Windows 本機）**
   - 安裝最小 Node.js 執行環境（僅供 REA），或確認 Bun 可執行 `npx rea-agents`。
   - 安裝 Ghidra 12+ (JDK 21+) 與 Cutter/Rizin 作為 FOSS 靜態分析主力。
   - 安裝 Frida (pip install frida frida-tools) 與對應 frida-server。
   - 安裝 JADX 與 JADX-AI-MCP 外掛。
   - 準備隔離 VM/沙箱（建議 Windows Sandbox / Hyper-V VM / Docker 容器）用於動態執行目標。

2. **REA 核心佈署**
   - `npx rea-agents setup` 驗證安裝。
   - 連結 Ghidra MCP 與 Frida 到 REA 的 MCP 目錄（參考 `docs/mcp-contracts.md`）。
   - 設定 Agent（Claude Code / Codex）啟用 REA MCP Server。

3. **靜態分析整合**
   - 啟動 bethington/ghidra-mcp（或 Konamiu/Maleick 分支），驗證 42–200+ MCP 工具可被 Agent 呼叫。
   - 以 Ghidra/Cutter 對樣本 EXE/DLL/ELF 做反組譯、反編譯、函式交叉參照，輸出為 MCP 工具回傳。

4. **動態分析整合**
   - 在隔離 VM 中啟動 frida-server，從 Windows 主機透過 Frida Python 繫結做函式追蹤與插樁。
   - 將 Frida 觀察結果作為證據，驗證靜態分析假設。

5. **Android 與平台專用技能**
   - JADX 反編譯 APK/DEX；啟用 jadx-ai-mcp 讓 Agent 即時查詢 decompiled context。
   - 韌體/封裝：Binwalk / unblob 做檔案識別與提取。

6. **差異與行為比較**
   - ghidriff-mcp 驅動 Ghidra 做舊/新建置差異，回傳 match 統計與函式變更。
   - 搭配執行紀錄做行為比對（前後版本差異分析）。

7. **獨立驗證與證據層（關鍵品質門檻）**
   - 對每個目標檔計算 SHA-256 雜湊並記錄工具版本。
   - 將每個 Agent 結論與原始證據（Ghidra 反編譯輸出、Frida 追蹤日誌、JADX 原始碼）綁定。
   - 引入人工核准門檻：AI 推論不直接當成已驗證行為，需人工覆核並簽核。
   - 產出可重現分析報告（輸入、步驟、證据、信心、待確認事項、獨立測試結果）。

8. **驗證計畫**
   - 以已知樣本（如 crackme / DX-Ball 類可測試目標）驗證 REA → Ghidra → Frida 端到端流程。
   - 驗證：反編譯函式位元組與原始二進位位元組一致；測試案例覆蓋 3205 個 x86 案例（參考 REA showcase）。
   - 驗證隔離：確認動態執行不脫離沙箱、未以主機使用者權限寫入版本控制棧出。

## 驗收標準（Definition of Done）

- REA MCP 可在 Windows 本機成功連接 Claude Code / Codex 並呼叫 Ghidra 與 Frida 工具。
- 能對一個未知 EXE/DLL 完成「靜態反編譯 → 動態追蹤驗證 → 撰寫等效程式碼 → 測試合比」的完整循環。
- 每個重要結論皆附帶原始證據、分析依據、信心等級與待確認事項。
- 動態分析全部在隔離環境中執行，未污染主機與版本控制棧。
- 報告可重現：相同輸入與工具版本可產出一致分析結果。

## 未解決開放問題（需使用者確認後再執行）

1. Node.js 保留與「已移除 Node.js」專案決策如何協調？（建議：REA 單獨保留最小 Node.js/Bun 執行環境）
2. 是否允許在隔離 VM 中使用商業 IDA 作為選擇性升級路徑？
3. 隔離環境採用 Windows Sandbox 還是 Hyper-V VM / Docker？
4. Agent 端使用 Claude Code 還是 Codex？（兩者皆支援 MCP）

---

此計畫可直接交給具實作權限的代理執行。執行時請依「獨立驗證與證據層」的品質門檻，確保 AI 結論皆有可追溯證據。