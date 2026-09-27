# Hướng dẫn cài đặt F.Auto

Tài liệu dành cho khách hàng. **Một file binary tĩnh duy nhất, không cần runtime** — không Node,
không Python, không Docker, không cần toolchain, không cần quyền admin. Cài mới mất khoảng năm phút,
và gần như toàn bộ thời gian đó là chạy một dòng lệnh cài đặt rồi `fauto config`.

> Tài liệu này chỉ nói về **cài đặt và cấu hình lần đầu**. Muốn xem toàn bộ lệnh, xem
> [hướng dẫn sử dụng đầy đủ](usetut_vi.md) hoặc [bản tham chiếu tiếng Anh](REFERENCE.md). Cơ chế
> phân quyền nằm ở [SANDBOX.md](SANDBOX.md).

---

## 1. Bạn cần gì

| | |
|---|---|
| **Hệ điều hành** | Windows 10/11 (x86_64) · Linux (x86_64) · macOS (Apple Silicon; máy Intel không còn được hỗ trợ) |
| **Terminal** | Một terminal tương tác thật. REPL cần TTY — nếu stdin là pipe/CI, chương trình in gợi ý rồi thoát |
| **Model** | Một endpoint `/chat/completions` tương thích OpenAI và API key tương ứng — OpenAI, OpenRouter, llama.cpp/vLLM chạy nội bộ, gateway Anthropic, … |

