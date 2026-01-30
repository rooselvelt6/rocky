#!/bin/bash
# bin/zeus-start.sh - El Orquestador Supremo de UCI Scales
set -euo pipefail

# Colores ZEUS
GOLD='\033[1;33m'
CYAN='\033[0;36m'
GREEN='\033[0;32m'
RED='\033[0;31m'
NC='\033[0m'

echo -e "${GOLD}⚡ UCI SCALES - SISTEMA DE ARRANQUE ZEUS ⚡${NC}"
echo -e "${CYAN}----------------------------------------------------${NC}"

# 1. PERCEPCIÓN: Detección de Plataforma y Arquitectura
OS=$(uname -s)
ARCH=$(uname -m)
echo -e "🖥️  Entorno: ${GREEN}$OS ($ARCH)${NC}"

# 2. HEURÍSTICA DE CONEXIÓN
check_internet() {
    if ping -c 1 -W 2 1.1.1.1 &>/dev/null; then
        return 0 # Conectado
    else
        return 1 # Offline
    fi
}

# 3. CAPA DE DECISIÓN
if command -v docker &>/dev/null && docker info &>/dev/null; then
    echo -e "🐳 Docker Detectado y Activo. Lanzando Modo Robusto..."
    ./bin/start-robust.sh
    exit $?
fi

echo -e "⚠️  Docker no disponible o inactivo."

if check_internet; then
    echo -e "🌐 Conexión a Internet: ${GREEN}ACTIVA${NC}"
    echo -n -e "❓ ¿Deseas intentar instalar Docker automáticamente? (s/n): "
    read -r choice
    if [[ "$choice" =~ ^[Ss]$ ]]; then
        echo -e "${CYAN}📦 Iniciando Auto-Instalador de Docker Nivel Zeus...${NC}"
        # Aquí llamaríamos al instalador específico según el OS
        if [[ "$OS" == "Linux" ]]; then
            curl -fsSL https://get.docker.com | sh
            sudo usermod -aG docker $USER || true
            echo -e "${GREEN}✅ Docker instalado. Por favor, reinicia la sesión o ejecuta './bin/zeus-start.sh' de nuevo.${NC}"
            exit 0
        fi
    fi
fi

# 4. FALLBACK: MODO NATIVO (EL CORAZÓN DE ZEUS)
echo -e "${GOLD}🚀 Iniciando MODO ZEUS NATIVO (Binario Único + DB Embebida)${NC}"

# Verificar si el binario existe, si no, compilar
if ! [ -f "target/release/uci-server" ]; then
    echo -e "🛠️  Binario no encontrado. Forjando aplicación (compilación optimizada)..."
    if ! command -v cargo &>/dev/null; then
        echo -e "${RED}❌ Error: No se encontró 'cargo'. Instala Rust o Docker para continuar.${NC}"
        exit 1
    fi
    cargo build --release --features ssr
fi

# Configuración de Variables de Entorno para Modo Embebido
export DB_MODE="embedded"
export DB_PATH="rocksdb:uci_data"
export RUST_LOG="info"

# Saneamiento de procesos previos
pkill uci-server || true

echo -e "${GREEN}✅ ZEUS NATIVO EN MARCHA EN http://localhost:3000${NC}"
./target/release/uci-server
