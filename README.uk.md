<div align="center">
  <img src="https://raw.githubusercontent.com/a4ivi401/lumina-remote/main/docs/assets/logo.png" alt="LuminaRemote Logo" width="120" />
  
  # LuminaRemote
  
  **Високопродуктивне програмне забезпечення для віддаленого доступу**
  
  [🇷🇺 Русский](README.ru.md) | [🇺🇦 Українська](README.uk.md) | [🇬🇧 English](README.md)

  <br />

  ![Rust](https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white)
  ![Tauri](https://img.shields.io/badge/Tauri-24C8DB?style=for-the-badge&logo=tauri&logoColor=FFFFFF)
  ![React](https://img.shields.io/badge/React-20232A?style=for-the-badge&logo=react&logoColor=61DAFB)
  ![TypeScript](https://img.shields.io/badge/TypeScript-007ACC?style=for-the-badge&logo=typescript&logoColor=white)
  ![QUIC](https://img.shields.io/badge/Protocol-QUIC%20%2F%20WebRTC-blue?style=for-the-badge)
  ![macOS](https://img.shields.io/badge/macOS-000000?style=for-the-badge&logo=apple&logoColor=white)
  ![Windows](https://img.shields.io/badge/Windows-0078D6?style=for-the-badge&logo=windows&logoColor=white)
  ![Linux](https://img.shields.io/badge/Linux-FCC624?style=for-the-badge&logo=linux&logoColor=black)

</div>

<br />

## 🌟 Про проєкт

**LuminaRemote** — це передовий додаток для віддаленого доступу до робочого столу, розроблений з нуля з фокусом на максимальну швидкість, безпеку та мінімальне споживання ресурсів.

На відміну від важких Electron-додатків (таких як TeamViewer або AnyDesk), LuminaRemote побудований на базі **Tauri** та **Rust**. Він використовує нативні API операційної системи для захоплення екрану (DXGI для Windows, CoreGraphics для macOS, X11/Wayland для Linux) і передає дані через сучасний **P2P-тунель на базі QUIC**, оминаючи обмеження веб-браузерів.

## ✨ Ключові особливості

- ⚡ **Ультра-низька затримка:** Написано повністю на Rust для максимальної продуктивності. Апаратне кодування відео VP9 забезпечує плавну трансляцію при 60FPS.
- 🛡️ **Військова криптографія:** Справжнє наскрізне шифрування (End-to-End Encryption) з використанням QUIC та TLS 1.3. Взаємна автентифікація HMAC-SHA256 із прив'язкою до каналу (Channel Binding) виключає атаки Man-In-The-Middle.
- 🌐 **Підключення без налаштувань:** Легко з'єднує комп'ютери через континенти завдяки технології пробиття NAT (STUN/TURN). Якщо обидва пристрої знаходяться в одній Wi-Fi мережі, програма миттєво перемикається на прямий пошук **mDNS** для затримки в 0мс.
- 🎨 **Сучасний Cyberpunk UI:** Стильний інтерфейс на React, натхненний гласморфізмом. Жодних лагів та мінімальне споживання оперативної пам'яті.
- 🔌 **Неконтрольований доступ (Unattended Access):** Безпечне збереження довірених машин локально для миттєвого доступу у фоновому режимі в один клік.

## 🏗️ Архітектура

LuminaRemote складається з двох головних компонентів:
1. **Клієнт (Tauri):** Додаток, який встановлюється на ваш комп'ютер. Він відповідає за красивий інтерфейс, нативне захоплення екрану, передачу натискань клавіатури та пряме P2P-з'єднання.
2. **Сигнальний сервер (Axum):** Легковаговий WebSocket-сервер, який використовується *тільки* для обміну початковими метаданими (IP-адресами) для встановлення прямого P2P тунелю між клієнтами. **Відеодані ніколи не проходять через сервер.**

## 🚀 Встановлення та Використання

1. Перейдіть на вкладку [Releases](../../releases).
2. Завантажте інсталятор для вашої операційної системи (`.exe` для Windows, `.dmg` для macOS, `.deb` для Linux).
3. Відкрийте додаток. Ви побачите свій унікальний **Device ID**.
4. Повідомте свій ID та згенерований **Session PIN** партнеру.
5. Партнер вводить ці дані та миттєво підключається до вашого робочого столу!

## 🛠️ Для Розробників

Бажаєте зібрати LuminaRemote з вихідного коду?

### Вимоги
- [Rust](https://rustup.rs/) (остання стабільна версія)
- [Node.js](https://nodejs.org/) (v20+)
- Системні залежності для збірки (наприклад, `libwebkit2gtk-4.1-dev` на Ubuntu).

### Локальний запуск
```bash
# Клонуємо репозиторій
git clone https://github.com/a4ivi401/lumina-remote.git
cd lumina-remote/client/lumina-ui

# Встановлюємо залежності фронтенду
npm install

# Запускаємо локальний сервер розробки Tauri
npm run tauri dev
```

## 📜 Ліцензія
Розповсюджується під ліцензією MIT. Детальніше дивіться у файлі `LICENSE`.
