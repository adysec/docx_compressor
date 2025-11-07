# 📘 DOCX 图片压缩器

**DOCX Compressor** 是一个轻量级、跨平台的图形化工具，用于压缩 Word (`.docx`) 文档中的图片，以减小文件体积。  
适合用于编写标书、报告、论文等需要减小上传大小的文档场景。

---

## ✨ 功能特点

- 🖼️ **智能图片压缩**：自动检测 DOCX 内的所有图片并按指定质量与尺寸进行压缩。  
- ⚙️ **可调参数**：自定义图片压缩质量（1-100）与最大分辨率（宽度）。  
- 💾 **保持文档结构**：仅压缩图片，不影响文字、格式或样式。  
- 💻 **跨平台支持**：基于egui提供图形化界面
  - ✅ Windows（自动隐藏控制台窗口）
  - ✅ Linux（同时支持 Wayland 与 X11 桌面环境）
- 🌐 **中文界面**：内置 Noto Sans CJK 字体，完美显示中文。

---

<img width="1402" height="1068" alt="图片" src="https://github.com/user-attachments/assets/fbee7718-5f4d-4cdf-8ff2-c634a49839e7" />

## 🖥️ 使用方法

1. **运行程序**
   - Windows 用户：直接双击 `docx_compressor.exe`
   - Linux 用户：执行 `./docx_compressor`

2. **选择输入文件**
   - 点击「📂 输入文件」选择 `.docx` 文档。

3. **设置参数**
   - 压缩质量（1–100，默认 70）
   - 最大宽度（默认 1280px）

4. **选择输出文件路径**
   - 默认输出到同目录下，文件名为 `原文件名_压缩后.docx`

5. **点击「🚀 开始压缩」**
   - 进度条与耗时将实时显示。

---

## 🧩 构建说明

### 依赖项

请确保系统已安装以下依赖：

- Rust 1.75 或更高版本  
- `libfontconfig`（Linux）  
- C 编译工具链（如 `mingw`）

### 构建命令

```bash
# 克隆仓库
git clone https://github.com/AdySec/docx-compressor.git
cd docx-compressor

# 构建可执行文件
cargo build --release
cargo build --release --target x86_64-unknown-linux-gnu
cargo build --release --target x86_64-pc-windows-gnu

