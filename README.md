# 🏛️ OLYMPUS UCI - Sistema de Gestión de Unidad de Cuidados Intensivos

![Rust](https://img.shields.io/badge/Rust-2021-orange?style=for-the-badge&logo=rust)
![Version](https://img.shields.io/badge/Version-16.0.0-gold?style=for-the-badge)
![Actors](https://img.shields.io/badge/Actors-21%20Gods%20Mesh-green?style=for-the-badge)
![Status](https://img.shields.io/badge/Status-Production%20Ready-brightgreen?style=for-the-badge)

> **Sistema UCI industrial basado en la malla de actores Ractor (OTP) con persistencia en múltiples niveles y seguridad de memoria.**

---

## 📋 Descripción

**OLYMPUS UCI** es un sistema de gestión de Unidad de Cuidados Intensivos desarrollado en Rust, utilizando una arquitectura de actores basada en **Ractor** para alta disponibilidad y concurrencia segura.

### Características Principales

- 🏛️ **21 Actores (Dioses)** - Cada dios representa un dominio funcional del sistema
- ⚡ **Ractor Framework** - Modelo de actores nativo de Rust para concurrencia
- 🔒 **Zeroize & Secrecy** - Protección de memoria con borrado seguro de datos sensibles
- 💾 **Persistencia Triple** - Valkey (cache) + SurrealDB (transaccional)
- 🧠 **Cálculos Clínicos** - SAPS, SOFA, NEWS2, Glasgow automatizados
- 🌐 **API REST** - Backend Axum con servidor frontend estático

---

## 🏛️ El Panteón: 21 Dioses Activos

### Trinidad Suprema

| Dios | Dominio | Descripción |
|------|---------|-------------|
| **👑 Zeus** | Gobernanza | Supervisor raíz del sistema |
| **🔒 Hades** | Seguridad | Protección y cifrado de datos |
| **🌊 Poseidón** | Datos | Gestión de flujo de datos |

### Dioses de Análisis Clínico

| Dios | Dominio |
|------|---------|
| **🧠 Athena** | Escalas/ML |
| **📊 Demeter** | Estadísticas |
| **🔬 Apollo** | Laboratorios |
| **📋 Artemis** | Registros |
| **⚕️ Moirai** | Protocolos |

### Dioses de Infraestructura

| Dios | Dominio |
|------|---------|
| **📨 Hermes** | Mensajería |
| **🏛️ Hestia** | Persistencia |
| **👁️ Erinyes** | Monitoreo |
| **⏱️ Chronos** | Scheduling |
| **🔮 Iris** | Logging |

### Dioses de UI y Negocio

| Dios | Dominio |
|------|---------|
| **🎨 Aphrodite** | UI/Temas |
| **⚔️ Ares** | Validaciones |
| **👸 Hera** | Usuarios |
| **🔥 Hefesto** | Template engine |
| **🎭 Dionysus** | Reportes |

### Dioses de Sistema

| Dios | Dominio |
|------|---------|
| **🌅 Aurora** | Notificaciones |
| **🌌 Chaos** | Fallbacks |
| **⚖️ Nemesis** | Auditoría |

---

## 📁 Estructura del Proyecto

```
rocky/
├── olympus-core/           # Tipos compartidos (Patient, User, escalas clínicas)
│   └── src/
│       ├── patient.rs       # Modelo de paciente UCI
│       ├── saps.rs          # SAPS II score
│       ├── sofa.rs          # SOFA score
│       ├── glasgow.rs       # Escala Glasgow
│       └── news2.rs         # NEWS2 score
│
├── olympus-server/         # Servidor principal (21 actores)
│   └── src/
│       ├── main.rs         # Axum + Tokio runtime
│       ├── system/
│       │   └── genesis.rs   # Bootloader de actores
│       ├── actors/         # Los 21 dioses
│       │   ├── zeus.rs     # Supervisor
│       │   ├── hades.rs    # Seguridad
│       │   ├── poseidon.rs # Datos
│       │   └── ...
│       ├── traits/         # Traits de actor
│       ├── infrastructure/ # Valkey, SurrealDB
│       └── uci/            # Lógica UCI
│
├── olympus-client/         # Frontend estático
│   └── dist/
│
├── server/                  # Módulo legacy (compatibilidad)
│
├── client/                  # Cliente CLI legacy
│
├── tests/                   # Tests de integración
│
├── Cargo.toml              # Workspace configuration
├── docker-compose.yml      # Valkey + SurrealDB
└── justfile                # Tareas de desarrollo
```

---

## 🚀 Guía de Inicio Rápido

### 1. Requisitos

- Rust 1.75+
- Docker y Docker Compose
- Valkey
- SurrealDB (opcional)

### 2. Iniciar Infraestructura

```bash
docker-compose up -d valkey surrealdb
```

### 3. Ejecutar el Servidor

```bash
cargo run -p olympus-server
```

**Salida esperada:**
```
🏔️  OLYMPUS SYSTEM v16 - STARTING UP  🏔️
⚡  Server Mode with 21 Gods (Actors)
✨ GENESIS: Iniciando secuencia de ignición Ractor v16...
⚡ Zeus igniting as Root Supervisor...
🌌 GENESIS: All 21 Gods have been successfully spawned in Ractor.
🌍 API Gateway escuchando en http://127.0.0.1:3000
🌐 Frontend disponible en http://127.0.0.1:3000/
```

### 4. Endpoints API

| Endpoint | Descripción |
|----------|-------------|
| `GET /` | Redirección al frontend |
| `GET /health` | Health check |
| `GET /api/status` | Estado del sistema |
| `GET /api/login` | Endpoint de login |
| `GET /api/patients` | Listar pacientes |
| `GET /api/patients/:id` | Obtener paciente |

---

## 📊 Escalas Clínicas Implementadas

El sistema calcula automáticamente:

- **SAPS II** - Simplified Acute Physiology Score
- **SOFA** - Sequential Organ Failure Assessment
- **NEWS2** - National Early Warning Score
- **Glasgow** - Escala de coma de Glasgow
- **Apache II** - Acute Physiology and Chronic Health Evaluation

---

## 🔧 Desarrollo

### Comandos Disponibles

```bash
just build      # Compilar proyecto
just test       # Ejecutar tests
just lint       # Verificar código
just format     # Formatear código
just clean      # Limpiar build
```

---

## 📄 Licencia

MIT License - Ver [LICENSE](LICENSE) para detalles.

---

> **🏛️ OLYMPUS UCI: La fuerza del acero (Rust), la resiliencia del cristal (Ractor) y la precisión de la medicina.**
