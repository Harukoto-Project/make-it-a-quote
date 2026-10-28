<div align="center">

<div style="display: flex; align-items: center; justify-content: center; gap: 24px;">
  <img src="public/miq-rs.svg" alt="Spinel" width="128" height="128" />
  <h1 style="margin: 0;">Make it a Quote</h1>
</div>

Make it a Quote形式の引用画像を、PNGで描画します

[![License: Apache-2.0](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-stable-000000?logo=rust&logoColor=white)](rust-toolchain.toml)
[![Release miq-api](https://github.com/Harukoto-Project/miq-rs/actions/workflows/release.yml/badge.svg)](https://github.com/Harukoto-Project/miq-rs/actions/workflows/release.yml)

[概要](#概要) · [制限](#制限) · [セットアップ](#セットアップ) · [使い方](#使い方) · [HTTP API](#http-api) · [ライセンス](#ライセンス)

</div>

---

## 概要

`miq-rs`は、引用文・表示名・ユーザー名・透かし・アバターから、1200×630のPNGを作る。CLI、HTTP API、ライブラリの3つの入口があります。

描画に使うフォントとTwemojiはリポジトリに含まれない。実行前に[セットアップ](#セットアップ)の取得スクリプトを動かす必要があります。

## 制限

- 出力サイズは1200×630で固定されます。色付きアバター、白背景、左右反転の指定はありません。
- フォント名を選ぶAPIは未実装です。`scripts/fetch-assets.sh`は描画が読まないフォントファイルも取得します。
- 対応するTwemojiのSVGが無い絵文字は、文字幅だけ進めて絵は出しません。
- 折り返し結果が0行のとき、表示名とユーザー名は描きません。
- HTTP APIは本文を必須にし、Rustの`chars`で数えて4000までとします。CLIはこの長さを検査しません。

## 構成

| パス                            | 役割                                |
| ------------------------------- | ----------------------------------- |
| `crates/miq-render`             | 描画ライブラリ                      |
| `crates/miq-cli`                | CLI。バイナリ名は`miq`              |
| `crates/miq-api`                | HTTPサーバ。バイナリ名は`miq-api`   |
| `scripts/fetch-assets.sh`       | フォントとTwemojiの取得             |
| `assets/`                       | 取得先。Gitの対象外                 |
| `.github/workflows/release.yml` | `miq-api`のLinux x86_64向けリリース |

## 動作要件

- Rust
- `scripts/fetch-assets.sh`はbashスクリプトで、`curl`と`tar`を使います
- リリース用のGitHub ActionsはUbuntu 24.04です

## セットアップ

1. フォントとTwemojiを取得する。

```bash
./scripts/fetch-assets.sh
```

2. 必要なバイナリをビルドする。

```bash
cargo build --release -p miq-cli -p miq-api
```

成果物は`target/release/miq`と`target/release/miq-api`。フォントの場所は環境変数`MIQ_ASSETS`、またはCLIの`--assets`で渡します。未指定のときは、`assets/fonts`があるディレクトリを順に探します。

## 使い方

### CLI

```bash
cargo run -p miq-cli -- \
  --text "こんにちは" \
  --display-name Harukoto \
  --username example \
  --watermark miq \
  --avatar ./avatar.png \
  -o quote.png
```

| オプション       | 既定         | 意味                            |
| ---------------- | ------------ | ------------------------------- |
| `--text`         | なし（必須） | 引用本文                        |
| `--display-name` | 空           | 表示名。画像では`- 名前`になる  |
| `--username`     | 空           | 空でなければ`@名前`になる       |
| `--watermark`    | 空           | 空でなければ右下に描く          |
| `--avatar`       | なし         | PNG、JPEG、WebP、GIFのファイル  |
| `-o`, `--output` | `quote.png`  | 出力パス                        |
| `--assets`       | `MIQ_ASSETS` | フォントとTwemojiのディレクトリ |

標準エラーに、出力パス、文字サイズ、行、打ち切りの有無を出す。

### ライブラリ

`miq-render`を同じワークスペースから使う。

```rust
use miq_render::{encode_png, render_quote, FontCatalog, RenderOptions};

let catalog = FontCatalog::load_default()?;
let opts = RenderOptions {
    text: "こんにちは".into(),
    display_name: "Harukoto".into(),
    username: String::new(),
    watermark: String::new(),
};
let out = render_quote(&catalog, &opts, None)?;
let png = encode_png(&out.image)?;
```

`render_quote`の第3引数はアバター画像のバイト列。`None`のときはプレースホルダになる。

## HTTP API

`miq-api`は既定で`0.0.0.0:8080`を待つ。

```bash
MIQ_ASSETS=./assets cargo run -p miq-api
```

### エンドポイント

| メソッド | パス        | 応答                |
| -------- | ----------- | ------------------- |
| `GET`    | `/healthz`  | 本文`ok`            |
| `POST`   | `/v1/quote` | 成功時は`image/png` |

`/v1/quote`はJSONと`multipart/form-data`を受け付ける。成功時のヘッダは次のとおり。

| ヘッダ              | 意味                   |
| ------------------- | ---------------------- |
| `content-type`      | `image/png`            |
| `cache-control`     | `no-store`             |
| `x-quote-font-size` | 採用した文字サイズ     |
| `x-quote-lines`     | 行数                   |
| `x-quote-truncated` | 打ち切ったときは`true` |

失敗時はJSON。形は`{"error":"...","message":"..."}`。

| 状態 | `error`                | 条件                                                                       |
| ---- | ---------------------- | -------------------------------------------------------------------------- |
| 400  | `invalid_field`        | JSONや必須項目の不備、本文の長さ、`https`でないアバターURL、拒否したホスト |
| 401  | `unauthorized`         | `MIQ_TOKEN`があるのにBearerが無い、または一致しない                        |
| 413  | `payload_too_large`    | アバターが8MiBを超える                                                     |
| 422  | `avatar_decode_failed` | 描画関数が失敗したとき。アバターのデコード以外でもこのコードになる         |
| 500  | `render_failed`        | PNGへの符号化に失敗した                                                    |
| 502  | `avatar_fetch_failed`  | アバターURLの取得に失敗した                                                |

リクエストボディの上限は10MiB。

JSONの例:

```bash
curl -sS -X POST http://127.0.0.1:8080/v1/quote \
  -H 'content-type: application/json' \
  -d '{"text":"こんにちは","display_name":"Harukoto"}' \
  -o quote.png
```

| フィールド     | 必須   | 意味                                  |
| -------------- | ------ | ------------------------------------- |
| `text`         | はい   | 引用本文。空は不可。`chars`で4000まで |
| `display_name` | はい   | 表示名                                |
| `username`     | いいえ | 既定は空文字                          |
| `watermark`    | いいえ | 既定は空文字                          |
| `avatar_url`   | いいえ | `https`の画像URL                      |

multipartでは`text`と`display_name`が必須。追加のフィールド`avatar`に画像バイトを渡せる。`avatar`が無いときだけ`avatar_url`を取得する。デコードできる形式はPNG、JPEG、WebP、GIF。

`avatar_url`はタイムアウト5秒、リダイレクトは3回まで。`MIQ_AVATAR_HOSTS`にカンマ区切りのホストを書くと、そのホストだけを許可する。未設定なら、一部のローカルホストを除いて`https`を許可する。DiscordのCDNでアバターやアイコンのURLは、クエリ`size=512`を付けて取得する。

`MIQ_TOKEN`を設定したときは、`Authorization: Bearer <token>`が必要。未設定のときは認証しない。

### 環境変数

| 変数               | 既定           | 意味                                       |
| ------------------ | -------------- | ------------------------------------------ |
| `MIQ_LISTEN`       | `0.0.0.0:8080` | 待ち受けアドレス                           |
| `MIQ_ASSETS`       | 後述の探索     | `fonts/`と`twemoji/svg/`があるディレクトリ |
| `MIQ_TOKEN`        | なし           | 空でなければBearer認証を有効にする         |
| `MIQ_AVATAR_HOSTS` | なし           | アバターURLに許可するホスト。カンマ区切り  |
| `RUST_LOG`         | tracingの既定  | `miq-api`のログフィルタ                    |

`MIQ_ASSETS`が無いとき、描画ライブラリは`assets`、`../assets`、`../../assets`、`/workspace/assets`の順に`fonts/`を探す。

## 第三者の素材

- フォントは[Google Fonts](https://github.com/google/fonts)から取得する。描画が読むのはM PLUS Rounded 1cと、Noto Sans（JP、SC、TC、KR、ラテン、アラビア、ヘブライ、タイ、デーヴァナーガリー、数学、記号）
- 絵文字は[jdecked/twemoji](https://github.com/jdecked/twemoji)のv15.1.0
- 日本語の折り返しモデルは`crates/miq-render/src/budoux_ja.json`。形式は[Budoux](https://github.com/google/budoux)に沿っている

これらの条件は、このリポジトリのApache License 2.0とは別である。

## コントリビュート

IssueとPull Requestを歓迎します。手順は[CONTRIBUTING.md](CONTRIBUTING.md)をご覧ください。

## ライセンス

[Apache License 2.0](LICENSE)で公開しています。

<div align="center">

---

[Harukoto-Project](https://github.com/Harukoto-Project)

</div>