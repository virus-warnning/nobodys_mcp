# Nobody's MCP

一個給台灣小人物用的 MCP 工具箱

## 幹嘛用

| 工具 | 用途 |
|------|-------------|
| `web_search` | 透過 DuckDuckGo 搜尋網頁並回傳結果 |
| `cve` | 根據產品清單從 cve.org 查詢相關的資安漏洞 |
| `price` | 根據產品型號從電商平台查詢價格 |
| `s2twp` | 將內文的簡體中文用詞轉換為台灣繁體中文用詞, 讓 AI 回答看起來舒服一點 |
| `hello` | 回傳 service 名稱、用途與版本資訊 |

## 怎麼用

### AnythingLLM

詳細資訊請參考 [MCP Compatibility / Forwarding user & workspace identity](https://docs.anythingllm.com/mcp-compatibility/overview#forwarding-user--workspace-identity)

```json
{
  "mcpServers": {
    "Nobody's MCP": {
      "type": "streamable",
      "url": "http://localhost:55851/mcp",
      "anythingllm": {
        "autoStart": true
      }
    }
  }
}
```

### Zoo Code (VSCode Extension)

詳細資訊請參考 [MCP / Using MCP in Zoo Code](https://docs.zoocode.dev/features/mcp/using-mcp-in-roo#streamable-http-transport)

```json
{
  "mcpServers": {
    "Nobody's MCP": {
      "type": "streamable-http",
      "url": "http://localhost:55851/mcp",
      "headers": {
        "X-API-Key": "your-secure-api-key"
      },
      "alwaysAllow": ["hello", "web_search", "cve", "price", "s2twp"],
      "disabled": false
    }
  }
}
```

## Build

```bash
cargo build --release
```

## Run

```bash
./target/release/nobodys_mcp
```

Server listens on `http://0.0.0.0:55851/mcp` by default.

To customize, edit [`config.json`](config.json):

```json
{
  "server_name": "nobodys_mcp",
  "version": "0.1.0",
  "host": "0.0.0.0",
  "port": 55851,
  "endpoint": "/mcp"
}
```

## License

[MIT](LICENSE)