Cảnh báo file chưa ký số và các lưu ý lần chạy đầu nằm ở [§6 Những hành vi cần biết](#6-những-hành-vi-cần-biết).

## 2. Cài đặt

### Windows (PowerShell 5+)

```powershell
irm https://raw.githubusercontent.com/Huutantv/factory-ai-main_v2/main/install.ps1 | iex
```

Trình cài tải `fauto.exe` mới nhất từ GitHub Releases, đặt vào `%LOCALAPPDATA%\F.Auto` và thêm thư
mục đó vào **PATH của người dùng**. Không cần quyền admin.

Muốn đổi thư mục cài, đặt `$env:FAUTO_INSTALL` trước khi chạy dòng lệnh trên.

### Linux và macOS

```bash
curl -fsSL https://raw.githubusercontent.com/Huutantv/factory-ai-main_v2/main/install.sh | sh
```

Cài vào `~/.fauto/bin/fauto` (đổi bằng `$FAUTO_INSTALL`). Script **không** sửa file cấu hình shell
của bạn; nó in ra đúng một dòng `export PATH="…"` để bạn thêm vào `~/.bashrc` hoặc `~/.zshrc`.

`fauto update` thay thế binary trong chính thư mục cài mà nó đang chạy; nếu thư mục đó không ghi
được, đặt `AIZEN_INSTALL` trỏ tới thư mục cài bạn sở hữu.

### Sau đó, mở một terminal **mới**

```bash
fauto --version     # phải in ra: F.Auto 1.7.7
```

Nếu báo không tìm thấy lệnh, shell mới chưa nạp PATH — hãy mở terminal mới (trên Linux/macOS thêm
dòng `export PATH` ở bước trên trước).

<details>
<summary>Cài thủ công (không dùng script)</summary>

Tải gói cho đúng hệ điều hành từ
[bản release mới nhất](https://github.com/Huutantv/factory-ai-main_v2/releases/latest), giải nén rồi
đặt binary `fauto` vào một thư mục có trong PATH.

Hoặc tự build — cần có sẵn Rust toolchain:

```bash
cargo install --git https://github.com/Huutantv/factory-ai-main_v2
```

</details>

## 3. Cấu hình model — `fauto config`

```bash
fauto config
```

Đây là bước thiết lập bắt buộc duy nhất. Trình hướng dẫn chạy qua các bước ngắn và **lưu ở bước
cuối**, nên bấm `Ctrl-C` trước đó sẽ hủy sạch, không lưu gì:

1. **Kết nối** — chọn một preset nhà cung cấp hoặc nhập base URL, rồi dán API key. Key được kiểm tra
   thật với endpoint ngay tại đây (key sai sẽ báo lỗi ở bước này, không phải ở câu hỏi đầu tiên).
2. **Model & ngữ cảnh** — chọn từ danh sách model mà endpoint trả về, hoặc tự nhập id. Cửa sổ ngữ
   cảnh được điền sẵn nếu nhà cung cấp báo, không thì ước lượng theo tên model.
3. **Tìm kiếm web** *(tùy chọn)* — key của dịch vụ tìm kiếm. `web_search` bắt buộc phải có key, nên
   không nhập thì agent vẫn tải được URL nhưng không tìm kiếm được. Bỏ qua bằng Enter, thêm sau.
4. **Hành vi** — ngưỡng tự nén ngữ cảnh, và hai cơ chế tự học (kỹ năng và bộ nhớ dài hạn).
5. **Hiển thị** — kiểu icon: `emoji` (hiện trên mọi font), `nerd` (cần Nerd Font), hoặc `off`.

Cấu hình lưu ở `~/.aizen/cli-config.json` (`%USERPROFILE%\.aizen\cli-config.json` trên Windows), ghi
với quyền chỉ chủ sở hữu đọc được. Bạn không bao giờ phải sửa file bằng tay.

### Hoặc cấu hình không tương tác

```bash
fauto config set --base-url https://api.openai.com/v1 --api-key sk-... --model gpt-4o-mini
```

Thứ tự ưu tiên cho mọi trường kết nối là **cờ CLI → biến môi trường → cấu hình đã lưu**. Nên có thể
ghi đè cho riêng một shell mà không đụng tới file:

| biến môi trường | cờ |
|---|---|
| `AIZEN_BASE_URL` | `--base-url` |
| `AIZEN_API_KEY` | `--api-key` |
| `AIZEN_MODEL` | `-m, --model` |

Xem cấu hình đang thực sự có hiệu lực bằng `fauto config show`; `fauto config path` in ra đường dẫn
file. `fauto config set --help` liệt kê mọi trường có thể đặt.

### Kiểm tra kết nối

```bash
fauto models                       # liệt kê model endpoint báo về
fauto chat -p "chào bạn trong một dòng"
```

Nếu `fauto chat` trả lời được, bạn đã xong.

## 4. Bắt đầu làm việc

```bash
fauto
```

Mở thẳng vào REPL chat + agent hợp nhất — không tách chế độ, cứ gõ. Câu bình thường thì trả lời; việc
cần công cụ thì nó tự dùng. Thử:

```bash
fauto agent "chạy test rồi sửa chỗ nào fail"
```

Vài lệnh nên biết:

| lệnh | tác dụng |
|---|---|
| `fauto config` | chạy lại trình cấu hình bất cứ lúc nào |
| `fauto --help` | danh sách lệnh cấp cao nhất |
| `/help` trong REPL | tầng lệnh slash |
| `fauto update` | nâng cấp **hoặc quay về** phiên bản bất kỳ đã phát hành |

## 5. Cập nhật

```bash
fauto update          # liệt kê mọi bản đã phát hành, chọn bản để cài
```

`fauto update` đánh dấu sẵn bản đang chạy trong danh sách và mặc định chọn bản ổn định mới nhất, nên
nó vừa là cách cập nhật vừa là cách kiểm tra phiên bản. `fauto --version` cho biết bạn đang chạy bản
nào; `fauto config show` có kèm mục kiểm tra cập nhật.

## 6. Những hành vi cần biết

Đây là các giới hạn hiện tại, nói thẳng — không phải lỗi bạn cần tìm cách lách.

- **SmartScreen trên Windows.** File `.exe` **chưa ký số**, nên lần chạy đầu sẽ hiện cảnh báo:
  *More info → Run anyway* (Xem thêm → Vẫn chạy). Đây là điều bình thường ở mọi bản release cho tới
  khi binary được ký số.
- **macOS chưa notarized.** Gatekeeper sẽ cần bạn cho phép thủ công lần đầu (`System Settings → Privacy
  & Security → Open Anyway`). Chỉ hỗ trợ Apple Silicon.
- **Chưa có trên trình quản lý gói.** F.Auto chưa phát hành qua winget / scoop / Homebrew / crates.io /
  AUR. Hai script cài ở trên là đường cài được hỗ trợ chính thức.
- **Kiểm tra cập nhật tự động.** REPL kiểm tra bản mới mỗi ngày một lần ở nền (có cache, tối đa một
  dòng). Tắt bằng `fauto config set --update-check false` hoặc đặt `AIZEN_NO_UPDATE_CHECK=1`.
- **Cần terminal thật.** REPL in gợi ý rồi thoát khi stdin là pipe/CI. Muốn dùng trong script, dùng
  dạng một-lần `fauto chat -p "…"` hoặc `fauto agent "…"`; các lượt chạy không giám sát sẽ từ chối
  chạy (fail closed) ở khâu duyệt lệnh.
- **API key nằm trong file cấu hình.** File được ghi quyền chỉ chủ sở hữu, và key bị loại khỏi môi
  trường của mọi tiến trình con mà F.Auto khởi chạy. Đừng dán key vào đoạn chat mà bạn lưu hoặc chia sẻ.

## 7. Cần trợ giúp

- Hướng dẫn sử dụng đầy đủ tiếng Việt: [`docs/usetut_vi.md`](usetut_vi.md) — mọi tính năng, từ lúc mở
  lên tới các lệnh nâng cao.
- Bản tham chiếu tiếng Anh: [`docs/REFERENCE.md`](REFERENCE.md) — toàn bộ lệnh, MCP, công cụ trình
  duyệt, tự host (Telegram/Discord/Docker/Kubernetes), bộ nhớ, kỹ năng.
- Mô hình an toàn chi tiết: [`docs/SANDBOX.md`](SANDBOX.md) — sàn chặn lệnh cứng, tầng duyệt lệnh, và
  sandbox cấp hệ điều hành (`fauto sandbox status` cho biết máy *của bạn* thực thi được gì).
- Báo lỗi hoặc đề xuất tính năng: mở issue tại
  [`Huutantv/factory-ai-main_v2`](https://github.com/Huutantv/factory-ai-main_v2/issues).

---

## Gỡ cài đặt

Xóa binary và, nếu muốn sạch hoàn toàn, xóa cả thư mục dữ liệu.

| hệ điều hành | binary | dữ liệu |
|---|---|---|
| Windows | xóa `%LOCALAPPDATA%\F.Auto` và mục PATH tương ứng | xóa `%USERPROFILE%\.aizen` |
| Linux / macOS | `rm -rf ~/.fauto` và bỏ dòng `export PATH` | `rm -rf ~/.aizen` |

`~/.aizen` chứa cấu hình, các phiên đã lưu, bộ nhớ và kỹ năng — xóa là không thể khôi phục, nên sao
lưu trước nếu bạn có thể cài lại.
