#!/bin/bash
# Script de diagnóstico completo para UCI System

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}=== UCI System Diagnostic Tool ===${NC}"
date
echo ""

# 1. Contenedores
echo -e "${YELLOW}📦 Estado de Contenedores:${NC}"
docker-compose ps
echo ""

# 2. SurrealDB
echo -n -e "${YELLOW}💾 Salud de SurrealDB (Port 8000): ${NC}"
if curl -sf http://localhost:8000/health &>/dev/null; then
    echo -e "${GREEN}✅ OPERATIVO${NC}"
else
    echo -e "${RED}❌ FALLANDO${NC}"
fi

# 3. Aplicación API
echo -n -e "${YELLOW}🌐 Salud de App API (Port 3000):   ${NC}"
HEALTH_DATA=$(curl -s http://localhost:3000/api/health || echo "error")
if [[ $HEALTH_DATA == *"up"* ]]; then
    echo -e "${GREEN}✅ OPERATIVO${NC} ($HEALTH_DATA)"
else
    echo -e "${RED}❌ FALLANDO${NC} (Check logs: docker-compose logs uci-app)"
fi

# 4. Recursos
echo ""
echo -e "${YELLOW}📊 Uso de Recursos:${NC}"
docker stats --no-stream rocky-surrealdb rocky-app 2>/dev/null || echo "No se pueden obtener stats - contenedores quizás detenidos."

echo ""
echo -e "${BLUE}==============================${NC}"
