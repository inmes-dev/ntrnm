# 奶甜软糯喵

![Rust Version][rustc-image]
[![crates.io][crate-image]][crate-link]
[![Documentation][docs-image]][docs-link]
[![Dependency Status][deps-image]][deps-link]

奶甜软糯喵（代号：`Ntrnm`），基于Rust构建，依赖于会话托管实现的依赖`OneBot 11` / `Kritor`的通用即时通讯跨平台基础框架。

> 写来玩的，不要看了！！！只提供学习与交流使用！！！
> 
> 本项目仅提供学习与交流用途，请在24小时内删除。
> 
> 本项目目的是研究 Tokio 框架的使用。
> 
> 如有违反法律，请联系删除。 请勿在任何平台宣传，宣扬，转发本项目，请勿恶意修改企业安装包造成相关企业产生损失，如有违背，必将追责到底。 
> 请勿用于商业用途，如有违反，后果自负。
> 

<!-- markdown-toc start - Don't edit this section. Run M-x markdown-toc-refresh-toc -->

**主页目录**

- [`ntrnm`](#奶甜软糯喵)
    - [许可操作](#支持的操作)
    - [风险行为](#禁止的操作)
- [构建方法](#构建及部署)
    - [从源代码构建](#从源代码构建)
- [使用方法](#使用方法)
    - [命令行](#命令行)
- [License](#license)
    - [贡献与感谢](#contribution)

<!-- markdown-toc end -->

## 支持的操作

<details>

<summary>以下是该项目支持的操作</summary>

| Login | State              | Group | State |
|-------|--------------------|-------|-------|
| 密码登录  |                    | 获取群列表 |       |
| 二维码登录 |                    |       |       |
| 托管登录  | :heavy_check_mark: |       |       |

</details>

> It is recommended to use the `Shamrock` framework to export login tickets, login via tickets has bypassed the wind control as well as signature errors.

## 禁止的操作

<details>

<summary>不会支持的操作</summary>

- **金钱敏感操作**

</details>

# 构建及部署

`ntrnm` is a single binary that must be placed somewhere in your `$PATH`.

One can either download 64-bit Linux binaries from [the Release Page](https://github.com/inmes-dev/ntrnm/releases)
or one can also compile from source.

## 从源代码构建

Ensure you have a [Rust toolchain installed](https://rustup.rs). Some of the
dependencies also require `gcc` to be installed.

```
$ git clone https://github.com/inmes-dev/ntrnm
$ cd ntrnm
$ cargo build --release
$ sudo cp target/release/ntrnm /usr/local/bin/
```

# 使用方法

## 命令行

```
$ ntrnm --help
```

# License

 * [GPLv3 license](https://opensource.org/license/gpl-3-0)

## Contribution

[![][contrib-image]][contrib-link]

[//]: # (badges)

[rustc-image]: https://img.shields.io/badge/rustc-1.73+-blue.svg
[crate-image]: https://img.shields.io/crates/v/ntrnm.svg
[crate-link]: https://crates.io/crates/ntrnm
[docs-image]: https://docs.rs/ntrnm/badge.svg
[docs-link]: https://docs.rs/ntrnm
[deps-image]: https://deps.rs/repo/github/inmes-dev/ntrnm/status.svg
[deps-link]: https://deps.rs/repo/github/inmes-dev/ntrnm
[contrib-image]: https://contrib.rocks/image?repo=inmes-dev/ntrnm
[contrib-link]: https://github.com/inmes-dev/ntrnm/graphs/contributors
