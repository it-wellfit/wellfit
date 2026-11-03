# Kilo Gateway 429 診斷與 Wellfit 作業邊界

**版本：** 1.0  
**查核日期：** 2026-11-02  
**狀態：** 操作指引草案；以 Kilo 官方公開文件為查核依據

## 1. 先確認影響範圍

Wellfit 七個 crate 沒有 Kilo API client、Gateway endpoint 或模型呼叫；資料匯入、驗證、對帳與報告產生由本機確定性程式執行。因此 Kilo 回傳 HTTP 429 不應中斷、重試或延遲 Wellfit 的批次資料管線。本機改設定、重裝、切換本機 binary 或增加並行請求，不能解除服務端配額。

如果 429 發生在 Kilo IDE／CLI 或其他外部工作工具，應把它當成外部服務限制來診斷，不把它當作 Wellfit 程式失敗。

## 2. 官方資料能支持的結論

截至查核日，Kilo 官方文件指出：

- 免費模型有速率限制，供應的模型與可用性會變動；部分模型還可能受上游 provider 的額外限制。
- 官方文件所列每小時 200 次／IP 的明確數字，限定於**匿名使用者**；不能將此數字推廣為已登入帳戶、所有模型或所有 provider 的共同配額。
- 官方文件沒有在本指引查核的頁面中公布可普遍套用於所有已登入使用者的固定請求配額、延遲保證或可用性承諾。帳戶、組織、模型及 provider 的實際限制需以當下官方產品資訊與帳戶回應為準。
- `kilo-auto/free` 可能路由至會記錄 prompt／output 並用於改善服務的 provider；官方特別提醒不要傳送個人或機密資料，部分路由也可能使用 NVIDIA 免費端點。

來源：

1. [Kilo Gateway — Models & Providers](https://kilo.ai/docs/gateway/models-and-providers)；本次直接讀取。內容包含免費模型限流、匿名使用者 200 requests/hour/IP、上游 provider 限制，以及 Auto Free 資料處理警示。
2. [Using Kilo for Free](https://kilo.ai/docs/getting-started/using-kilo-for-free)；本次直接讀取。內容說明免費模型可用性變動、上游 rate limit 與 Auto Free 資料處理警示。
3. [IETF RFC 6585 §4 — 429 Too Many Requests](https://www.rfc-editor.org/rfc/rfc6585#section-4)；HTTP 429 表示一段時間內請求過多，回應可以包含 `Retry-After`；該 header 非必備。缺少時停止自動重試是本文件的保守作業建議。

以上是指定頁面於查核日可見的說明，不代表所有帳戶的配額清單，也不保證限制或模型目錄之後不變；每次排查需重新檢查官方即時目錄及帳戶介面。

## 3. 429 人工作業流程

1. **停止連續重試。**不使用緊密迴圈、無上限重試、帳號／IP 輪替或並行請求測試來試探上限。若回應有 `Retry-After`，只按該回應所示時間等待；沒有時先停止，人工確認限制來源後再做一次有界重試。
2. **記錄最少必要診斷資料。**保存 UTC 時間、HTTP 狀態、請求 endpoint 類型、模型 ID、登入／匿名狀態、可見的 rate-limit／`Retry-After` 標頭，以及 Kilo 或 provider 的 request ID。記錄使用的 Kilo extension／CLI 版本與操作步驟。
3. **區分限制來源。**從官方帳戶／服務狀態資訊確認 Gateway、帳戶／組織方案、特定模型或上游 provider；僅依錯誤內容、標頭、request ID 和官方支援回覆作判斷。資訊不足時登記為 `Unknown`，不要推定某個來源已被證明。
4. **安全提交支援請求。**只提交經核准的 request ID、狀態、時間、模型和必要的去識別摘要；刪除 `Authorization`、API key、token、cookie、session、個資、完整 prompt／completion 及含機密的 log。
5. **確認恢復。**恢復前確認等待期已過、服務／帳戶允許，以及所選模型的資料處理條件符合組織政策。不要為了避開限制而切換匿名入口、輪換身份／IP 或重送敏感內容。

## 4. Wellfit 與 AI 的隔離規則

- 批次處理、金額／數量計算、隔離、SQLite 帳本及驗收不得依賴 Kilo、免費模型或 LLM 輸出。
- 未來若另行加入 AI，只能作低頻、非權威的例外說明或審查輔助；須另立需求、資料分類及供應商風險審查、人工覆核與失敗回退，且 AI 不得自動決定法規適用、財務結果、風險接受或批次發布。
- 免費模型／Gateway 無法作為每日百萬次請求、延遲、可用性或容量的保證；不得以本地調整宣稱增加其服務端額度。
