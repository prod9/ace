```
░█▀█░█▀▀░█▀▀
░█▀█░█░░░█▀▀
░▀░▀░▀▀▀░▀▀▀
```

**ACE** (Augmented Coding Environment) — เครื่องมืออัตโนมัติสำหรับตั้งค่าและดูแลสภาพแวดล้อม AI coding
ให้พร้อมใช้งานอยู่เสมอ ทำหน้าที่เป็นจุดเริ่มต้นสู่ [Claude Code](https://docs.anthropic.com/en/docs/claude-code)
หรือ [OpenCode](https://github.com/opencode-ai/opencode)

## ติดตั้ง

```sh
cargo install --path .
```

## การใช้งาน

```sh
ace setup prod9/school                       # โคลน school, ลงทะเบียน MCP, เขียน config
ace                                          # เปิด backend (claude/opencode)
ace -- --continue                            # ส่ง flag ต่อไปยัง backend
ace import anthropics/skills --skill commit  # นำเข้า skill จาก repo ภายนอก
ace school update                            # ดึง skill ที่นำเข้าทั้งหมดใหม่
```

## คำสั่ง

| คำสั่ง | รายละเอียด |
|--------|-----------|
| `ace setup [specifier]` | โคลน school, ลงทะเบียน MCP servers, เขียน config |
| `ace config` | แสดง configuration ที่ resolve แล้ว |
| `ace paths [key]` | แสดง path ของระบบไฟล์ที่ resolve แล้ว (เช่น `ace paths school`) |
| `ace import <source> [--skill <name>]` | นำเข้า skill จาก repository ภายนอก |
| `ace school init` | สร้าง school repository ใหม่ |
| `ace school update` | ดึง skill ที่นำเข้าทั้งหมดใหม่จากแหล่งที่มา |
| `ace diff` | แสดงการเปลี่ยนแปลงที่ยังไม่ commit ใน school cache |

## วิธีการทำงาน

ACE จัดการ **schools** — repository ที่แชร์ร่วมกัน ประกอบด้วย skills, rules, commands, agents
และ configuration สำหรับเครื่องมือ AI coding เมื่อรัน `ace` จะ:

1. หา school ที่จะใช้ (จาก `ace.toml`)
2. ดึง/อัปเดต school repository
3. สร้าง symlink ของโฟลเดอร์ school เข้าไปในโปรเจกต์
4. เปิด backend ที่ตั้งค่าไว้พร้อม session prompt ของ school

## Configuration

- `ace.toml` — config ระดับโปรเจกต์ (school specifier, backend, env)
- `ace.local.toml` — override เฉพาะเครื่อง (gitignored)
- `~/.config/ace/config.toml` — config ระดับผู้ใช้ (credentials)
- `school.toml` — ข้อมูล school (ชื่อ, MCP servers, projects)

## Development

```sh
cargo test              # unit tests + integration tests (ไม่ต้องใช้ network)
cargo test --test setup_test  # รันไฟล์ test เดียว
```

Integration tests อยู่ใน `tests/` ใช้ `TestEnv` (tempdir sandbox + `assert_cmd`) แต่ละไฟล์
ทดสอบ CLI command เดียว Tests ที่ต้องใช้ network (clone) ยังไม่รองรับ — ดู ROADMAP

## Cross-build

Build สำหรับ linux/mac × arm64/amd64 โดย target ของเครื่องใช้ `cargo` ส่วน target อื่นใช้
[`cross`](https://github.com/cross-rs/cross) (ใช้ Docker)

ต้องมี: Docker, `cargo install cross`, Rust stable toolchain

```sh
./build-all.sh            # output ไปที่ target/dist/
./build-all.sh out/       # กำหนด output dir เอง
```

`ureq` ใช้ `rustls` (pure Rust TLS) จึงไม่ต้องพึ่ง OpenSSL ของระบบ

## License

MIT
