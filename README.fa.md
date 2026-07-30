# Hyper Get

پایین داشبورد اکنون فقط دو لایه دارد: نوار پیشرفت سراسری واقعی با محاسبهٔ وزنی بر اساس بایت‌ها، و نوار وضعیت فشرده. ورودی Quick Download حذف شده و افزودن آدرس از Add URL، میانبر Ctrl+V، افزونهٔ مرورگر و پنجرهٔ Batch انجام می‌شود.

Pause All ابتدا وضعیت پایدار اجرای صف را غیرفعال می‌کند تا scheduler هیچ مورد queued را شروع نکند؛ Start All دوباره صف را فعال می‌کند. مسیر پیش‌فرض دانلود توسط Tauri از سیستم‌عامل دریافت و مسیر انتخابی پیش از ذخیره اعتبارسنجی می‌شود.

یک دانلود منیجر سریع، قابل ادامه و چندسکویی برای لینوکس و ویندوز که با Rust و Tauri ساخته می‌شود.

Hyper Get قرار است امکانات کاربردی دانلود منیجرهایی مانند IDM را با معماری مدرن، متن‌باز و چندسکویی ارائه کند. هسته دانلود با Rust نوشته می‌شود و بین برنامه دسکتاپ، CLI و افزونه‌های مرورگر مشترک خواهد بود.

> وضعیت پروژه: نسخه `v0.1.0-alpha` در حال توسعه است. هسته واردشده منطق مفیدی برای دانلود دارد، اما در وضعیت فعلی هنوز Build کامل، تست‌شده و آماده Release نیست. هیچ قابلیتی تا زمان عبور از تست‌های خودکار و دستی «کامل» اعلام نمی‌شود.

## هدف اصلی

- دانلود پایدار فایل‌های بزرگ.
- ادامه دانلود پس از قطع اینترنت، بسته‌شدن برنامه یا ری‌استارت سیستم.
- دانلود چندبخشی با HTTP Range.
- بازگشت امن به دانلود تک‌اتصالی در سرورهای بدون Range.
- صف پایدار و قابل مرتب‌سازی.
- رابط دسکتاپ مدرن با Tauri.
- CLI واقعی که از همان هسته استفاده کند.
- افزونه سبک برای Chrome، Chromium، Edge و Firefox.
- انتشار برای Linux و Windows.
- جلوگیری از خرابی یا از بین رفتن دانلودهای نیمه‌کاره.

## وضعیت هسته فعلی

هسته فعلی شامل بخش‌هایی از قابلیت‌های زیر است:

- Tokio و Reqwest برای دانلود Async.
- درخواست HTTP Range.
- تقسیم فایل به Segment.
- دانلود هم‌زمان Segmentها.
- Retry با Exponential Backoff.
- فایل‌های موقت `.part`.
- Manifest مبتنی بر JSON.
- Merge کردن Segmentها.
- بررسی اختیاری SHA-256.
- Fallback به دانلود تک‌اتصالی.
- جداسازی نسبی Core از CLI و GUI.

مشکلات فعلی که باید قبل از Release رفع شوند:

- فایل ماژول `segment` وجود ندارد.
- importهای Serde در `types.rs` ناقص‌اند.
- `StarLevel` traitهای مورد نیاز را ندارد.
- ساختار `crs/` استاندارد نیست.
- Progress به شکل API رویداد پایدار در اختیار UI نیست.
- وضعیت هر Segment بعد از تکمیل به‌صورت اتمیک ذخیره نمی‌شود.
- اعتبار Resume با `ETag` و `Last-Modified` بررسی نمی‌شود.
- اتکا به `HEAD` زیاد است و باید با `Range: bytes=0-0` تکمیل شود.
- Pause و Cancel واقعی با Cancellation Token وجود ندارد.
- SQLite، Migration، Queue و بازیابی پس از Crash کامل نیست.
- CI، تست، بسته‌بندی و اسناد GitHub ناقص‌اند.

نسخه اول باید این مشکلات را با حفظ منطق مفید هسته اصلاح کند.

## معماری هدف

```text
hyper-get/
├── crates/
│   ├── hyper-core/       هسته دانلود، صف، دیتابیس و رویدادها
│   └── hyper-cli/        رابط خط فرمان
├── apps/
│   └── desktop/
│       ├── src/          React + TypeScript + Tailwind
│       └── src-tauri/    دستورات Tauri و اتصال به Core
├── extensions/
│   └── browser/          افزونه مرورگر
├── docs/
└── .github/
```

قانون اصلی معماری:

- `hyper-core` نباید به Tauri، React یا افزونه مرورگر وابسته باشد.
- Desktop و CLI باید دقیقاً از یک Core استفاده کنند.
- UI نباید داده دانلود ساختگی داشته باشد.
- کارهای شبکه، فایل و Hash باید در Backend انجام شوند.
- وضعیت‌ها و رویدادهای Core باید Serializable باشند.

