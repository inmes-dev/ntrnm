# Ntrim Environment

设置环境变量，可以改变程序运行的相关参数。

## 语法

```shell
IS_NT_IPV6=1 ./ntrim
```

上述操作，将会启用 IPv6 的连接与`trpc`服务器。

## 参数列表

| 参数名                  | 说明             | 默认值               |
|----------------------|----------------|-------------------|
| RUST_LOG             | 日志级别           | info              |
| IS_NT_IPV6           | 是否启用 trpc IPv6 | 0                 |
| NT_SEND_QUEUE_SIZE   | trpc协议发包队列大小   | 32                |
| HEARTBEAT_INTERVAL   | 标准心跳间隔时间(秒)    | 270               |
| AUTO_RECONNECT       | trpc自动重连       | 1                 |
| RECONNECT_INTERVAL   | trpc自动重连间隔(秒)  | 5                 |
| AUTO_REFRESH_SESSION | 自动刷新质押的会话      | 1                 |
| REFRESH_ADVANCE_TIME | 自动会话刷新时间提前(秒)  | 60 * 60 * 24 * 25 |

### HEARTBEAT_INTERVAL

默认要求的心跳的心跳270秒，如果大于该时间，可能导致掉线！

### AUTO_REFRESH_SESSION

> `AUTO_REFRESH_SESSION` 默认为开启，质押的会话会在刷新过后的一个月后过期。
> 
> 开启自动刷新可以保证长时间挂机不会掉线，但是会增加封号风险。

### REFRESH_ADVANCE_TIME

`REFRESH_ADVANCE_TIME` 即会在会话过期提前**n**秒刷新会话。

会话刷新即，会话过期前会自动刷新会话，保证不掉线。

#### 使用质押会话模式操作

该模式提供一上线就自动刷新会话的操作：

```shell
.\ntrim.exe -c [配置文件路径] session -s [质押会话路径] -i true
```
