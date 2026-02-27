# 元宵抽奖系统 - 管理后台 API 接口文档

**基础路径 (Base URL):** `/pro-api`
**数据格式:** `application/json`

---

## 1. 用户管理

### 1.1 获取用户列表
- **路径:** `/users`
- **方法:** `GET`
- **请求参数 (Query):**
  - `page`: 当前页码 (默认: 1)
  - `pageSize`: 每页条数 (默认: 10)
- **响应结构:**
  - `code`: 状态码 (200 为成功)
  - `data.total`: 总记录数
  - `data.list`: 用户对象数组

---

## 2. 灯谜管理

### 2.1 获取灯谜列表
- **路径:** `/riddles`
- **方法:** `GET`
- **请求参数 (Query):**
  - `page`: 当前页码
  - `pageSize`: 每页条数
- **响应结构:** 同用户列表

### 2.2 新增或修改灯谜
- **路径:** `/riddles`
- **方法:** `POST`
- **请求体 (JSON):**
  - `id`: (修改时必填) 灯谜ID
  - `question`: 谜面
  - `answer`: 谜底
  - `options`: 选项数组 (例如 `["选项A", "选项B"]`)
  - `remark`: 备注
  - `reset_status`: (boolean) 是否重置题目为未解决状态

### 2.3 批量导入灯谜
- **路径:** `/riddles/import`
- **方法:** `POST`
- **请求体 (JSON):** 灯谜对象数组 `[{question, answer, options}, ...]`

### 2.4 删除灯谜
- **路径:** `/riddle/<int:r_id>`
- **方法:** `DELETE`
- **说明:** 该接口会级联删除相关的抢答记录。

---

## 3. 统计与排行榜

### 3.1 抢答排行榜
- **路径:** `/stats/leaderboard`
- **方法:** `GET`
- **响应示例:**
```json
[
  {
    "user_id": 1,
    "username": "用户名",
    "avatar": "头像地址",
    "score": 5
  }
]
```

---

## 4. 活动设置

### 4.1 获取活动配置
- **路径:** `/activity`
- **方法:** `GET`

### 4.2 修改活动配置
- **路径:** `/activity`
- **方法:** `POST`
- **请求体 (JSON):**
  - `name`: 活动名称
  - `start_time`: "YYYY-MM-DD HH:MM:SS"
  - `end_time`: "YYYY-MM-DD HH:MM:SS"
