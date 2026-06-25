# LuminaRemote: Архітектура

Цей документ описує модульну архітектуру проєкту LuminaRemote, призначення компонентів та їх взаємодію.

## 1. Огляд Компонентів (C4 Model)

LuminaRemote працює як гібридний P2P додаток. Усі Rust компоненти об'єднані в один Cargo workspace.

```mermaid
graph TD
    subgraph Client UI ["UI Шар (React / Vue)"]
        A(Графічний інтерфейс)
    end
    
    subgraph Tauri Backend ["Tauri Обгортка (src-tauri)"]
        B(Зв'язуючий IPC шар)
    end
    
    subgraph Rust Crates ["Ядро застосунку (Rust)"]
        Core[lumina-core]
        Proto[lumina-protocol]
        Net[lumina-network]
        Capture[lumina-capture]
        Encode[lumina-encoder]
        Input[lumina-input]
    end
    
    subgraph External Infra ["Інфраструктура"]
        SigServer(Lumina Signal Server)
        Relay(Lumina Relay Server)
    end
    
    A <-->|Tauri IPC| B
    B --> Core
    B --> Net
    B --> Capture
    B --> Encode
    B --> Input
    
    Net <-->|WebSocket| SigServer
    Net <-->|QUIC| Relay
    Net <-->|QUIC (P2P)| Net
```

## 2. Опис Модулів (Crates)

* **`lumina-core`**: Базові типи, конфігурація, криптографія (Argon2id, X25519).
* **`lumina-protocol`**: Серіалізація та десеріалізація структур мережевого спілкування (кадри, команди).
* **`lumina-network`**: Керування QUIC-з'єднаннями, LAN-виявлення (mDNS), NAT Traversal.
* **`lumina-capture`**: Абстракція для захоплення відео з екрану.
* **`lumina-encoder`**: Кодування кадрів у відеопотік (H.264).
* **`lumina-input`**: Емуляція введення (клавіатура, миша).
* **`lumina-signal-server`**: Сервер для обміну ключами та IP-адресами.

## 3. Zero-Trust Безпека

```mermaid
sequenceDiagram
    participant Host
    participant Signal as Signal Server
    participant Client
    
    Host->>Host: Генерація Seed (12 симв.)
    Host->>Host: Argon2(Seed) -> X25519 Ключі
    Host->>Signal: Реєстрація (PubKey_H)
    
    Note over Client: Користувач вводить 12 символів
    
    Client->>Client: Argon2(Seed) -> X25519 Ключі
    
    Client->>Signal: Запит на доступ (PubKey_H)
    Signal-->>Client: Віддає IP Хоста
    
    Client->>Host: QUIC Handshake
    
    Host->>Host: ECDH(PrivKey_H, PubKey_C) -> SharedSecret
    Client->>Client: ECDH(PrivKey_C, PubKey_H) -> SharedSecret
    
    Host<-->>Client: Захищене QUIC з'єднання (TLS 1.3 PSK)
```
