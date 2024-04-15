# Ntrim Environment

设置环境变量，可以改变程序运行的相关参数。

## 语法

```shell
IS_NT_IPV6=1 ./ntrim
```

上述操作，将会启用 IPv6 的连接与`trpc`服务器。

## 参数列表

| 参数名                | 说明             | 默认值 |
|--------------------|----------------|-----|
| IS_NT_IPV6         | 是否启用 trpc IPv6 | 0   |
| NT_SEND_QUEUE_SIZE | trpc协议发包队列大小   | 32  |
