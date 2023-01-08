# chrome-access-test
## 概要
- 設定ファイルで指定した監視対象のWEBサービスが稼働していることを確認するアクセステストを行います。
- Headless ChromeをAPIで操作してアクセステストを行います。
- アクセステストに失敗した場合、Google Chatに通知します。
## 新たに監視対象を追加する方法
- 「conf/service」ディレクトリ内に、toml形式の設定ファイルを追加します。
- 設定ファイル内に記載する内容は下記の形式になります。
### ログインしない場合
```
access_url = "監視対象のURL"
find_selector = "描画出来ているか検索するCSSセレクタ"
```
### ログインする場合
先に認証情報cookieを特定しておく必要があります。
```
access_url = "監視対象のURL"
find_selector = "描画出来ているか検索するCSSセレクタ"
[cookie]
name = "cookieの名前"
value = "cookieの値"
```
