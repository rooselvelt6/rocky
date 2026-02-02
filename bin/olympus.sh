#!/bin/bash
# 🏛️ OLYMPUS SOVEREIGN TOOL (v10) - La Luz Abyssal

# Colores para la gloria del Olimpo
GOLD='\033[0;33m'
BLUE='\033[0;34m'
RED='\033[0;31m'
NC='\033[0m' # No Color

SERVER_BIN="./target/debug/uci-server" # Ajustar a release si es necesario

function show_usage() {
    echo -e "${GOLD}Uso: $0 {start|setup|watchdog|health|clean|debug}${NC}"
    echo "  start    - Inicia el servidor y la DB (Modo Soberano)"
    echo "  setup    - Detecta el OS e instala dependencias faltantes"
    echo "  watchdog - Inicia el vigilante Aura"
    echo "  health   - Comprueba la salud de los dioses"
    echo "  clean    - Forzar limpieza de Demeter (Filesystem)"
    echo "  debug    - Inicia con logs extendidos"
}

function install_dependencies() {
    echo -e "${GOLD}🏹 Detectando sistema operativo...${NC}"
    if [ -f /etc/fedora-release ]; then
        echo -e "${BLUE}Detected Fedora. Installing development tools...${NC}"
        sudo dnf groupinstall -y "Development Tools" "C Development Tools and Libraries"
        sudo dnf install -y gcc gcc-c++
    elif [ -f /etc/debian_version ]; then
        echo -e "${BLUE}Detected Debian/Ubuntu. Installing build-essential...${NC}"
        sudo apt update && sudo apt install -y build-essential
    else
        echo -e "${RED}OS no reconocido automáticamente. Por favor, instala un compilador C manualmente.${NC}"
    fi
}

function start_server() {
    echo -e "${GOLD}🏛️  Iniciando Jerarquía Soberana v10...${NC}"
    # Verificar si cc existe
    if ! command -v cc &> /dev/null; then
        echo -e "${RED}Linker 'cc' no encontrado. Intentando autoinstalación...${NC}"
        install_dependencies
    fi
    cargo run --bin uci-server --features ssr
}

function run_watchdog() {
    echo -e "${BLUE}🏹 Aura: Vigilante activado.${NC}"
    while true; do
        if ! pgrep -f "uci-server" > /dev/null; then
            echo -e "${RED}🚨 Zeus ha caído! Aura resucitando al Olimpo...${NC}"
            start_server &
        fi
        sleep 5
    done
}

function health_check() {
    echo -e "${GOLD}⚖️  Comprobando pulso de los dioses...${NC}"
    if curl -s http://localhost:3000/api/health > /dev/null; then
        echo "✅ El Olimpo está respondiendo."
    else
        echo "❌ Los dioses no responden."
    fi
}

case "$1" in
    start)
        start_server
        ;;
    setup)
        install_dependencies
        ;;
    watchdog)
        run_watchdog
        ;;
    health)
        health_check
        ;;
    clean)
        echo "🌾 Demeter forzará limpieza en el próximo inicio."
        ;;
    debug)
        RUST_LOG=debug cargo run --features ssr
        ;;
    *)
        show_usage
        ;;
esac
