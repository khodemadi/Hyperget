# Hyper Get

![آیکن Hyper Get](apps/desktop/src-tauri/icons/128x128.png)

**دانلود سریع و پایدار، بدون وابستگی به مرورگر.** Hyper Get یک دانلود منیجر متن‌باز برای لینوکس و ویندوز است که با Rust، SQLite، Tauri 2 و React ساخته شده است.

> نسخهٔ فعلی: **v0.2.2-alpha.1**

## قابلیت‌ها

- صف پایدار با اولویت‌بندی و تعداد دانلود هم‌زمان قابل تنظیم
- Pause، Resume، بازیابی پس از اجرای مجدد و ادامهٔ فایل نیمه‌کاره
- پیشرفت واقعی و وزنی بر اساس بایت، سرعت ترکیبی و زمان باقی‌مانده
- محدودیت سرعت سراسری و جداگانه برای هر دانلود
- بررسی SHA-256 و تغییر نام امن فایل‌های تکراری
- انتخاب پوشه با پنجرهٔ استاندارد سیستم‌عامل و ذخیرهٔ تنظیمات
- تشخیص خودکار دانلود دسته‌ای با واردکردن یک `*`
- دکمهٔ Clear all برای پاک‌کردن یکجای لیست با تأیید
- افزونه برای Chrome، Chromium، Edge و Firefox
- رابط روشن/تیره با مدیریت خطا و جلوگیری از سفیدشدن برنامه

## دریافت نسخه‌ها

| سیستم | فایل انتشار |
| --- | --- |
| Windows 10/11 x64 | نصب‌کنندهٔ NSIS با پسوند `.exe` و WiX با پسوند `.msi` |
| Linux x64 | AppImage و بستهٔ Debian با پسوند `.deb` |
| Chrome، Chromium و Edge | `hyper-get-chromium.zip` |
| Firefox | `hyper-get-firefox.zip` |

خروجی‌های Release دارای checksum از نوع SHA-256 هستند. این نسخه Alpha است؛ تا پایان بررسی دانلود، لینک منبع فایل‌های مهم را نگه دارید.

## اتصال افزونهٔ مرورگر

بعد از نصب برنامه، Native Host را نصب کنید:

```bash
# Linux
./scripts/install-browser-host-linux.sh
```

```powershell
# Windows PowerShell
.\scripts\install-browser-host-windows.ps1
```

سپس ZIP مناسب مرورگر را در حالت Developer/Unpacked نصب کنید. جزئیات در [راهنمای افزونه](extensions/browser/README.md) آمده است.

## توسعه و ساخت

```bash
corepack enable
pnpm install --frozen-lockfile
cargo test --workspace --all-features
pnpm lint && pnpm typecheck && pnpm test
pnpm --filter @hyper-get/desktop tauri dev
```

ساخت برنامه و افزونه‌ها:

```bash
pnpm --filter @hyper-get/desktop tauri build
pnpm browser:build
pnpm browser:package
```

ساختار پروژه:

- `crates/hyper-core`: هسته دانلود، scheduler و SQLite
- `crates/hyper-cli`: ابزار خط فرمان
- `crates/hyper-native-host`: ارتباط با افزونه
- `apps/desktop`: برنامه Tauri و React
- `extensions/browser`: افزونه‌های Chromium و Firefox

مستندات: [معماری](docs/ARCHITECTURE.md)، [تست](docs/TESTING.md)، [انتشار](docs/RELEASE.md)، [مشارکت](CONTRIBUTING.md) و [امنیت](SECURITY.md).

پروژه تحت [مجوز MIT](LICENSE) منتشر می‌شود.
