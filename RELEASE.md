# 自动发布指南

本项目配置了GitHub Actions来自动构建和发布Windows和macOS版本。

## 如何触发发布

当你在GitHub上创建一个新的tag时，会自动触发构建和发布流程：

```bash
# 本地创建并推送tag
git tag v1.0.0
git push origin v1.0.0
```

或者在GitHub网页界面创建release时选择"Create a new tag"。

## 发布流程

1. **触发条件**: 推送以 `v` 开头的tag（如 `v1.0.0`, `v2.1.3`）
2. **构建平台**: 
   - Windows x64 (x86_64-pc-windows-msvc)
   - macOS x64 (x86_64-apple-darwin)
   - macOS ARM64 (aarch64-apple-darwin)
3. **输出文件**:
   - `mousepilot-windows-x64.zip` - Windows可执行文件
   - `mousepilot-macos-x64.zip` - macOS Intel可执行文件
   - `mousepilot-macos-arm64.zip` - macOS Apple Silicon可执行文件

## 构建优化

- 使用Rust stable工具链
- 启用LTO (Link Time Optimization)
- 二进制文件strip优化
- 依赖缓存加速构建
- 针对目标CPU优化


## 手动触发

如果需要手动触发构建（不创建tag），可以修改工作流文件：

```yaml
on:
  push:
    tags:
      - 'v*'
  workflow_dispatch:  # 添加这行允许手动触发
```

## 注意事项

- 确保项目在Windows和macOS上都能正常编译
- 检查所有依赖是否支持目标平台
- 测试生成的二进制文件在目标平台上是否正常工作 