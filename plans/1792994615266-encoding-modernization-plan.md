# Encoding & Data Format Modernization Plan

> **Disposition (2026-11-02): Historical proposal; baseline partly implemented and partly unverified.** The root `.editorconfig`, project `.gitattributes`, and `scripts/encoding-tools.ps1` now exist, so the adjacent UTF-8 plan's “missing” inventory is stale. The advanced secure-envelope and format-expansion proposals below are not approved requirements. See [`../PLAN.md`](../PLAN.md); validate the current configuration separately before claiming the standard is enforced.

Goal: 讓工作區內所有檔案與資料交換，能在「文字編碼（UTF-8 繁簡體無亂碼）」與「二進位資料交換（緊湊、可壓縮、可加密、可驗證）」之間無缺損轉換，並支援先進格式（Base91/Z85/basE91、CBOR、Parquet/Arrow、聯合加密壓縮）。

## 現狀（已完成）
- `.editorconfig`：UTF-8 / LF，涵蓋 md/rs/json/toml/yml/ps1/sh/html/css/ts/js/py/go/java/c/cpp/h/cs/rb/php/sql/xml/svg/txt/ini/cfg/conf/env，Binary：png/jpg/pdf/zip/exe/dll/lock/base64/hex/bin/dat/oct/bcd
- `ekpi-platform/.gitattributes`：`* text=auto encoding=utf-8` + 二進位規則
- `Wellfit/.gitattributes`：`* text=auto eol=lf encoding=utf-8` + 二進位規則
- `docs/encoding-standard.md`：編碼標準文件
- `scripts/encoding-tools.ps1`：detect/validate/convert/base64/hex，已修正中文路徑與 mojibake 判斷
- 驗證通過：GEMINI.md、Wellfit/AGENTS.md、ekpi-platform/AGENTS.md、WORKSPACE_CHARTER.md 均為 UTF-8 無 mojibake

## 待執行（先進編碼與資料格式整合）

### 1. 二進位轉文字編碼擴充（超越 Base64）
在 `scripts/encoding-tools.ps1` 新增指令：
- `base91-encode <path> [-OutputPath]` — 以 basE91 編碼，擴展約 1.23x（比 Base64 1.33x 更緊湊）
- `base91-decode <path> [-OutputPath]`
- `z85-encode <path> [-OutputPath]` — ZeroMQ Z85，擴展 1.25x，字串安全（可用於 JSON/SQL/原始碼）
- `z85-decode <path> [-OutputPath]`
- `base122-encode / base122-decode` — 以 UTF-8 輸出，擴展約 1.15x，適合可控編解碼器的場景

在 `.editorconfig` 新增副檔名規則：
- `*.base91` binary（不正規化編碼）
- `*.z85` binary
- `*.base122` binary
- `*.tave` binary（Tavra 加密封套）

### 2. 統一安全資料封套格式（CBOR + 壓縮 + 加密 + 完整性）
建立 `scripts/secure-pack.ps1`，參考 Tavra (.tave) 與 RustBinary 設計，實作單一封套格式 `.sek`（Secure Encapsulated Kit）：
- 頭部：Magic (`\x53\x45\x4B\x01`)、版本、旗標（壓縮/加密/簽章）、演算法 ID
- 載體：CBOR 序列化（RFC 8949，Deterministic Encoding）
- 壓縮：Zstandard（Zstd），僅在輸出更小時保留
- 加密：XChaCha20-Poly1305（AEAD），隨機 nonce
- 完整性：Ed25519 簽章（選用）或 SHA-256 摘要
- 支援多收件人（每收件人一個 LOCK 區塊，參考 IETF draft-sullivan-safe）

指令：
- `pack <input> <output.sek> [-Password | -PublicKey] [-Compress] [-Sign]`
- `unpack <input.sek> <output> [-Password | -PrivateKey] [-Verify]`

### 3. 大型資料與分析格式支援
在 `.editorconfig` 與 `.gitattributes` 新增：
- `*.parquet` binary
- `*.arrow` binary
- `*.avro` binary
- `*.zarr` binary（目錄結構，以 `.zarr` 為標記）
- `*.lance` binary
- `*.safetensors` binary
- `*.gguf` binary
- `*.onnx` binary

### 4. 編碼工具驗證與 CI 整合
- `scripts/encoding-tools.ps1 batch-validate <path>` 遞迴驗證所有文字檔為 UTF-8 且無 mojibake
- 新增 `scripts/encoding-tools.ps1 scan <path>`：偵測所有檔案編碼，列出非 UTF-8 文字檔與二進位檔
- 將驗證納入 `scripts/check.ps1`（ekpi-platform）與 Wellfit 本機檢查入口（若適用）

## 驗收標準
1. `batch-validate` 對工作區所有文字檔 exit code 0，無 mojibake
2. `detect` 可正確辨識 UTF-8 / Big5 / GBK / Shift-JIS / ISO-8859-1
3. `convert -FromEncoding big5 -ToEncoding utf-8 -NoBom -Force` 可將舊中文檔轉為 UTF-8 無 BOM
4. `base91-encode` / `base91-decode` 往返不損失
5. `z85-encode` / `z85-decode` 往返不損失，且輸出為字串安全 ASCII
6. `pack` / `unpack` 在有/無密碼、有/無壓縮、有/無簽章下均可正確往返
7. `.editorconfig` 與兩個 `.gitattributes` 對新增副檔名規則一致

## 風險與限制
- 不存在單一格式可「一次解決所有編碼與資料格式問題」。UTF-8 是字元編碼頂標；CBOR/Parquet 等解決結構化與大資料；Z85/Base91 只解決二進位轉文字；加密封套解決安全與完整性。必須分層使用。
- Base91/basE91 非 IETF/RFC 標準，相容性不如 Base64，僅建議在受控管線內使用。
- Z85 要求輸入長度為 4 位元組區塊，需處理對齊與填充。
- 聯合壓縮+加密（CompCrypt/ANS）仍屬研究階段，不納入首要實作，僅作未來觀察。

## 相關參考（已搜尋）
- basE91：1.23x 擴展，高效但非通用標準（base91.sourceforge.net / douzebis/base91）
- Z85：ZeroMQ spec:32，1.25x 擴展，字串安全（rfc.zeromq.org/spec/32）
- CBOR：RFC 8949，二進位序列化，支援 Deterministic Encoding（draft-rundgren-cbor-core）
- Tavra：.tav/.tavb/.tave，文字+二進位+加密封套（GitHub SkuldNorniern/tavra）
- SAFE：IETF draft-sullivan-safe，多收件人加密封套
- Gordian Envelope：IETF draft-mcnally-envelope，CBOR-based 隱私導向結構化資料
- DARE：IETF draft-hallambaker-mesh-dare，封存加密封套
- CompCrypt：ANS-based 聯合壓縮加密（IEEE TIFS 2021）