## رابط دسکتاپ

رابط با Tauri 2، React، TypeScript، Vite و Tailwind ساخته می‌شود.

ساختار اصلی:

```text
┌─────────────────────────────────────────────────────────────────┐
│ جستجو | افزودن لینک | شروع همه | توقف همه | تنظیمات             │
├──────────────┬──────────────────────────────────────────────────┤
│ همه          │ file.iso       63%   18.4 MB/s   04:31          │
│ فعال         │ ████████████░░░░░░░                               │
│ صف           │                                                  │
│ متوقف        │ archive.zip    Waiting                            │
│ کامل‌شده     │                                                  │
│ خطادار       │ video.mp4      Paused at 41%                      │
├──────────────┴──────────────────────────────────────────────────┤
│ پیشرفت کل 47% | 3 فعال | 8 در صف | 32.8 MB/s | محدودیت: خاموش  │
└─────────────────────────────────────────────────────────────────┘
```

هر دانلود باید نمایش دهد:

- نام فایل.
- دامنه منبع.
- نوار پیشرفت.
- حجم دانلودشده و حجم کل.
- سرعت.
- زمان باقی‌مانده.
- وضعیت.
- عملیات سریع.
- اولویت و جایگاه صف.

نوار پایین باید با داده واقعی نشان دهد:

- پیشرفت کل بر اساس مجموع Byteها.
- سرعت کل.
- تعداد فعال، در صف، متوقف، کامل‌شده و خطادار.
- محدودیت سرعت کل.

میانگین درصد فایل‌ها نباید به‌عنوان پیشرفت کل استفاده شود.

## رفتارهای ضروری UI

- Add URL.
- افزودن چند لینک.
- Start، Pause، Resume، Restart، Cancel و Remove.
- Drag & Drop برای مرتب‌سازی صف.
- انتخاب چندتایی.
- Context Menu.
- جستجو و فیلتر.
- Dark و Light Theme.
- پشتیبانی از پنجره باریک.
- میانبرهای صفحه‌کلید.
- نمایش جزئیات خطا.
- تفاوت واضح بین حذف از لیست و حذف فایل.
- کلید `*` برای بازکردن عملیات دانلود همه لینک‌های دریافت‌شده از مرورگر.
- Drag کردن URL داخل پنجره.
- تشخیص Clipboard فقط به‌صورت اختیاری.

## Core

Core باید وضعیت‌های مشخص و قابل کنترل داشته باشد:

```text
Created
Resolving
Queued
Connecting
Downloading
Pausing
Paused
RetryWaiting
Merging
Verifying
Completed
Failed
Cancelled
```

Resume باید:

- اندازه فایل‌های موقت را بررسی کند.
- مشخصات فایل Remote را با `ETag` و `Last-Modified` اعتبارسنجی کند.
- از نوشتن تکراری Segmentهای کامل جلوگیری کند.
- در صورت تغییر فایل Remote، Resume ناامن را متوقف کند.
- تغییرات وضعیت را با Transaction ذخیره کند.

Pause باید:

- درخواست‌های جدید را متوقف کند.
- فایل‌ها را Flush کند.
- Progress را ذخیره کند.
- فایل‌های موقت قابل استفاده را نگه دارد.
- وضعیت نهایی `Paused` ارسال کند.

Cancel و Remove نباید یک معنی داشته باشند.

## CLI

نام باینری:

```bash
hyper-get
```

نمونه دستورات:

```bash
hyper-get add "https://example.com/file.iso"
hyper-get list
hyper-get status
hyper-get pause <id>
hyper-get resume <id>
hyper-get cancel <id>
hyper-get remove <id>
hyper-get start-all
hyper-get pause-all
hyper-get limit global 20MiB
hyper-get queue move <id> --before <other-id>
```

CLI باید:

- خروجی خوانا داشته باشد.
- حالت `--json` برای اسکریپت‌ها داشته باشد.
- روی خطا Exit Code غیرصفر بدهد.
- همان Core برنامه دسکتاپ را استفاده کند.

## افزونه مرورگر

پس از پایدارشدن Desktop ساخته می‌شود:

- Chrome، Chromium و Edge با Manifest V3.
- نسخه Firefox.
- Download with Hyper Get.
- Download all links with Hyper Get.
- ارسال لینک‌های انتخاب‌شده.
- Popup برای وضعیت اتصال.
- تنظیمات Allow/Deny برای سایت‌ها.
- Capture اختیاری.

ارتباط مرورگر باید امن باشد:

- فقط روی Loopback.
- Token تصادفی نصب.
- اعتبارسنجی Origin.
- محدودیت اندازه درخواست.
- جلوگیری از دریافت Path دلخواه از صفحات.
- امکان تعویض Token.
- مستند Threat Model.
- بررسی Native Messaging به‌عنوان گزینه امن‌تر.

## دیتابیس

SQLite حداقل باید این داده‌ها را ذخیره کند:

