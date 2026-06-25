#!/bin/bash

echo "🚀 Начинаем развертывание Lumina Signal Server..."

# 1. Проверяем наличие Docker
if ! command -v docker &> /dev/null; then
    echo "Устанавливаем Docker..."
    sudo apt-get update
    sudo apt-get install -y docker.io docker-compose
else
    echo "✅ Docker уже установлен."
fi

# 2. Переходим в директорию сигнала
cd "$(dirname "$0")/lumina-signal-server"

# 3. Собираем и запускаем контейнер
echo "⏳ Собираем Rust-бинарник и запускаем сервер (может занять несколько минут)..."
sudo docker-compose up -d --build

echo "✅ Сервер успешно запущен на порту 3000!"
echo "Для просмотра логов используйте команду: sudo docker-compose logs -f"
