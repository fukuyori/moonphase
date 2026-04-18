# moon

[English README](README.md)

月齢、月相、日の出・日の入り、月の出・月の入りを表示するコマンドラインツールです。必要に応じて、墨絵風のアスキーアートも表示できます。

現在のバージョン: `0.9.6`

あわせて: [CHANGELOG](CHANGELOG.md)

## 特長

- 選択したタイムゾーンの `0:00` 基準で月齢、照明率、月距離を計算
- 指定地点の `日の出 / 日の入り / 月の出 / 月の入り` を表示
- アスキーアートの表示有無を切り替え可能
- 日本語表示と英語表示を切り替え可能
- 設定ファイルがなくても既定値で動作

## ビルド

```bash
cargo build --release
```

バイナリは `target/release/moon` に生成されます。Windows では `moon.exe` です。

## macOS の Developer ID 配布

Mac App Store 外で配布するために、`Developer ID Application` でリリースバイナリへ署名し、ZIP を notarization に送る補助スクリプトを同梱しています。

前提:

- macOS と Xcode のコマンドラインツールが入っていること
- キーチェーンに `Developer ID Application` 証明書があること
- `xcrun notarytool store-credentials` で notarytool 用のキーチェーンプロファイルを作成済みであること

例:

```bash
export CODESIGN_IDENTITY="Developer ID Application: Noriaki Fukuyori (Q6GG27UYG5)"
export NOTARY_PROFILE="moon-notary"

sh ./scripts/sign-and-notarize-macos.sh sign
sh ./scripts/sign-and-notarize-macos.sh notarize
```

まとめて実行する場合:

```bash
export CODESIGN_IDENTITY="Developer ID Application: Noriaki Fukuyori (Q6GG27UYG5)"
export NOTARY_PROFILE="moon-notary"

sh ./scripts/sign-and-notarize-macos.sh all
```

成果物は `dist/macos/` に出力されます。

補足:

- 現在のワークフローは CLI の単体バイナリへ署名し、ZIP アーカイブを notarization に送ります。
- Apple は単体バイナリや ZIP へは ticket を staple できないため、この配布形態ではオンラインの notarization 確認に依存します。

## 使い方

```bash
# 今日の月情報（選択タイムゾーン基準）
moon

# 相対日付
moon prev
moon next
moon today

# 指定日
moon 2026-04-08

# 指定日と座標
moon 2026-04-08 35.6762 139.6503

# オプション形式
moon --date 2026-04-08 --lat 35.6762 --lon 139.6503

# 現在の座標と表示設定を config.toml に保存
moon --lat 35.6762 --lon 139.6503 --write-config

# グローバルIPから概算の座標を取得
moon --detect-location
moon --detect-location --write-config

# タイムゾーンオフセット
moon --tz 09:00
moon --date today --tz -05:00

# アスキーアートの表示切り替え
moon --art
moon --no-art

# 表示言語の切り替え
moon --lang ja
moon --lang en

# ヘルプとバージョン
moon --help
moon --version

# 2026年4月を通しで確認
powershell -ExecutionPolicy Bypass -File .\scripts\check-april-2026.ps1
powershell -ExecutionPolicy Bypass -File .\scripts\check-april-2026.ps1 --no-art
sh ./scripts/check-april-2026.sh
sh ./scripts/check-april-2026.sh --no-art
```

## オプション

- `--date <YYYY-MM-DD|today|prev|next>`
- `--lat <latitude>`
- `--lon <longitude>`
- `--art`
- `--no-art`
- `--lang <ja|en>`
- `--tz <09:00|+09:00|-05:00|UTC>`
- `--detect-location`
- `--write-config`
- `--help`
- `--version`

位置引数でも `moon YYYY-MM-DD LAT LON` の形を使えます。

## 設定ファイル

設定ファイルは必須ではありません。存在しない場合は既定値で動作します。

- 既定緯度: `35.6762`
- 既定経度: `139.6503`
- 既定のアスキーアート表示: `true`
- 既定言語: `ja`
- 既定タイムゾーン: `09:00`

