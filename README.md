# web

一个用于"浏览互联网"的 MCP（Model Context Protocol）服务器。它通过 Streamable HTTP 暴露三个工具：联网搜索、打开网页、在网页中按正则查找。返回结果均为结构化 JSON，适合作为大模型的检索与阅读后端。

服务标识：名称 `web`，说明 `Browsing the Internet`，协议版本 `2025-06-18`。

## 功能概览

- `search_query`：调用 Exa 搜索，返回每条结果的标题、日期、URL 与摘要。
- `open`：抓取指定 URL 的网页，按 token 切分后返回指定分块的正文。
- `find`：抓取指定 URL 的网页，在其中用正则匹配文本并返回带上下文的片段。

服务还内置了对常见站点的直连抓取（代码托管、包仓库、维基类站点、Microsoft Learn 等），抓不到时回退到 Jina Reader；同时对所有用户提供的 URL 施加 SSRF 防护。

## 协议与端点

服务以 MCP over Streamable HTTP 提供，默认监听 `0.0.0.0:18080`。

- MCP 端点：`POST /mcp`
- 健康检查：`GET /health`，返回 `{"status":"ok","name":"web"}`

默认采用无状态模式（`stateful_http: false`）并直接返回 JSON（`json_response: true`），即客户端发起请求后会直接得到 JSON 响应，而非 SSE 流。

MCP 客户端连接示例（指向 MCP 端点即可）：

```
http://<host>:18080/mcp
```

可选的 `allowed_hosts` 与 `allowed_origins` 用于约束请求的 `Host` 头与浏览器 `Origin`；二者为空时分别表示接受任意 Host、不做 Origin 校验。

## 认证头

不同工具依赖不同的上游服务，凭据通过 HTTP 头随请求传入：

- `x-exa-api-key`：`search_query` 必需（Exa 搜索）。
- `x-jina-api-key`：仅当目标网页无法直连抓取、需要回退到 Jina Reader 时才需要。能被直连抓取的站点（见下文）无需该头。

## 工具：search_query

调用 Exa 搜索，返回标题、日期、URL 与摘要。

请求参数（位于 `requests` 数组的每个元素中）：

- `q`：查询词，必填字符串。
- `recency`：可选，表示"最近 N 天内发布"，会被转换为起始发布日期。
- `domains`：可选，限定站点的域名数组，例如 `["openai.com"]`。
- `category`：可选，取值之一：`company`、`research paper`、`news`、`pdf`、`personal site`、`financial report`、`people`。

请求示例：

```json
{
  "requests": [
    {
      "q": "OpenAI",
      "recency": 30,
      "domains": ["openai.com"],
      "category": "research paper"
    }
  ]
}
```

响应示例：

```json
{
  "results": [
    {
      "title": "标题，可能为空",
      "date": "发布日期，可能为空",
      "url": "https://example.com/...",
      "summary": "命中的高亮片段，按行拼接"
    }
  ],
  "warning": ["对输入做出的规范化提示（可选）"]
}
```

多个 `requests` 会并发执行，结果按顺序汇总到同一个 `results` 列表中。每次请求默认返回 15 条结果，单条摘要高亮上限约 1000 字符。

## 工具：open

打开 `url` 指向的网页，返回正文中的某个分块。

请求参数：

- `url`：必填，绝对的 HTTP/HTTPS URL。
- `chunk`：要返回的分块序号，从 1 开始；缺省、为空或小于 1 时按 1 处理。

请求示例：

```json
{
  "requests": [
    { "url": "https://example.com/article", "chunk": 1 }
  ]
}
```

响应示例：

```json
{
  "pages": [
    {
      "chunk": 1,
      "total_chunks": 3,
      "content": "该分块的正文内容"
    }
  ],
  "warning": ["分块越界等提示（可选）"]
}
```

正文会先转换为 Markdown，再按 token 切分（默认每块 5000 token，相邻块间约 10% 重叠）。短页面通常只有一个分块（`total_chunks` 为 1）。当请求的 `chunk` 超出范围时，会返回第 1 块并在 `warning` 中给出可用范围提示。先用 `total_chunks` 了解页面被切成了几块，再按需逐块翻阅。

## 工具：find

抓取 `url` 指向的网页，并在其中查找文本 `pattern`。

请求参数：

- `url`：必填，绝对的 HTTP/HTTPS URL。
- `pattern`：必填，正则表达式（支持前瞻/后顾等高级语法，并默认按多行模式匹配）。
- `snippet_tokens`：可选，每个命中片段的 token 预算；默认 200，且不超过分块上限 5000，超出时按 5000 截断并给出提示。

请求示例：

```json
{
  "requests": [
    { "url": "https://example.com/page", "pattern": "\\w+(?=!)", "snippet_tokens": 200 }
  ]
}
```

