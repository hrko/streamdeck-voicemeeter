# VoiceMeeter for Stream Deck — Rust版

[English](README.md) | 日本語

VoiceMeeter for Stream DeckをRustで独立して再実装したものです。インストール可能な開発版パッケージを用意しています。
Gain Control（ゲイン操作）、Gain Control Combo（2チャンネルのゲイン操作）、Macroは、
Windows 11／Stream Deck +／VoiceMeeter Potatoで動作を確認しています。
リリースに向けた検証は継続中です。確認済みの動作と未検証環境は、[実機検証記録](docs/hardware-validation.md)を参照してください。

Rust版は既定ブランチの`main`（旧名`rust`）で開発しています。
旧Go版は`legacy-go`ブランチと署名付きタグ`legacy-go-before-rust`に保存しています。
対象のコミットは`8ab6ae9522423747bd3be7f9a667c82200d583dd`です。

## 対象環境と機能

- Windows x64、VoiceMeeter Basic／Banana／Potatoを対象にしています。
- Gain Control、Gain Control Combo、Macroの3つのアクションを提供します。
- 設定画面（Property Inspector、PI）はReact製で、Material Symbolsのアイコンをカスタマイズできます。
- 日本語・英語の翻訳を同梱し、翻訳リソースの追加で他の言語にも対応できます。
- ローカルとCIの開発ツールはmiseで、Node依存パッケージはpnpmで管理します。
- バージョン解決時に公開から7日間の待機期間を設け、ツールと依存パッケージのロックファイルをGitで管理します。

新しいプラグインIDは`jp.hrko.streamdeck.voicemeeter.rust`です。旧Go版の設定は移行しません。

## ライセンス

Rust版は、[LICENSE](LICENSE)に定めるソース公開型ライセンスで提供します。
業務や収益化した配信を含む商用利用、改変、無償での再配布を許可しています。
プラグインや改変版の販売、有料でのアクセス提供、有料製品への同梱には、hrkoによる事前の書面での許可が必要です。
OSIが承認するオープンソースライセンスではありません。

第三者の依存ライブラリには、それぞれのライセンスが適用されます。
旧Go版とRust版の最初のコミット`6d25302`については、元のMITライセンスによる許諾が引き続き有効です。
詳細は[従来のMITライセンス](licenses/LEGACY-MIT.txt)を参照してください。
独自のライセンス条項は、一般公開前に法律の専門家による確認を受けることを推奨します。

## 開発と検証状況

mise 2026.10.3（CIで固定しているバージョン）とVisual Studio Build Toolsをインストールしてください。
Build Toolsには「C++によるデスクトップ開発」とWindows SDKが必要です。準備後、次のコマンドを実行します。

```powershell
mise trust
mise run setup
mise run check
mise run test
mise run package
```

`setup`は、このプロジェクトでロックしたNode、pnpm、Rustのツールだけをインストールします。
無関係なグローバルツールの影響を避けるため、タスク実行時のツール自動インストールは無効にしています。
`check`には、生成したTypeScriptの型定義と翻訳リソースの整合性検査が含まれます。
Rustの設定型を変更した場合は`mise run types`を実行してください。
言語の追加方法は[翻訳手順](docs/translating.md)を参照してください。

`mise run package`は、`.streamDeckPlugin`、SHA-256チェックサム、第三者ライセンスの一覧と本文を`dist/`に生成します。
ElgatoのCLIでマニフェストとレイアウトを検証し、アーカイブ内の全ファイルをパッケージ作成前のファイルと照合します。
VoiceMeeter DLLは同梱しません。`mise run ci`は検査・テストと同じパッケージ作成処理を実行し、ロックファイルに変更がないことも確認します。

生成物は開発版です。Potatoでは、通常操作、切断中の操作を再実行しない復旧動作、ページ移動・対象変更時のPush解放、
同じ対象を扱うアクション間の表示同期、メーターの減衰、日本語ラベル、フォント設定の動作を確認しています。
その後の検証で、正常終了時のPush解放、カラオケ状態の表示、日本語・英語の切り替えと設定保持も確認しました。
Basic／Bananaでは起動、種別判定、チャンネルの選択肢を確認していますが、全機能の検証は完了していません。
Windows 10と最小対応版のStream Deck 6.4は未検証です。長期安定性は、所有者の希望により日々の使用で確認することにしています。
完了した作業の範囲は[完了・検証記録](docs/delivery-audit.md)、検証の詳細は[実装状況](docs/implementation-status.md)を参照してください。

インストール、各アクションの動作、開発版のリンク設定、ログの保存場所は[使い方と診断](docs/usage.md)に記載しています。
依存関係の更新方針と監査コマンドは[依存関係の更新](docs/dependency-updates.md)を参照してください。
リンク先の補足文書は英語です。