設定ファイルの保存先:

- Windows: `%APPDATA%\moon\config.toml`
- macOS: `~/Library/Application Support/moon/config.toml`
- Linux: `$XDG_CONFIG_HOME/moon/config.toml`
  未設定時は `~/.config/moon/config.toml`

例:

```toml
lat = 35.6762
lon = 139.6503
art = true
lang = "ja"
tz = "09:00"
```

コマンドラインで指定した値は、設定ファイルより優先されます。

`--detect-location` を付けると、グローバルIPアドレスから概算の緯度経度を取得します。ネット接続が必要で、GPS や OS のネイティブ位置情報より精度は低くなります。

`--write-config` を付けると、その実行で解決された `lat` / `lon` / `art` / `lang` / `tz` を設定ファイルへ保存してから結果を表示します。

## 出力例

```text
    時差       ── UTC+09:00
    月齢       ── 19.57 日
    照明率     ── 73.7%
    月相       ── 寝待月
    距離       ── 404933 km
    日の出     ── 05:20（081°）
    日の入     ── 18:08（280°）
    月の入     ── 07:50（227°）
    月の出     ── 23:20（119°）
```

## 実行例

コマンド:

```bash
moon 2026-04-08 --no-art
```

出力例:

```text
    時差       ── UTC+09:00
    月齢       ── 19.57 日
    照明率     ── 73.7%
    月相       ── 寝待月
    距離       ── 404933 km
    日の出     ── 05:20（081°）
    日の入     ── 18:08（280°）
    月の入     ── 07:50（227°）
    月の出     ── 23:20（119°）
```

コマンド:

```bash
moon 2026-04-08
```

出力例:

```text
                          ・                      
                 ░▒▓▓▓▓▓▓▓▓▓▓▓▒▒░░ ・・            
             ░▓▓▓███████████▓▓▓▓▒▒▒░░・・・・        
          ░▓▓█████████████████▓▓▓▓▒▒▒░░・・ ・      
       ・▒▓▓█████████████████████▓▓▓▒▒▒░░░・・ ・・   
      ░▓▓████████████████████████▓▓▓▒▒▒░░░・・・ ・  
     ░▓▓██████████████████████████▓▓▓▒▒▒░░░・・・ ・ 
    ░▓▓███████████████████████████▓▓▓▓▒▒▒░░░・・・ ・
    ▓▓████████████████████████████▓▓▓▓▒▒▒░░░░・・・ 
    ▓██████████████████████████████▓▓▓▓▒▒▒░░░・・・ 
    ▓██████████████████████████████▓▓▓▓▒▒▒░░░・・・ 
    ▓██████████████████████████████▓▓▓▒▒▒▒░░░・・・ 
    ▓▓█████████████████████████████▓▓▓▒▒▒░░░░・・・ 
    ░▓▓███████████████████████████▓▓▓▓▒▒▒░░░・・・ ・
     ░▓▓██████████████████████████▓▓▓▒▒▒░░░・・・ ・ 
      ░▓▓████████████████████████▓▓▓▒▒▒░░░・・・ ・  
       ・▒▓▓█████████████████████▓▓▓▒▒▒░░░・・ ・・   
          ░▓▓██████████████████▓▓▓▒▒▒░░・・ ・      
             ░▓▓▓████████████▓▓▓▒▒▒░░・・・・        
                 ░▒▓▓▓▓▓▓▓▓▓▓▓▒▒░░ ・・            
                          ・                      

    時差       ── UTC+09:00
    月齢       ── 19.57 日
    照明率     ── 73.7%
    月相       ── 寝待月
    距離       ── 404933 km
    日の出     ── 05:20（081°）
    日の入     ── 18:08（280°）
    月の入     ── 07:50（227°）
    月の出     ── 23:20（119°）
```

## 補足

- 時刻は選択したタイムゾーンで計算されます。
- 月の出・月の入りは、そのローカル日付に属するイベントのみ表示します。
- 天文計算は、利用できる範囲で `astro` クレートを優先して使用しています。
