#!/bin/sh

# UCI System - Universal Start Script
# Compatible with Linux, macOS, BSD and Windows (via Git Bash/WSL)

set -e

echo "🚀 Iniciando UCI System Portability Mode..."

# Detectar Docker
if command -v docker-compose >/dev/null 2>&1 || docker compose version >/dev/null 2>&1; then
    echo "🐳 Docker detectado. Iniciando con Docker Compose..."
    docker-compose up --build -d
    echo "✅ Sistema iniciado en http://localhost:3000"
    echo "📝 Usa 'docker-compose logs -f' para ver los registros."
else
    echo "⚠️ Docker no detectado. Intentando inicio nativo..."
    
    # Verificar base de datos local
    if ! command -v surreal >/dev/null 2>&1; then
        echo "❌ Error: SurrealDB no está instalado localmente y Docker no está disponible."
        exit 1
    fi
    
    # Iniciar DB en segundo plano si no está corriendo
    if ! curl -s http://localhost:8000/health >/dev/null; then
        echo "💾 Iniciando SurrealDB local..."
        surreal start --user root --pass root file:uci.db > surreal.log 2>&1 &
        sleep 2
    fi
    
    # Iniciar app
    echo "⚙️ Iniciando backend..."
    cargo run --release --bin uci-server
fi
