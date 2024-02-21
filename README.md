# yd-lib-rs

## 参与开发

### 生成 binding.rs

```sh
cargo build
```

这将执行 build.rs，生成 binding.rs

### 更新易达相关依赖

由于 C++ 目前没有方便的包管理器，需要手动将依赖的源码复制黏贴过来。

1. 解压 `ydClient_1_386_40_0.tgz`
1. 将得到的文件夹里的 `ydAPI_c++/` 文件夹替换到本仓库的 `ydClient/ydAPI_c++` 里即可。（PDF 文件已经 gitignore 了，注意看 git 只提交了代码和动态链接库文件即可。）

### 运行示例

```sh
# fix `error while loading shared libraries: libyd.so: cannot open shared object file: No such file or directory`
ln -s ydAPI_c++/linux64/yd.so ydAPI_c++/linux64/libyd.so
# fix `/usr/bin/ld: cannot find -lyd: No such file or directory` error
export LD_LIBRARY_PATH=/root/repo/yd-lib-rs/ydClient/ydAPI_c++/linux64:$LD_LIBRARY_PATH
cargo run --example yd_version
```
