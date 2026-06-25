<div align="center">
  <img src="https://raw.githubusercontent.com/a4ivi401/lumina-remote/main/docs/assets/logo.png" alt="LuminaRemote Logo" width="120" />
  
  # LuminaRemote
  
  **Высокопроизводительное программное обеспечение для удаленного доступа**
  
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

## 🌟 О проекте

**LuminaRemote** — это передовое приложение для удаленного доступа к рабочему столу, разработанное с нуля с фокусом на максимальную скорость, безопасность и минимальное потребление ресурсов.

В отличие от тяжелых Electron-приложений (таких как TeamViewer или AnyDesk), LuminaRemote построено на базе **Tauri** и **Rust**. Оно использует нативные API операционной системы для захвата экрана (DXGI для Windows, CoreGraphics для macOS, X11/Wayland для Linux) и передает данные через современный **P2P-туннель на базе QUIC**, обходя ограничения веб-браузеров.

## ✨ Ключевые особенности

- ⚡ **Ультра-низкая задержка:** Написано полностью на Rust для максимальной производительности. Аппаратное кодирование видео VP9 обеспечивает плавную трансляцию при 60FPS.
- 🛡️ **Военная криптография:** Настоящее сквозное шифрование (End-to-End Encryption) с использованием QUIC и TLS 1.3. Взаимная аутентификация HMAC-SHA256 с привязкой к каналу (Channel Binding) исключает атаки Man-In-The-Middle.
- 🌐 **Подключение без настроек:** Легко соединяет компьютеры через континенты благодаря технологии пробития NAT (STUN/TURN). Если оба устройства находятся в одной Wi-Fi сети, программа мгновенно переключается на прямой поиск **mDNS** для задержки в 0мс.
- 🎨 **Современный Cyberpunk UI:** Стильный, вдохновленный глассморфизмом интерфейс на React. Никаких лагов и минимальное потребление оперативной памяти.
- 🔌 **Неконтролируемый доступ (Unattended Access):** Безопасное сохранение доверенных машин локально для мгновенного доступа в фоновом режиме в один клик.

## 🏗️ Архитектура

LuminaRemote состоит из двух главных компонентов:
1. **Клиент (Tauri):** Приложение, которое устанавливается на ваш компьютер. Оно отвечает за красивый интерфейс, нативный захват экрана, передачу нажатий клавиатуры и прямое P2P-соединение.
2. **Сигнальный сервер (Axum):** Легковесный WebSocket-сервер, который используется *только* для обмена первоначальными метаданными (IP-адресами) для установки прямого P2P туннеля между клиентами. **Видеоданные никогда не проходят через сервер.**

## 🚀 Установка и Использование

1. Перейдите на вкладку [Releases](../../releases).
2. Скачайте инсталлятор для вашей операционной системы (`.exe` для Windows, `.dmg` для macOS, `.deb` для Linux).
3. Откройте приложение. Вы увидите свой уникальный **Device ID**.
4. Сообщите свой ID и сгенерированный **Session PIN** партнеру.
5. Партнер вводит эти данные и мгновенно подключается к вашему рабочему столу!

## 🛠️ Для Разработчиков

Хотите собрать LuminaRemote из исходного кода?

### Требования
- [Rust](https://rustup.rs/) (последняя стабильная версия)
- [Node.js](https://nodejs.org/) (v20+)
- Системные зависимости для сборки (например, `libwebkit2gtk-4.1-dev` на Ubuntu).

### Локальный запуск
```bash
# Клонируем репозиторий
git clone https://github.com/a4ivi401/lumina-remote.git
cd lumina-remote/client/lumina-ui

# Устанавливаем зависимости фронтенда
npm install

# Запускаем локальный сервер разработки Tauri
npm run tauri dev
```

## 📜 Лицензия
Распространяется под лицензией MIT. Подробнее смотрите в файле `LICENSE`.
