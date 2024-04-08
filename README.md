# yd-lib-rs

## 生成产物

### API 封装器

api_wrapper.rs 文件为 YDApi 结构定义了一个 impl 块，提供了围绕本地 C API 函数的方法。这些方法为底层 C 库提供了一个安全、习以为常的 Rust 接口。它们处理的事项包括将 Rust 字符串转换为 C 字符串、确保内存安全以及将原始指针封装到 Rust 结构中。

### SPI 封装器

spi_wrapper.rs 文件定义了一组特质和结构，作为交易系统事件的监听器。 YDListenerTrait 特征定义了各种事件的回调，如登录成功、订单更新和市场数据。实现该特性后，您的 Rust 代码就能以类型安全的方式响应这些事件。

YDListenerStream 结构提供了一种使用 Rust 异步特性与 SPI 交互的方法。它实现了 Stream 特性，允许异步接收事件。这对于与 Rust 系统的其他部分集成特别有用，比如发送消息或更新状态以响应事件。

## 参与开发

### 生成 binding.rs

```sh
cargo build
```

这将执行 build.rs，生成 binding.rs

### 更新易达相关依赖

由于 C++ 目前没有方便的包管理器，需要手动将依赖的源码复制黏贴过来。

1. 从 [飞书文档](https://questerai.feishu.cn/wiki/RK4Ow7yXri0smPkXg4vcjK9cnwf) 下载解压 `ydClient_1_386_40_0.tgz`
1. 将得到的文件夹里的 `ydAPI_c++/` 文件夹替换到本仓库的 `crates/yd_client_sys/thirdparty/ydClient/ydAPI_c++/` 里即可。（PDF 文件已经 gitignore 了，注意看 git 只提交了代码和动态链接库文件即可。）
1. `mv crates/yd_client_sys/thirdparty/ydClient/ydAPI_c++/linux64/yd.so crates/yd_client_sys/thirdparty/ydClient/ydAPI_c++/linux64/libyd.so # fix error while loading shared libraries: libyd.so: cannot open shared object file: No such file or directory`
1. 更新相关测试，例如 `crates/yd_client_sys/tests/api_version.rs`

### 运行示例

```sh
cargo test
cargo run -p yd_client_sys --example create_yd_listener
```

### 开发容器

由于 yd 无法在 Mac 下运行，要不是使用[Linux 远程开发容器](https://questerai.feishu.cn/wiki/V9KTwpefBi5oVwkPNrfc1i4jnUb)，要不就是用 Linux 电脑。而且由于本地开发容器的 bug，需要用很慢的 osxfs (Legacy)，不能用 VirtioFS。所以还是建议用开发机。

用 VSCode 左下角的 >< 远程主机按钮，打开菜单选择「在容器内重新打开文件夹」。

#### Cannot connect to the Docker daemon at unix:///var/run/docker.sock. Is the docker daemon running?

参考 https://stackoverflow.com/a/77361735/4617295
