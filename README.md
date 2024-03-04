# yd-lib-rs

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