- Downloads.
- Segments.
- Settings.
- Migration version.
- وضعیت صف.
- Progress.
- مشخصات Remote.
- خطاها.
- تاریخ ایجاد و تکمیل.

بعد از بسته‌شدن برنامه، صف و دانلودهای نیمه‌کاره باید بازیابی شوند.

## تست‌های اجباری

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
pnpm lint
pnpm typecheck
pnpm test
pnpm build
```

تست‌های Core:

- فایل کوچک.
- فایل بزرگ با Range.
- سرور بدون Range.
- قطع اینترنت.
- بسته‌شدن اجباری برنامه.
- Resume بعد از اجرا.
- تغییر فایل Remote.
- Redirect.
- محتوای بدون Content-Length.
- SHA-256 صحیح و اشتباه.
- Merge صحیح Segmentها.
- Pause و Resume چندباره.
- Cancel و بازیابی وضعیت.
- جلوگیری از Path Traversal.
- نام فایل تکراری.

تا جای ممکن تست‌ها باید با HTTP Server محلی و قابل تکرار اجرا شوند، نه URL عمومی ناپایدار.

## نقشه نسخه‌ها

### `v0.1.0-alpha`

- اصلاح و انتقال Core فعلی.
- Cargo Workspace.
- Tauri 2.
- Add، Start، Pause، Resume، Cancel و Remove واقعی.
- SQLite و Queue پایه.
- دانلود تک‌اتصالی و چندبخشی.
- Retry و بازیابی پس از Restart.
- Progress واقعی هر فایل.
- نوار وضعیت واقعی پایین برنامه.
- CLI مشترک با Core.
- Dark و Light.
- تست Core و HTTP محلی.
- Build توسعه‌ای Linux.
- آماده‌سازی کامل GitHub و CI.

### `v0.2.0-alpha`

- Drag & Drop کامل صف.
- تعداد دانلود هم‌زمان قابل تنظیم.
- Limit کلی و جداگانه.
- Speed و ETA دقیق‌تر.
- دسته‌بندی و History.
- Scheduler پایه.

### `v0.3.0-alpha`

- افزونه Chrome/Chromium/Edge.
- نسخه Firefox.
- پروتکل امن محلی.
- دانلود تک‌لینک و همه لینک‌ها.
- کلید `*`.

### `v0.4.0-beta`

- Notification.
- Clipboard Monitor.
- Start on login.
- عملیات بعد از دانلود.
- Proxy و Authentication.
- آماده‌سازی Release پایدار Windows و Linux.

### `v1.0.0`

- بدون باگ شناخته‌شده از نوع ازبین‌رفتن داده.
- Crash Recovery قابل اعتماد.
- Migration پایدار.
- CLI و پروتکل افزونه پایدار.
- Release Asset همراه SHA-256.
- تست نهایی Linux و Windows.

## آماده‌سازی GitHub

ریپو باید شامل این موارد باشد:

- `README.md`
- `README.fa.md`
- `LICENSE`
- `CHANGELOG.md`
- `CONTRIBUTING.md`
- `SECURITY.md`
- `CODE_OF_CONDUCT.md`
- `.gitignore`
- `.editorconfig`
- مستندات Architecture، Core، UI، Testing و Release
- Issue Template
- Pull Request Template
- GitHub Actions CI
- GitHub Actions Release
- Dependabot
- Lockfileهای قابل تکرار
- عدم وجود Secret، Token، دیتابیس محلی، فایل دانلودی و پوشه Build
- Source Archive تمیز
- SHA-256 برای فایل‌های Release
- Tag استاندارد مثل `v0.1.0-alpha.1`
- Release Notes شامل قابلیت‌های واقعی، محدودیت‌ها و نتیجه تست

وقتی Format، Clippy، تست‌ها، Type Check یا Build شکست می‌خورد، Release نباید ساخته شود.

## محدودیت امنیتی و قانونی

Hyper Get فقط باید منابعی را دانلود کند که کاربر مجاز به دریافت آن‌هاست.

پروژه نباید:

- DRM را دور بزند.
- Authentication یا Paywall را دور بزند.
- Cookie یا Credential سرقت کند.
- کنترل‌های امنیتی سیستم یا مرورگر را غیرفعال کند.
- فایل دانلودشده را بدون اجازه مستقیم کاربر اجرا کند.

## مجوز

MIT License.

## خروجی ویندوز

Workflow با نام `Release installers` روی runner واقعی ویندوز، نصب‌کننده‌های NSIS با پسوند `.exe` و WiX با پسوند `.msi` را همراه `SHA256SUMS-windows.txt` می‌سازد. برای دریافت خروجی، یک Tag مثل `v0.1.0-alpha.1` Push کنید یا Workflow را دستی اجرا کنید و Artifact با نام `hyper-get-windows-*` را دانلود کنید. فایل ویندوز از محیط لینوکس قابل تأیید نهایی نیست.
