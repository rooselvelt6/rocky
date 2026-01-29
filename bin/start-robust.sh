#!/bin/bash
set -euo pipefail

# Colores para una salida premium
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

echo -e "${BLUE}====================================================${NC}"
echo -e "${BLUE}🚀 UCI System - Ultra-Robust Startup Optimizer${NC}"
echo -e "${BLUE}====================================================${NC}"

# 1. Verificaciones de Sistema
echo -e "${CYAN}[1/5] Verificando entorno de ejecución...${NC}"
if ! command -v docker &> /dev/null; then
    echo -e "${RED}❌ Error: Docker no está instalado.${NC}"
    exit 1
fi

if ! docker info &> /dev/null; then
    echo -e "${RED}❌ Error: El demonio de Docker no está corriendo.${NC}"
    exit 1
fi
echo -e "${GREEN}✅ Docker está listo.${NC}"

# 2. Limpieza y Preparación
echo -e "${CYAN}[2/5] Preparando contenedores...${NC}"
docker-compose down --remove-orphans &>/dev/null || true
echo -e "${GREEN}✅ Entorno limpio.${NC}"

# 3. Construcción y Arranque
echo -e "${CYAN}[3/5] Construyendo e iniciando servicios (esto puede tardar la primera vez)...${NC}"
docker-compose up -d --build

# 4. Espera Inteligente (Health Checks)
echo -e "${CYAN}[4/5] Verificando salud de los servicios...${NC}"

# Esperar SurrealDB
echo -n -e "${YELLOW}⏳ Esperando SurrealDB... ${NC}"
MAX_RETRIES=30
COUNT=0
until docker-compose exec -T surrealdb /surreal isready --conn http://localhost:8000 &>/dev/null; do
    echo -n "."
    sleep 2
    COUNT=$((COUNT + 1))
    if [ $COUNT -ge $MAX_RETRIES ]; then
        echo -e "\n${RED}❌ Timeout: SurrealDB no inició a tiempo.${NC}"
        docker-compose logs surrealdb
        exit 1
    fi
done
echo -e "${GREEN} ¡LISTO!${NC}"

# Esperar Aplicación
echo -n -e "${YELLOW}⏳ Esperando Aplicación UCI... ${NC}"
COUNT=0
until curl -sf http://localhost:3000/api/health &>/dev/null; do
    echo -n "."
    sleep 2
    COUNT=$((COUNT + 1))
    if [ $COUNT -ge $MAX_RETRIES ]; then
        echo -e "\n${RED}❌ Timeout: La aplicación no respondió.${NC}"
        docker-compose logs uci-app
        exit 1
    fi
done
echo -e "${GREEN} ¡LISTO!${NC}"

# 5. Inicialización de Datos
echo -e "${CYAN}[5/5] Finalizando configuración...${NC}"
if [ -f "db/schema.surql" ]; then
    echo -e "${YELLOW}📊 Importando esquema de base de datos...${NC}"
    docker-compose exec -T surrealdb /surreal import \
        --conn http://localhost:8000 \
        --user root --pass root \
        --ns hospital --db uci \
        /db/schema.surql || echo -e "${YELLOW}⚠️ Nota: El esquema podría ya existir.${NC}"
fi

echo -e "\n${GREEN}╔══════════════════════════════════════════════════╗${NC}"
echo -e "${GREEN}║        ✅ SISTEMA INICIADO EXITOSAMENTE          ║${NC}"
echo -e "${GREEN}╚══════════════════════════════════════════════════╝${NC}"
echo -e "\n${BLUE}🌐 Aplicación:${NC} http://localhost:3000"
echo -e "${BLUE}💾 SurrealDB:${NC}  http://localhost:8000"
echo -e "\n${YELLOW}📝 Comandos útiles:${NC}"
echo -e "   - Ver logs: ${CYAN}docker-compose logs -f${NC}"
echo -e "   - Ver salud: ${CYAN}./healthcheck.sh${NC}"
echo -e "   - Detener: ${CYAN}docker-compose down${NC}"
echo -e "${BLUE}====================================================${NC}"
