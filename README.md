# ARTCRAFT-TOAN (dự án cá nhân)

Build Android (APK) cá nhân cho 3 ứng dụng:

| App | Package | Mã nguồn gốc |
|---|---|---|
| WordCraft | `com.toan.wordcraft` | Fork riêng: https://github.com/BongBang104/wordcraft (nhánh `toan`) |
| GridCraft | `com.toan.gridcraft` | Fork riêng: https://github.com/BongBang104/gridcraft (nhánh `toan`) |
| PdfCraft | `com.toan.pdfcraft` | https://github.com/storytold/pdfcraft |

Dựa trên hệ thống build của [MostafaXS1/artcraft-android](https://github.com/MostafaXS1/artcraft-android) (MIT).
README gốc giữ tại [`README.upstream.md`](README.upstream.md).

> Bản không chính thức, dùng cá nhân. Không liên kết hay được bảo trợ bởi ArtCraft / Storytold.
> Tên và icon "ArtCraft", "…Craft" là thương hiệu của ArtCraft Team.

## Khác biệt so với bản gốc

- Workflow chỉ build **WordCraft, GridCraft, PdfCraft**; không build Launcher.
- **Push thường không tự build** (tiết kiệm phút GitHub Actions).
- Package mặc định `com.toan.<app>` → cài song song được với bản ArtCraft khác.
- Ký APK bằng **khoá cố định** lấy từ GitHub Secrets → bản mới cài đè (cập nhật) được lên bản cũ.
- Thông báo cập nhật trong app trỏ về Releases của repo này.
- Bỏ file rác `coiman-ide.strace` (36 MB) và 2 workflow không cần (contributors, security scan).

## Cách build trên GitHub (không cần cài gì trên máy)

1. Tab **Actions** → **Build APKs** → **Run workflow**.
2. `apps`: `wordcraft, gridcraft, pdfcraft` (hoặc `all`, hoặc 1 app). `source_version`: để **stable**.
3. Chờ build xong (~20–35 phút/app, chạy song song) → tải APK ở mục **Artifacts** cuối trang run.

Cách khác:
- Commit có chứa `[build: wordcraft]` (hoặc `[build: all]`) sẽ tự build app đó.
- Push tag dạng `v1.0.0` → build cả 3 app và tạo **Release** (app sẽ nhận thông báo cập nhật).

## Secrets bắt buộc (Settings → Secrets and variables → Actions)

| Tên | Nội dung |
|---|---|
| `ANDROID_KEYSTORE_B64` | File keystore `.jks` mã hoá base64 |
| `ANDROID_KEYSTORE_PASSWORD` | Mật khẩu keystore (alias: `artcraftbuilder`) |

**Giữ bản sao keystore + mật khẩu ở nơi an toàn.** Mất khoá thì không cài đè bản cập nhật được nữa (phải gỡ app, mất dữ liệu app).

## Build trên máy (Linux)

```bash
bash build.sh -wordcraft      # hoặc -gridcraft, -pdfcraft
# APK nằm trong output/
```

Cần Rust (target `aarch64-linux-android`), Android NDK 27, JDK 21, `cargo-ndk`, Python 3 + Pillow.

## Phát triển tính năng riêng

- **WordCraft, GridCraft** build từ fork riêng, nhánh **`toan`** (tách từ bản ổn định WordCraft v0.18.1, GridCraft v0.19.1).
  Sửa tính năng = commit vào nhánh `toan` của fork, rồi build lại ở repo này.
  Chọn `source_version: stable` khi chạy workflow = build nhánh `toan`; `main` = bản mới nhất của fork.
- **PdfCraft** vẫn lấy từ repo gốc `storytold/pdfcraft`. Muốn sửa thì fork tương tự và thêm vào khối
  `PERSONAL_FORK_BRANCH` trong `lib/build-common.sh`.

## Giấy phép

Phần builder: MIT (xem `LICENSE`). Các app gốc: MIT OR Apache-2.0 (xem `APP_LICENSES.csv`).
