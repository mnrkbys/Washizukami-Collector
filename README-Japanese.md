<div align="center">
 <p>
    <img alt="Washizukami Logo" src="Logo.png" width="60%">
 </p>
  [<a href="README.md">English</a>] | [<b>日本語</b>]
</div>

# Washizukami (鷲掴)

> **Windows 向けフォレンジック証拠収集ツール**

[![License: AGPL v3](https://img.shields.io/badge/License-AGPL_v3-blue.svg)](LICENSE)
[![Built with Rust](https://img.shields.io/badge/Built%20with-Rust-orange.svg)](https://www.rust-lang.org/)
[![Platform](https://img.shields.io/badge/Platform-Windows%2010%2F11%20x64-blue.svg)]()

---

## 概要

**Washizukami（鷲掴）** は、Rust で実装された Windows 向けのファストフォレンジック証拠収集ツールです。

OS がファイルをロックしている状況下でも、NTFS の Master File Table (MFT) を直接解析することで、レジストリハイブやイベントログなどのアーティファクトを取得できます。収集した証拠は SHA-256 ハッシュ付きの監査ログとともに保存されるため、そのまま各種解析ツールへの入力として利用できます。

このツールは、[CDIR-C](https://github.com/CyberDefenseInstitute/CDIR)（サイバーディフェンス研究所）に着想を得て開発しました。CDIR-C が切り拓いたライブシステムからのアーティファクト収集という手法を、Rust による実装でポータブルな単一バイナリとして提供することを目指しています。

**想定する解析ツールの例:**

- [Hayabusa](https://github.com/Yamato-Security/hayabusa) — Windows イベントログの脅威ハンティング
- [Velociraptor](https://github.com/Velocidex/velociraptor) / [KAPE](https://www.kroll.com/en/services/cyber-risk/incident-response-litigation-support/kroll-artifact-parser-extractor-kape) などのフォレンジックフレームワーク
- ELK Stack / Splunk などの SIEM への取り込み

### プロジェクトの方針

> **Keep it simple. Keep it reliable.**

Washizukami は多機能な DFIR プラットフォームを目指しません。Windows インシデントの初動における証拠収集という一点に集中し、解析はそのための専用ツールに委ねます。デフォルトの挙動をむやみに増やさず、緊迫した現場でも間違えずに使える範囲にコマンドラインを保ちます。

そうして抑えた複雑さは、もう一方の「信頼できること」に振り向けます。収集の確実性・再現性・保守性を優先し、デフォルトで収集するアーティファクトは継続的に検証し続けられるものだけに絞っています。定義数を意図的に抑えているのはそのためです。

---

## 機能

| 機能                     | 説明                                                                                                              |
| ------------------------ | ----------------------------------------------------------------------------------------------------------------- |
| **NTFS Raw Read**        | MFT を直接解析し、OS のファイルロックをバイパスして収集                                                           |
| **SHA-256 整合性検証**   | 収集したファイルをすべてハッシュ化し、改ざん検知を可能に                                                          |
| **監査ログ**             | タイムスタンプ・収集方法・SHA-256 を含む構造化ログ (`collection.log`)                                             |
| **単一バイナリ**         | Core Set の定義をコンパイル時に内蔵 — 実行時に外部ファイル不要                                                    |
| **柔軟なフィルタリング** | `--category` でカテゴリ単位のインクルード/エクスクルード（`!` プレフィックスで除外）、詳細は `config.yaml` で制御 |
| **ZIP 出力**             | 収集完了後にすべての成果物を単一 ZIP に圧縮して搬出を容易に                                                       |
| **メモリ取得連携**       | `--mem` オプションで [WinPmem](https://github.com/Velocidex/WinPmem) と連携してメモリダンプを取得                 |
| **Dry-Run モード**       | ファイルシステムに触れずに収集対象パスのみを確認                                                                  |
| **YARA スキャン**        | `scan` サブコマンドで永続化メカニズムを YARA-X でスキャン、検知ファイルを `infected.zip` に収集                   |
| **確認プロンプト**       | 収集・スキャン開始前に `[y/N]` で確認を求め、誤操作による意図しない収集を防止                                     |

---

## 動作環境

| 項目                     | 要件                                                                 |
| ------------------------ | -------------------------------------------------------------------- |
| **OS**                   | Windows 10 / Windows 11（x64）                                       |
| **権限**                 | **管理者権限**（Administrator）で実行すること                        |
| **ランタイム**           | 不要（静的ビルド済み — VC++ 再頒布可能パッケージ・MinGW DLL は不要） |
| **ディスク空き容量**     | 収集するアーティファクトの合計サイズ以上                             |
| **メモリ取得オプション** | `--mem` 使用時は `tools\` フォルダに `winpmem*.exe` を配置すること   |

> **注意:** NTFS Raw Read を使用するため、対象ボリュームが NTFS フォーマットであることが前提です。FAT32/exFAT ボリューム上のファイルは通常の File コレクタで収集されます。

---

## 使い方

### クイックスタート — ダブルクリックで使う

最も簡単な使い方は、エクスプローラーから **`washi.exe` をダブルクリック**するだけです。

設定は不要です。右クリック →「**管理者として実行**」するだけで、デフォルト設定で収集が始まります。

- **Core Set 全体**を収集（[収集対象アーティファクト](#収集対象アーティファクト)を参照）
- 出力先: **`HOSTNAME_ALL_YYYYMMDDHHMMSS\`**（`%AllNtfsDrives%` 利用時、`washi.exe` と同じ階層に生成）
- 監査ログ: **`HOSTNAME_ALL_YYYYMMDDHHMMSS\collection.log`**（SHA-256 ハッシュ付き）

収集完了後もコンソールウィンドウはそのまま開いた状態を維持するため、結果を確認してから **Enter** キーで閉じることができます。

カテゴリの絞り込み・出力先の指定・ZIP 生成など、より細かく制御したい場合は以下の CLI オプションをご利用ください。

---

### アーティファクト収集モード（デフォルト）

```
washi.exe [OPTIONS]

Options:
  -o, --output <DIR>               出力先ディレクトリ
                                   [デフォルト: <実行ファイルのフォルダ>\<COMPUTERNAME>_<SOURCE>_<YYYYMMDDHHMMSS>]
                                   デフォルト命名では全ドライブ時は SOURCE=ALL、
                                   従来の単一ドライブ時は SOURCE=C を使用。
  -c, --category <CATEGORY>        カテゴリでフィルタリング（複数指定可、大文字小文字不問）
                                   プレフィックスなし: 指定カテゴリのみ収集
                                   '!' プレフィックス: 指定カテゴリを除外
                                   指定可能値: EventLogs, Registry, NTFS, Filesystem, WMI, SRUM, Web
                                   config.yaml で読み込んだ Custom 定義は独自のカテゴリを追加します。
      --dry-run                    パス解決結果のみ表示（ファイルは収集しない）
      --zip                        収集完了後に ZIP アーカイブを生成
      --mem                        tools\winpmem*.exe でメモリダンプを取得（収集前に実行）
      --vss                        取得可能なすべての Volume Shadow Copy から収集しつつ、
                                   ライブボリュームも常に同時収集
      --volume <LETTER>            収集対象のドライブレターを上書き。
                                   USB メモリ（D: 等）から washi.exe を実行する場合でも
                                   デフォルトで C: からアーティファクトを収集する。
                                   --volume D を指定すると D: のアーティファクトを収集。
                                   NTFS アーティファクトが %AllNtfsDrives% を使用していても
                                   --volume 指定時は指定ドライブのみを収集。
  -v, --verbose                    カテゴリ単位の集計ではなく収集ファイルを1件ずつ表示
  -h, --help
  -V, --version
```

### YARA スキャンモード

```
washi.exe scan [OPTIONS] --rules <FILE> --output <DIR>

Options:
      --yara-path <PATH>           YARA-X エンジン（yr.exe）のパス [デフォルト: ./tools/yr.exe]
      --rules <FILE>               YARA ルールファイルのパス（必須）
      --output <DIR>               スキャン結果の出力先（必須）
  -h, --help
```

**前提:** [YARA-X](https://github.com/VirusTotal/yara-x)（`yr.exe`）を `washi.exe` と同じフォルダの `tools\` に配置するか、`--yara-path` でパスを指定してください。

#### 動作の流れ

スキャンモードは、Windows の**永続化メカニズム**に登録された実行ファイルパスを対象にします。永続化メカニズムとは、OS が再起動後も自動的にプログラムを起動するために使う仕組みであり、マルウェアが生き残るための隠れ場所として頻繁に悪用されます。

**Step 1 — スキャン対象の収集**

以下のソースを自動的に列挙し、実行ファイルパスを抽出します。

| ソース                                                | 内容                                                                        |
| ----------------------------------------------------- | --------------------------------------------------------------------------- |
| `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Run`  | ログイン時に**全ユーザー**向けに起動されるプログラム（システム共通）        |
| `HKCU\SOFTWARE\Microsoft\Windows\CurrentVersion\Run`  | ログイン時に**現在のユーザーのみ**起動されるプログラム                      |
| `C:\Windows\System32\Tasks`（タスクスケジューラ XML） | スケジュールされたタスク — 時刻・イベント・システム状態などをトリガーに起動 |

Run キーの値については、引数文字列を除去（例: `"C:\App\tool.exe" --silent` → `C:\App\tool.exe`）し、環境変数を展開（`%SystemRoot%` → `C:\Windows`）します。絶対パスを持たないエントリ（`sc.exe` のようなベアファイル名）はディスク上の場所を特定できないためスキップし、警告を表示します。

**Step 2 — YARA-X でスキャン**

収集したパスを一時リストファイルに書き出し、`--scan-list` で `yr.exe` に渡します。yr.exe は各ファイルの内容を YARA ルールと照合します。

**Step 3 — 検知ファイルのアーカイブ**

YARA ルールにマッチしたファイルは、出力先の `infected.zip` にコピーされます。監査ログ（`collection.log`）には、マッチしたルール名とファイルパスがすべて記録されます。

#### 実行例（出力イメージ）

```
·  Collecting persistence targets…
⚠  Not an absolute path: sc.exe              ← スキップ（絶対パスなし）
·  48 target(s) to scan
·  Running YARA scan…
⚠  C:\Users\Public\malware.exe  —  Detect_Mimikatz
────────────────────────────────────────────────────
⚠  Scan complete  ·  1 of 48 target(s) matched
   Archive  C:\scan_out\infected.zip
```

### 確認プロンプト

収集モード・スキャンモードともに、処理開始前に確認を求めます。

```
[?] Start collection? [y/N]:
```

`y` または `yes`（大文字小文字不問）を入力すると処理を開始します。それ以外の入力・Enter のみ・Ctrl+C はすべてアボートとして扱われます。`--dry-run` では確認は不要です。

### 実行例

```powershell
# 全アーティファクトを収集（監査ログ付き）
washi.exe

# 収集後に ZIP アーカイブを生成
washi.exe --zip

# メモリダンプ取得 → 全アーティファクト収集 → ZIP 生成
washi.exe --mem --zip

# Registry と EventLogs のみ収集
washi.exe --category Registry --category EventLogs

# EventLogs と WMI を除外してすべて収集
washi.exe --category '!EventLogs' --category '!WMI'

# 収集ファイルを1件ずつ表示（詳細モード）
washi.exe --verbose

# 取得可能な全 Volume Shadow Copy + ライブボリュームを同時収集
washi.exe --vss

# 収集対象の確認（ファイルは書き込まない）
washi.exe --dry-run

# 出力先を指定
washi.exe --output D:\evidence\case001 --zip

# 別ドライブ（フォレンジックターゲットが D: としてマウントされている場合など）からアーティファクトを収集
washi.exe --volume D --output E:\evidence\case001

# VSS 展開結果を dry-run で確認
washi.exe --vss --dry-run

# YARA スキャン（永続化パスをスキャンし、検知ファイルを infected.zip に収集）
washi.exe scan --rules C:\rules\malware.yar --output C:\scan_out
```

---

## 収集対象アーティファクト

アーティファクト定義は Core と Custom の2種類です。分類ルールの詳細は [`artifacts/README.md`](artifacts/README.md) を参照してください。

| | Core | Custom |
| --- | --- | --- |
| 配置 | `artifacts/core/` | `artifacts/custom/` |
| 配布 | `washi.exe` に内蔵 | 本リポジトリ内のサンプルファイル |
| 収集 | 設定不要でデフォルト収集 | `config.yaml` で読み込んだ場合のみ |
| 保守 | 継続的に検証・保守 | 現状のまま提供 |

### Core Set

デフォルトで収集されます。`washi.exe` をダブルクリックしたときに取得されるのがこの範囲です。

| カテゴリ       | アーティファクト                                                                    | 収集方式   |
| -------------- | ----------------------------------------------------------------------------------- | ---------- |
| **EventLogs**  | Security / System / Application Event Log                                           | NTFS       |
| **Registry**   | SAM / SECURITY / SOFTWARE / SYSTEM ハイブ（トランザクションログ .LOG1 / .LOG2 含む） | NTFS       |
| **Registry**   | Amcache.hve（.LOG1 / .LOG2 含む）                                                   | NTFS       |
| **Registry**   | NTUSER.DAT / UsrClass.dat（全ユーザー、.LOG1 / .LOG2・TxR ファイル含む）            | NTFS       |
| **NTFS**       | `$MFT`（Master File Table）                                                         | NTFS       |
| **NTFS**       | `$SECURE:$SDS`（セキュリティ記述子ストリーム）                                      | NTFS + ADS |
| **NTFS**       | `$UsnJrnl:$J`（USN ジャーナル）— スパース領域をスキップし、実アロケート部分のみ収集 | NTFS + ADS |
| **Filesystem** | プリフェッチファイル（`Prefetch\*.pf`）                                             | File       |
| **Filesystem** | 最近使ったファイル（`Recent\*.lnk`）                                                | File       |
| **WMI**        | WMI リポジトリ（OBJECTS.DATA / INDEX.BTR / MAPPING\*.MAP）                          | NTFS       |
| **SRUM**       | SRUM データベース（SRUDB.dat）                                                      | NTFS       |
| **Web**        | Chrome 履歴                                                                         | File       |
| **Web**        | Firefox 履歴・Cookie（places.sqlite / cookies.sqlite）                              | File       |
| **Web**        | IE / Edge WebCache（WebCacheV01.dat）                                               | File       |
| **Web**        | Edge 履歴                                                                           | File       |

> **NTFS + ADS:** Alternate Data Stream を MFT 直接読み取りで取得します。通常の API では読み出せないストリームにもアクセス可能です。

> **全 NTFS ドライブ対応:** Core の NTFS メタデータ定義は `%AllNtfsDrives%` を使用し、OS が認識している NTFS ドライブへ自動展開されます。非NTFSドライブは自動的にスキップされます。

### Custom 定義

デフォルトでは収集されません。`artifacts/custom/` 配下の各ファイルは `config.yaml` と同じ形式になっているため、`washi.exe` と同じフォルダにコピーして `config.yaml` にリネームするか、既存の `config.yaml` の `artifacts:` にマージして使用します。

| ファイル | カテゴリ | アーティファクト | Core にしない理由 |
| --- | --- | --- | --- |
| `custom/paging.yaml` | `Paging` | `pagefile.sys` / `swapfile.sys` / `hiberfil.sys` | 搭載 RAM に比例したサイズになり、合計 20〜30GB に達することが多い。価値は高いが、ファストトリアージのデフォルトとしては成立しない |
| `custom/browsers-extra.yaml` | `Web` | Brave / Vivaldi / Opera / Yandex の履歴 | Chromium 派生であり、意図的にインストールした環境にのみ存在する |
| `custom/ai-tools.yaml` | `AI Tools` | Claude Code / Codex CLI のローカルデータ | ディスク上のレイアウトが各ツール自身のリリースサイクルで変化する。またプロンプト履歴はプライバシー影響が大きい |
| `custom/outlook.yaml` | `Mail` | Classic Outlook の `.pst` ファイル | パスが Outlook のエディション・表示言語・OneDrive 設定によって変わる |

```powershell
# artifacts\custom\paging.yaml を washi.exe の隣に config.yaml としてコピーした後
washi.exe --dry-run --category Paging   # 先にサイズを確認
washi.exe --category Paging
```

> Custom 定義を読み込むまで、そのカテゴリは CLI にとって未知であり `--category` は拒否されます。

---

## 出力構造

```
<実行フォルダ>\
├── HOSTNAME_ALL_YYYYMMDDHHMMSS\  ← 出力フォルダ（washi.exe と同じ階層に生成）
│   ├── collection.log          ← 監査ログ（タイムスタンプ・SHA-256・収集方法）
│   ├── memory.dmp              ← メモリダンプ（--mem 指定時のみ）
│   ├── EventLogs\
│   │   ├── C\
│   │   │   ├── Windows\System32\winevt\Logs\Security.evtx
│   │   │   └── ...
│   ├── Registry\
│   │   ├── C\
│   │   │   ├── Windows\System32\config\SAM
│   │   │   └── ...
│   ├── NTFS\
│   │   ├── C\
│   │   │   ├── $MFT
│   │   │   ├── $Secure_SDS     ← $SECURE:$SDS ストリーム
│   │   │   └── $UsnJrnl_J      ← $UsnJrnl:$J ストリーム
│   │   └── D\
│   │       └── ...             ← 追加の NTFS ドライブがある場合
│   ├── Filesystem\
│   │   ├── C\
│   │   │   └── ...
│   │   └── D\
│   │       └── ...
│   ├── WMI\
│   │   └── C\
│   │       └── ...
│   ├── SRUM\
│   │   └── C\
│   │       └── Windows\System32\sru\SRUDB.dat
│   └── Web\
│        └── C\
│            └── ...
└── HOSTNAME_ALL_YYYYMMDDHHMMSS.zip ← ZIP アーカイブ（--zip 指定時のみ）
```

### 監査ログ形式

```
[2026-03-21T10:30:00+0900] [OK   ] [NTFS        ] C:\Windows\System32\config\SAM -> HOSTNAME_ALL_20260418120000\Registry\C\Windows\System32\config\SAM (262144 bytes, SHA256: abcd1234...)
[2026-03-21T10:30:01+0900] [SKIP ] [-           ] C:\path\missing — file not found
[2026-03-21T10:30:02+0900] [FAIL ] [-           ] C:\path\locked — <error>
[2026-03-21T10:30:03+0900] [TOOL ] [winpmem_x64 ] Starting: tools\winpmem_x64.exe -> HOSTNAME_ALL_20260418120000\memory.dmp
[2026-03-21T10:30:10+0900] [INFO ] [-           ] Complete — OK: 141  Skipped: 1  Failed: 0

# washi.exe scan 実行時
[2026-03-23T11:00:00+0900] [SCAN ] [yr          ] Starting scan — engine: ./tools/yr.exe  rules: malware.yar  targets: 59
[2026-03-23T11:00:02+0900] [MATCH] [yara        ] C:\Windows\System32\notepad.exe — test_notepad
[2026-03-23T11:00:02+0900] [SCAN ] [-           ] Complete — matched: 1  archive: scan_out\infected.zip
```

---

## アーティファクト定義のカスタマイズ

Core Set は Windows イベントログ・レジストリハイブ・一般的なファイルシステムアーティファクトをカバーしています。`washi.exe` と同じフォルダに `config.yaml` を配置することで、収集対象の絞り込みや Custom アーティファクトの追加ができます。

**優先順位:** CLI フラグ > `config.yaml` > Core Set（内蔵）

### フィルタ

#### `enabled_artifacts`

収集するアーティファクト名のホワイトリストです。空または省略した場合はすべて収集されます。大文字小文字は区別しません。

<details>
<summary>Core Set アーティファクト名一覧</summary>

| カテゴリ   | 名前                           |
| ---------- | ------------------------------ |
| EventLogs  | `Security Event Log`           |
| EventLogs  | `System Event Log`             |
| EventLogs  | `Application Event Log`        |
| Registry   | `SAM Registry Hive`            |
| Registry   | `SECURITY Registry Hive`       |
| Registry   | `SOFTWARE Registry Hive`       |
| Registry   | `SYSTEM Registry Hive`         |
| Registry   | `Amcache.hve`                  |
| Registry   | `User NTUSER.DAT`              |
| Registry   | `User UsrClass.dat`            |
| NTFS       | `$MFT`                         |
| NTFS       | `$SECURE:$SDS`                 |
| NTFS       | `$UsnJrnl:$J`                  |
| Filesystem | `Prefetch Files`               |
| Filesystem | `Recent LNK Files`             |
| WMI        | `WMI Repository OBJECTS.DATA`  |
| WMI        | `WMI Repository INDEX.BTR`     |
| WMI        | `WMI Repository MAPPING Files` |
| SRUM       | `SRUM Database`                |
| Web        | `Chrome History`               |
| Web        | `Firefox places.sqlite`        |
| Web        | `Firefox cookies.sqlite`       |
| Web        | `IE/Edge WebCacheV01.dat`      |
| Web        | `Edge History`                 |

</details>

#### `disabled_categories`

カテゴリ単位で除外します。有効な値: `EventLogs` / `Registry` / `NTFS` / `Filesystem` / `WMI` / `SRUM` / `Web`（大文字小文字不問）。同じ `config.yaml` 内の Custom 定義が追加したカテゴリも指定できます。

> **注意:** `disabled_categories` は `enabled_artifacts` より**後に**評価されます。ホワイトリストに明示したアーティファクトでも、カテゴリが無効化されていれば除外されます。

### カスタムアーティファクト定義

`artifacts` キーで Core Set にないアーティファクトを追加できます。Core と同じ `name` を指定した場合はカスタム定義が優先されます。

必須フィールド:

| フィールド    | 説明                                                                               |
| ------------- | ---------------------------------------------------------------------------------- |
| `name`        | 一意な表示名。`config.yaml` の `enabled_artifacts` から参照されます。              |
| `category`    | グループ名。出力サブフォルダ名にも使用されます。                                   |
| `target_path` | 収集対象パス。`%VAR%` 形式の環境変数とグロブワイルドカード（`*`・`?`）が使用可能。 |
| `method`      | `File`（通常の OS コピー）または `NTFS`（MFT 直接読み取り、ファイルロック回避）。  |

### `config.yaml` の記述例

```yaml
# ── フィルタ ──────────────────────────────────────────────────────────────────
# 以下のアーティファクトのみ収集（コメントアウトすると全件収集）
enabled_artifacts:
  - "SAM Registry Hive"
  - "Security Event Log"
  - "System Event Log"

# カテゴリ単位で除外
disabled_categories:
  - Filesystem
  - Web

# ── カスタムアーティファクト定義 ──────────────────────────────────────────────
artifacts:
  - name: "My Application Log"
    category: "Custom"
    target_path: "C:\\MyApp\\logs\\app.log"
    method: File

  - name: "My Locked DB"
    category: "Custom"
    target_path: "%SystemDrive%\\MyApp\\data\\app.db"
    method: NTFS

  - name: "All XML Configs"
    category: "Custom"
    target_path: "C:\\MyApp\\config\\*.xml"
    method: File
```

### Custom 定義の読み込み

[`artifacts/custom/`](artifacts/custom/) 配下の各ファイルは `config.yaml` と同じ形式になっています。使用するには、`washi.exe` と同じフォルダにコピーして `config.yaml` にリネームするか、既存の `config.yaml` の `artifacts:` にマージします。実際の収集前に必ず `--dry-run` で対象を確認してください。

#### `custom/paging.yaml` — ページング／休止状態ファイル

`pagefile.sys` と `hiberfil.sys` は搭載 RAM に比例したサイズになるため、16GB のホストでは合計 20〜30GB に達することが一般的です。プロセスメモリの断片（認証情報、復号済みペイロード、ネットワークバッファ）が残存している可能性があります。

> **サイズに注意:** 必ず `--dry-run --category Paging` を先に実行してください。RAM 搭載量の多いホストでは、この3ファイルだけで収集先の空き容量を超えることがあります。
>
> `hiberfil.sys` は休止状態が無効（`powercfg /h off`）の場合、`swapfile.sys` はストアアプリを一度も起動していない場合には存在しません。

#### `custom/browsers-extra.yaml` — Chromium 派生ブラウザ

Brave / Vivaldi / Opera / Yandex。これらは `category: Web` を使用するため、Core のブラウザアーティファクトと一緒に収集され、同じ `Web\` 出力ツリーに配置されます。各 `History` は SQLite データベースで、Chrome と同じツールで解析できます。

#### `custom/ai-tools.yaml` — Claude Code / Codex CLI

[Claude Code](https://claude.ai/code) は `%USERPROFILE%\.claude\` 配下に、[Codex CLI](https://github.com/openai/codex) は `%USERPROFILE%\.codex\` 配下にローカルデータを保存します。

| パス | 内容 |
| ---- | ---- |
| `.claude\history.jsonl` | ユーザーが入力したすべてのプロンプト履歴 |
| `.claude\paste-cache\*` | `history.jsonl` を肥大化させないよう外部保存された大容量テキスト貼り付け |
| `.claude\image-cache\*` | 大容量画像貼り付け（paste-cache と同じ理由） |
| `.claude\file-history\*` | Claude Code がファイルを編集する前の事前スナップショット |
| `.codex\history.jsonl` | プロンプト履歴（プロンプト本文・セッション ID・タイムスタンプ） |
| `.codex\sessions\**\rollout-*.jsonl` | セッション単位の詳細イベントログ（セッションメタデータ、ユーザー／エージェントメッセージ、reasoning、ツール呼び出しと実行結果、タスク状態、トークン使用量、コンテキスト圧縮、エラー情報） |
| `.codex\attachments\**\*` | Codex CLI セッションで使用された添付ファイル |

すべて `method: NTFS` を使用します。収集時にこれらのツールが動作しファイルをロックしている可能性があるためです。特に Codex CLI はセッション中ロールアウトログに継続的に追記します。

> **`**` を使う理由:** Codex はロールアウトログを日付ツリー（`sessions\YYYY\MM\DD\`）に書き出します。単一階層の `*` を固定個数並べた指定では取りこぼしが発生するため、再帰展開する `**` が必要です。なお、末尾が単独の `**` の場合はディレクトリのみに一致するため、配下のファイルをすべて収集するには `**\*` と記述します。
>
> **収集対象外:** 認証情報（`auth.json`）、設定（`config.toml`）、環境ファイル、shell snapshots、SQLite データベース、内部キャッシュは、これらの定義には意図的に含めていません。
>
> **`image-cache` について:** Claude Code で画像を貼り付けたことがない場合、`image-cache` ディレクトリは存在しないため dry-run では `⚠ NO MATCH` と表示されます。これは正常な動作です。

#### `custom/outlook.yaml` — Classic 版 Outlook `.pst` ファイル

Classic 版 Outlook（新しい Outlook アプリではなく旧来の Outlook）の `.pst` ファイルは、Outlook のエディション・Windows の表示言語・OneDrive 設定によって保存場所が異なります。同梱の定義は**日本語 Windows かつ OneDrive 連携が有効な場合**を対象としています。

```
C:\Users\<ユーザー名>\OneDrive\ドキュメント\Outlook ファイル\*.pst
```

他のレイアウトでは `target_path` を調整し、`--dry-run` で確認してから使用してください。

> **`method: NTFS` を使う理由:** Classic 版 Outlook は起動中に `.pst` ファイルを排他ロックします。NTFS Raw Read を使うことでロックをバイパスし、Outlook を終了させることなく収集できます。
>
> **サイズに注意:** `.pst` ファイルは数 GB になる場合があります。収集前に `--dry-run` でファイルサイズを確認することをお勧めします。

---

## メモリ取得（winpmem 連携）

`--mem` オプションを使用すると、アーティファクト収集の前に [WinPmem](https://github.com/Velocidex/WinPmem) でメモリダンプを取得できます。

1. [WinPmem リリースページ](https://github.com/Velocidex/WinPmem/releases) から `winpmem_x64.exe` をダウンロード
2. `washi.exe` と同じフォルダの `tools\` に配置
3. `--mem` を付けて実行

```
（配置例）
washi.exe
tools\
└── winpmem_x64.exe
```

> `tools\winpmem*.exe` が見つからない場合は警告をログに記録し、アーティファクト収集のみ続行します。

---

## ソースからのビルド

**必要なもの:**

- Rust stable ツールチェーン（`x86_64-pc-windows-gnu`）
- MSYS2 + MinGW-w64（GNU リンカ）

```powershell
git clone https://github.com/tadmaddad/Washizukami-Collector.git
cd Washizukami-Collector
cargo build --release
```

---

## ロードマップ

現在計画中・検討中の機能拡張です。実装順は未定です。

### YARA スキャンの拡張

`scan` サブコマンドは v0.4.0 で実装済みです。今後の拡張として以下を検討しています。

- `--target` オプションによる任意ディレクトリの追加スキャン
- `infected.zip` へのパスワード保護（AES-256）— 現在はビルド環境の制約により未実装
- スキャン対象の拡張（スタートアップフォルダ、サービス登録パスなど）

### メールクライアントアーティファクト

#### Microsoft Outlook `.pst` — Custom 定義として対応済み

Classic 版 Outlook の `.pst` 収集は [`artifacts/custom/outlook.yaml`](artifacts/custom/outlook.yaml) で現在のバージョンでも対応できます。詳細は「[`custom/outlook.yaml` — Classic 版 Outlook `.pst` ファイル](#customoutlookyaml--classic-版-outlook-pst-ファイル)」を参照してください。

#### Custom 定義の追加（予定）

以下は今後の Custom 定義追加として検討中です。

| クライアント            | 対象ファイル                                                  |
| ----------------------- | ------------------------------------------------------------- |
| **Microsoft Outlook**   | `.ost` ファイル、添付ファイルキャッシュ                       |
| **Mozilla Thunderbird** | メールボックス（`*.msf` / `INBOX`）、アドレス帳、設定ファイル |

メールデータはサイズが大きくなりがちなため、収集対象期間の絞り込みや差分収集などの最適化も合わせて検討しています。

---

## バグ修正

### v0.6.1 — `$UsnJrnl:$J` スパースファイルの過大収集

**症状:** 収集した `$UsnJrnl:$J` のサイズが論理ファイルサイズ（10 GB 超）になる。

**原因:** `$UsnJrnl:$J` はスパースファイル。Windows は USN ジャーナルを循環バッファとして管理しており、古いエントリはスパースとして解放される。先頭のほぼ全域がスパース（仮想ゼロ・ディスク上に実体なし）で、実際の USN レコードは末尾の数十 MB のみ。修正前の実装は論理ストリーム全体を読み込んでいたが、ntfs クレートはスパースランをゼロ埋めして返すため、論理サイズ分がそのまま書き出されていた。

**修正:** `NtfsNonResidentAttributeValue::data_runs()` を直接イテレートするよう `copy_data()` を書き直した。スパースラン（`data_position()` が `None`）はゼロを書かずスキップし、実アロケートランのみを出力する。

---

## 名前の由来：なぜ「鷲掴（Washizukami）」なのか？

本ツールの名称は、Windows ログ解析のデファクトスタンダードであり、多くのセキュリティエンジニアが愛用する **[Hayabusa](https://github.com/Yamato-Security/hayabusa)** への深いリスペクトから命名されました。

空の王者であるハヤブサが鋭い眼光で獲物を見つけ出すなら、このツールはその獲物（アーティファクト）を物理的に「鷲掴み」にして、OS の制限（ファイルロック）さえもねじ伏せて持ち帰る。そんな力強い証拠収集へのこだわりを込めています。

…と、ここまでが公式の（真面目な）説明です。

たまに「作者の個人的な嗜好が反映されているのでは？」という邪推をいただくことがありますが、断じて違います。私はただ、NTFS の MFT とレジストリハイブを、法的に正しい手続きで、優しく、かつ力強くホールドしたいだけなのです。

---

## AI-Assisted Development（AI による開発支援）

本プロジェクトの設計判断・レビュー・最終的な意思決定は、いずれもメンテナが行っています。AI アシスタントは開発支援のツールとして利用しており、本セクションはその透明性のための記載です。

- **Claude Code** — リポジトリ上の実装作業。Rust コードの変更、リファクタリング、テスト、Git / GitHub の操作。
- **ChatGPT** — ロードマップの検討、アーキテクチャ・設計の議論、批判的レビュー、タスク分解、リリース計画、設計方針とプロジェクトの方向性の整理。

すべての変更は、取り込む前にメンテナがレビューし承認しています。

---

## ライセンス

Copyright (C) 2026 tadmaddad - Jawfish Lab

本ソフトウェアは、GNU Affero General Public License v3.0（AGPL-3.0）に基づき、オープンソースとして公開されています。

---

## 利用ライブラリ・ツール

- [ntfs](https://github.com/ColinFinck/ntfs) by Colin Finck — MFT 直接アクセスを可能にする Pure Rust NTFS パーサ
- [WinPmem](https://github.com/Velocidex/WinPmem) by Velocidex — Windows メモリ取得ツール