响应示例：

```json
{
  "pages": [
    {
      "total_chunks": 3,
      "matches": [
        { "chunk": 1, "snippet": "命中处及其上下文" }
      ]
    }
  ],
  "warning": ["snippet_tokens 越界等提示（可选）"]
}
```

查找会遍历页面的所有分块，每个命中返回所在分块号与一段包含上下文的片段；单页最多返回 50 个命中。片段会尽量将命中文本居中，并把剩余 token 预算分配到两侧。

## 参数书写便利特性

为减少调用方的格式负担，输入会先经过规范化，并把所有修正写入响应的 `warning` 字段，便于自查：

- 字段别名：`q` 可写作 `query`/`queries`；`url` 可写作 `urls`；`chunk` 可写作 `chunks`；`pattern` 可写作 `patterns`；`domains` 可写作 `domain`；`recency` 可写作 `recencies`；`category` 可写作 `categories`；`snippet_tokens` 可写作 `snippet_token`。字段名大小写不敏感。
- 单请求免数组：可以直接传一个请求对象（甚至直接传字段），会被自动包进 `requests` 数组。
- URL 自动补全协议：缺少协议头的 URL 会被补成 `https://`。
- 类别名称归一化：例如 `Research Papers` 会被识别为 `research paper`，连字符/下划线也会被容错处理。
- `domains` 建议传数组；若误用查询里的 `site:` 语法，会提示改用 `domains`。
- JSON 字符串自动解析：若把对象/数组以字符串形式传入，会尝试解析为真正的结构。

这些修正只影响"如何理解你的输入"，不会改变工具语义；提示信息仅供参考。

## 网页抓取行为

`open` 与 `find` 在抓取前会优先尝试直连，命中以下规则时直接取原始内容，通常无需 Jina API key：

- 代码托管：GitHub（含 `raw.githubusercontent.com`、Gist）、GitLab、Bitbucket、Hugging Face 的 `blob`/`raw`/`resolve` 链接，会被换算成原始文件地址，仅对可识别的文本类扩展名生效。
- Stack Overflow：问题页会通过 Stack Exchange API 直连读取，返回 `question` 与 `answers` 组成的 JSON。
- 包仓库：PyPI（`pypi.org/project/...` 等）、npm（`npmjs.com/package/...`、`registry.npmjs.org/...`）、crates.io，会取仓库 JSON 并做字段整理。
- 维基类站点：Wikipedia 等 Wikimedia 站点以及 `*.fandom.com`，会通过其 API 取单页内容，支持按标题、`oldid`、`curid` 等定位。
- Microsoft Learn：`learn.microsoft.com` 取其 Markdown 版本。
- 通用回退：对原始 URL 尝试 `Accept: text/markdown`，以及带 `.md` 后缀的地址。

当直连全部失败时，会回退到 Jina Reader，此时需要提供 `x-jina-api-key`。此外，`arxiv.org/pdf/...` 会在回退抓取时改写为 `arxiv.org/html/...`。直连内容大小上限为 1 MiB。

## 安全限制（SSRF）

对所有用户提供的 URL 与重定向目标统一校验：

- 仅允许 `http`/`https`。
- 不允许带凭据的 URL（形如 `user:pass@host`）。
- 默认拦截环回、私有、链路本地及保留网段的 IP（IPv4 与 IPv6）。
- 默认拦截 `localhost` 类主机名，以及 `.local`、`.localhost` 后缀域名。
- DNS 解析阶段同样受控，避免解析到内网地址后再连接。

重定向跳转上限默认为 5 跳，每跳都会重新校验。

## 配置项

服务的默认配置见 `config/default.yaml`，主要分组如下：

- `server`：监听地址与端口、`/mcp` 与 `/health` 路径、协议版本、有/无状态模式、JSON 响应开关、Host/Origin 允许列表、日志级别。
- `headers`：Exa 与 Jina 的密钥头名称（`x-exa-api-key`、`x-jina-api-key`）。
- `search`：Exa 端点、返回条数（默认 15）、搜索类型（`deep-lite`）、高亮字符上限、缓存与实时抓取超时。
- `http`：Jina/Exa 超时（默认 120s）、直连抓取超时（默认 20s）、最大重定向次数（默认 5）。
- `jina`：Jina Reader 端点与渲染引擎、返回格式、视口等参数。
- `chunking`：分词器（`o200k_base`）、单块 token 上限（默认 5000）、重叠比例（默认 0.1）。
- `find`：默认片段 token（200）、单页最大命中数（50）。
- `direct_fetch`：直连内容大小上限、各代码托管站点的主机列表、可直连的文本扩展名与文件名。
- `ssrf`：是否拦截私有网络与本地主机名。

日志级别可通过环境变量 `RUST_LOG` 覆盖，未设置时使用 `info`。
