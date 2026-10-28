# コントリビュートについて

IssueとPull Requestを歓迎します。

## 開発環境

- OS: Windows / Linux / macOS
- 言語: Rust

## 提出前の確認

- `./scripts/fetch-assets.sh` — フォントとTwemojiを`assets/`へ取得する。bashで、`curl`と`tar`を使う
- `cargo test --workspace --exclude miq-cli` — ワークスペースのテストを実行する。`.github/workflows/release.yml`が使っているコマンドで、`miq-cli`は対象外
- `cargo build --release -p miq-api` — `miq-api`のリリースビルド。同じワークフローが使っている

`crates/miq-render`のレイアウト照合テストは、フォント取得後に`testdata/samples.json`を読む。`testdata/`は`.gitignore`にあり、リポジトリには含まれない。ファイルが無いと、フォント取得後のそのテストは失敗する。

## バグ報告

次の情報があると調査しやすくなります。

- 使った入口（`miq` CLIか`miq-api`か、`miq-render`を直接呼んだか）
- OSと、Rustのtoolchain channel
- `./scripts/fetch-assets.sh`を実行したか。`MIQ_ASSETS`の値
- 入力（`text`、`display_name`、`username`、`watermark`、アバターの有無と形式）
- APIの場合は、`Content-Type`（JSONかmultipartか）、ステータスコード、応答JSONの`error`と`message`、ヘッダ`x-quote-font-size`、`x-quote-lines`、`x-quote-truncated`
- 期待した折り返し・省略・絵文字と、実際のPNGの違い
- 再現手順

## セキュリティ上の問題

セキュリティに関わる問題は公開Issueではなく、GitHubのSecurity Advisory(Report a vulnerability)から非公開で報告してください。

## ライセンス

コントリビュートされた内容は、本プロジェクトと同じApache License 2.0で提供されるものとします。
