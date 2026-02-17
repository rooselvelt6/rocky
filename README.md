# 🏛️ OLYMPUS UCI v15 - Sistema de Actores OTP

![Rust](https://img.shields.io/badge/Rust-2021-orange?style=for-the-badge&logo=rust)
![Version](https://img.shields.io/badge/Version-15.0.0-gold?style=for-the-badge)
![Actors](https://img.shields.io/badge/Actors-20%20Active-green?style=for-the-badge)
![Status](https://img.shields.io/badge/Status-Operational-brightgreen?style=for-the-badge)

> **Sistema UCI completo basado en actores OTP (Erlang-style) con 20 dioses griegos orquestados por Zeus**

---

## 🎯 ¿Qué es OLYMPUS UCI?

**OLYMPUS UCI** es un sistema de gestión de pacientes para Unidades de Cuidados Intensivos (UCI) construido con **Rust** y arquitectura de **actores OTP** (como Erlang/Elixir). El sistema utiliza 20 "dioses" (actores) que se comunican entre sí mediante mensajes asíncronos, supervisados por **Zeus** en una jerarquía de supervisión OTP.

### ⚡ Características Principales

- ✅ **20 Dioses (Actores) funcionando** - Cada uno con responsabilidades específicas
- ✅ **Autenticación OTP de 2 pasos** - Hades gestiona seguridad
- ✅ **5 Escalas Médicas** - Athena calcula Glasgow, SOFA, APACHE, SAPS, NEWS2
- ✅ **Gestión de Pacientes** - Poseidon + Hestia persisten datos
- ✅ **Temas UI Dinámicos** - Aphrodite controla 4 temas visuales
- ✅ **Monitoreo en Tiempo Real** - Erinyes monitorea health de todos los dioses
- ✅ **Trinidad Supervisada** - Zeus supervisa Zeus-Hades-Poseidon (críticos)

---

## 🏛️ El Panteón: 20 Dioses Activos

### ⚡ Trinidad Suprema (Críticos)

| Dios | Dominio | Función | Estado |
|------|---------|---------|--------|
| **👑 Zeus** | Gobernanza | Supervisor OTP de 19 actores, reinicio automático | ✅ Supervising |
| **🔒 Hades** | Seguridad | OTP auth, JWT tokens, validación credenciales | ✅ Protecting |
| **🌊 Poseidón** | Datos | Conexión SurrealDB, queries pacientes | ✅ Connected |

### 🎨 Dioses de UI/UX y Visualización

| Dios | Dominio | Función | Estado |
|------|---------|---------|--------|
| **🎨 Aphrodite** | UI/Belleza | **4 temas dinámicos**: Olympus Dark/Light, Golden, Cosmic | ✅ Designing |

### 🧠 Dioses de Análisis Clínico

| Dios | Dominio | Función | Estado |
|------|---------|---------|--------|
| **🧠 Athena** | Escalas/ML | Calcula Glasgow, SOFA, APACHE II, SAPS II, NEWS2 | ✅ Analyzing |

### 💾 Dioses de Infraestructura

| Dios | Dominio | Función | Estado |
|------|---------|---------|--------|
| **📨 Hermes** | Mensajería | Routing de mensajes entre actores | ✅ Routing |
| **🏛️ Hestia** | Persistencia | Cache Valkey, buffer de escritura | ✅ Caching |
| **👁️ Erinyes** | Monitoreo | Heartbeats cada 10s, health checks 20 dioses | ✅ Monitoring |

### 🌟 Dioses Menores (13)

Apollo, Artemis, Hera, Ares, Hefesto, Chronos, Moirai, Chaos, Aurora, Iris, Demeter, Dionysus, Nemesis

**Total: 20/20 Dioses Activos** ✅

---

## 🚀 Guía de Inicio Rápido

### **1. Clonar e Iniciar Infraestructura**

```bash
# Clonar repositorio
git clone https://github.com/rooselvelt6/rocky.git
cd rocky

# Iniciar Valkey + SurrealDB
docker-compose up -d valkey surrealdb

# Verificar servicios
docker ps
```

### **2. Compilar y Ejecutar**

```bash
# Compilar servidor
cargo build -p olympus-server --release

# Ejecutar (20 dioses se iniciarán automáticamente)
cargo run -p olympus-server
```

**Verás en consola:**
```
🏔️  OLYMPUS SYSTEM v15 - ACTOR SYSTEM  🏔️
⚡  20 Divine Gods - OTP Architecture
🚀  Integrando sistema de actores...
✨ GENESIS: Iniciando secuencia de ignición del Olimpo v15...
⚡ Zeus desplegado
🔒 Hades desplegado
🌊 Poseidón desplegado
🧠 Athena desplegada
📨 Hermes desplegado
🏛️ Hestia desplegada
👁️ Erinyes desplegado
🎨 Aphrodite desplegada - Gestionando UI/Temas
... 12 dioses menores
✅ 20 Dioses iniciados correctamente
🚀 Servidor Axum + Actores corriendo en http://127.0.0.1:3000
```

### **3. Acceder al Sistema**

Abre tu navegador: **http://127.0.0.1:3000**

**Credenciales de prueba:**
- Usuario: `admin`
- Password: `admin123`
- OTP: `123456`

---

## 📡 APIs del Sistema

### **Autenticación (Hades)**

```bash
# Paso 1: Login inicial
curl -X POST http://127.0.0.1:3000/api/login_step1 \
  -H "Content-Type: application/json" \
  -d '{"username":"admin","password":"admin123"}'

# Respuesta: {"session_id":"session_12345","message":"Código OTP: 123456"}

# Paso 2: Verificar OTP
curl -X POST http://127.0.0.1:3000/api/login_step2 \
  -H "Content-Type: application/json" \
  -d '{"session_id":"session_12345","otp_code":"123456"}'

# Respuesta: {"token":"jwt_token_olympus_2026","message":"¡Zeus aprueba tu acceso!"}
```

### **Gestión de Pacientes (Poseidón)**

```bash
# Crear paciente
curl -X POST http://127.0.0.1:3000/api/patients \
  -H "Content-Type: application/json" \
  -d '{
    "first_name": "Juan",
    "last_name": "Perez",
    "identity_card": "12345678",
    "principal_diagnosis": "Neumonía severa"
  }'

# Listar pacientes
curl http://127.0.0.1:3000/api/patients

# Eliminar paciente
curl -X DELETE http://127.0.0.1:3000/api/patients/{id}
```

### **Escalas Médicas (Athena)**

```bash
# Glasgow Coma Scale
curl -X POST http://127.0.0.1:3000/api/scales/glasgow \
  -d '{"patient_id":"1","eye":4,"verbal":4,"motor":6}'
# Respuesta: {"total":14,"interpretation":"Coma leve/Normal"}

# SOFA Score
curl -X POST http://127.0.0.1:3000/api/scales/sofa \
  -d '{"respiratory":2,"coagulation":1,"liver":0,"cardiovascular":1,"cns":0,"renal":0}'
# Respuesta: {"total":4,"predicted_mortality":"< 10%"}

# NEWS2
curl -X POST http://127.0.0.1:3000/api/scales/news2 \
  -d '{"respiration_rate":16,"oxygen_saturation":97,"temperature":37.0,"heart_rate":80,"systolic_bp":120}'
# Respuesta: {"total":0,"risk_level":"Bajo riesgo"}
```

### **Monitoreo de Dioses**

```bash
# Estado del sistema
curl http://127.0.0.1:3000/api/status

# Todos los dioses (20)
curl http://127.0.0.1:3000/api/olympus/gods

# Trinidad (Zeus-Hades-Poseidón)
curl http://127.0.0.1:3000/api/olympus/trinity

# Estadísticas
curl http://127.0.0.1:3000/api/admin/stats
```

### **UI/Temas (Aphrodite)**

```bash
# Ver temas disponibles
curl http://127.0.0.1:3000/api/aphrodite/themes
# Respuesta: ["Olympus Dark","Olympus Light","Golden Olympus","Cosmic"]

# Ver tema actual
curl http://127.0.0.1:3000/api/aphrodite/theme

# Cambiar tema
curl -X POST http://127.0.0.1:3000/api/aphrodite/theme \
  -d '{"theme_name":"Golden Olympus"}'

# Obtener CSS variables
curl http://127.0.0.1:3000/api/aphrodite/css
```

---

## 🎨 Temas de Aphrodite

### **4 Temas Disponibles**

| Tema | Descripción | Colores Principales |
|------|-------------|---------------------|
| **Olympus Dark** | Tema oscuro por defecto | Indigo (#6366f1), Slate (#0f172a) |
| **Olympus Light** | Tema claro profesional | Indigo (#4f46e5), White (#ffffff) |
| **Golden Olympus** | Dorado divino | Gold (#fbbf24), Stone (#1c1917) |
| **Cosmic** | Cósmico futurista | Cyan (#06b6d4), Deep Blue (#020617) |

Cada tema incluye:
- Paleta completa de colores (primario, secundario, acento, éxito, warning, error)
- CSS variables dinámicas
- Componentes UI estilizados
- Bordes y sombras consistentes

---

## 🏗️ Arquitectura

### **Diagrama de Componentes**

```
┌─────────────────────────────────────────────────────────────┐
│                    FRONTEND (Leptos)                        │
│         Leptos 0.7 + WebAssembly + Tailwind CSS             │
├─────────────────────────────────────────────────────────────┤
│                   SERVIDOR AXUM (Rust)                      │
│  ┌───────────────────────────────────────────────────────┐  │
│  │              PANTEÓN DE 20 DIOSES (Actores)            │  │
│  │  ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐     │  │
│  │  │  ZEUS   │ │  HADES  │ │POSEIDÓN │ │ ATHENA  │     │  │
│  │  │Governance│ │Security │ │  Data   │ │Clinical │     │  │
│  │  └─────────┘ └─────────┘ └─────────┘ └─────────┘     │  │
│  │  ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐     │  │
│  │  │ HERMES  │ │ HESTIA  │ │ ERINYES │ │APHRODITE│     │  │
│  │  │ Messages│ │  Cache  │ │ Monitor │ │  UI/UX  │     │  │
│  │  └─────────┘ └─────────┘ └─────────┘ └─────────┘     │  │
│  │  + 12 dioses menores (Apollo, Artemis, Hera, etc.)    │  │
│  └───────────────────────────────────────────────────────┘  │
├─────────────────────────────────────────────────────────────┤
│              INFRAESTRUCTURA DE DATOS                       │
│  ┌─────────────────┐  ┌─────────────────┐                  │
│  │   SurrealDB     │  │     Valkey      │                  │
│  │   (Persistencia)│  │     (Cache)     │                  │
│  │   Puerto: 8000  │  │   Puerto: 6379  │                  │
│  └─────────────────┘  └─────────────────┘                  │
└─────────────────────────────────────────────────────────────┘
```

### **Comunicación entre Actores**

Los dioses se comunican mediante **mensajes asíncronos** (patrón OTP):

1. **Zeus** supervisa a todos los demás dioses
2. **Erinyes** recibe heartbeats de todos los dioses cada 10s
3. **Hermes** enruta mensajes entre dioses
4. Cada dios tiene su mailbox (canal mpsc) para recibir mensajes
5. Si un dios falla, Zeus puede reiniciarlo automáticamente

---

## 📁 Estructura del Proyecto

```
rocky/
├── server/                    # Servidor Axum + Actores
│   ├── src/
│   │   ├── main.rs           # Servidor HTTP + rutas API
│   │   ├── genesis.rs        # Bootloader de 20 dioses
│   │   ├── actors/           # 20 Dioses implementados
│   │   │   ├── mod.rs        # Trait OlympianActor
│   │   │   ├── zeus.rs       # Supervisor
│   │   │   ├── hades.rs      # Seguridad/Auth
│   │   │   ├── poseidon.rs   # Datos/SurrealDB
│   │   │   ├── athena.rs     # Escalas médicas
│   │   │   ├── aphrodite.rs  # UI/Temas (4 temas)
│   │   │   ├── hermes.rs     # Mensajería
│   │   │   ├── hestia.rs     # Cache/Valkey
│   │   │   ├── erinyes.rs    # Monitoreo
│   │   │   └── minor_gods.rs # 12 dioses menores
│   │   └── lib.rs
│   └── Cargo.toml
│
├── client/                    # Frontend Leptos
│   ├── src/
│   │   └── lib.rs           # UI completa + panel Aphrodite
│   ├── index.html
│   └── Cargo.toml
│
├── docker-compose.yml         # Valkey + SurrealDB
├── Cargo.toml                # Workspace
└── README.md                 # Este archivo
```

---

## 🔧 Stack Tecnológico

### **Backend**
- **Rust 2021** - Lenguaje seguro y performante
- **Axum** - Framework web HTTP
- **Tokio** - Runtime asíncrono
- **async-trait** - Traits async

### **Frontend**
- **Leptos 0.7** - Framework web reactivo en Rust
- **WebAssembly (WASM)** - Ejecución en navegador
- **Tailwind CSS** - Estilos utility-first

### **Infraestructura**
- **SurrealDB 2.4** - Base de datos multimodal
- **Valkey (Redis)** - Cache en memoria
- **Docker Compose** - Orquestación

---

## 🎮 Uso del Sistema

### **1. Login**

```
http://127.0.0.1:3000

Usuario: admin
Password: admin123
→ Recibes OTP: 123456

Ingresa OTP: 123456
→ ¡Acceso concedido por Zeus!
```

### **2. Navegación Principal**

- **Inicio** - Dashboard con estadísticas
- **Pacientes** - CRUD completo de pacientes
- **Escalas** - Glasgow, SOFA, APACHE, SAPS, NEWS2
- **Dioses** - Monitor de 20 dioses en tiempo real
- **✨ Aphrodite** - Panel de temas y UI

### **3. Cambiar Tema (Aphrodite)**

1. Haz clic en **"✨ Aphrodite"** en la barra de navegación
2. Selecciona un tema: Olympus Dark, Light, Golden, o Cosmic
3. Haz clic en **"Aplicar Tema"**
4. ¡El sistema cambia de apariencia instantáneamente!

---

## 🧪 Testing

```bash
# Verificar que los 20 dioses están activos
curl http://127.0.0.1:3000/api/olympus/gods | jq '.gods | length'
# Resultado esperado: 20

# Verificar Trinidad saludable
curl http://127.0.0.1:3000/api/olympus/trinity | jq '.all_healthy'
# Resultado esperado: true

# Verificar estado del sistema
curl http://127.0.0.1:3000/api/status | jq '.active_gods'
# Resultado esperado: 20
```

---

## 🌐 Endpoints Principales

| Endpoint | Método | Descripción | Dios |
|----------|--------|-------------|------|
| `/api/login_step1` | POST | Login inicial | Hades |
| `/api/login_step2` | POST | Verificación OTP | Hades |
| `/api/patients` | GET/POST | CRUD pacientes | Poseidón |
| `/api/scales/glasgow` | POST | Escala Glasgow | Athena |
| `/api/scales/sofa` | POST | Escala SOFA | Athena |
| `/api/scales/news2` | POST | Escala NEWS2 | Athena |
| `/api/olympus/gods` | GET | Lista 20 dioses | Zeus |
| `/api/olympus/trinity` | GET | Estado Trinidad | Zeus |
| `/api/aphrodite/themes` | GET | Temas disponibles | Aphrodite |
| `/api/aphrodite/theme` | POST | Cambiar tema | Aphrodite |

---

## 📊 Estado Actual del Sistema

```
╔══════════════════════════════════════════════════════════════╗
║              🏛️  OLYMPUS UCI v15 - ESTADO REAL              ║
╚══════════════════════════════════════════════════════════════╝

⚡ DIOSES ACTIVOS: 20/20
  ✅ Zeus (Supervising)     ✅ Hades (Protecting)   ✅ Poseidón (Connected)
  ✅ Athena (Analyzing)     ✅ Hermes (Routing)     ✅ Hestia (Caching)
  ✅ Erinyes (Monitoring)   ✅ Aphrodite (Designing) ✅ + 12 menores

🔧 FUNCIONALIDADES OPERATIVAS:
  ✅ Autenticación OTP 2 pasos
  ✅ CRUD Pacientes (SurrealDB)
  ✅ 5 Escalas médicas (Glasgow, SOFA, APACHE, SAPS, NEWS2)
  ✅ 4 Temas UI dinámicos (Aphrodite)
  ✅ Monitoreo en tiempo real (Erinyes)
  ✅ Supervisión OTP-style (Zeus)

🗄️ INFRAESTRUCTURA:
  ✅ SurrealDB (Puerto 8000) - Persistencia
  ✅ Valkey (Puerto 6379) - Cache
  ✅ Servidor (Puerto 3000) - HTTP + Actores

🎨 TEMAS DISPONIBLES:
  ✅ Olympus Dark (default)
  ✅ Olympus Light
  ✅ Golden Olympus
  ✅ Cosmic

📡 URLs:
  🌐 Frontend: http://127.0.0.1:3000
  📊 API:      http://127.0.0.1:3000/api/status

🔑 CREDENCIALES DEMO:
  Usuario:  admin
  Password: admin123
  OTP:      123456
```

---

## 🤝 Contribuir

```bash
# 1. Fork y clone
git clone https://github.com/tu-usuario/rocky.git
cd rocky

# 2. Crear rama
git checkout -b feature/nueva-funcionalidad

# 3. Implementar cambios
# 4. Commit
git commit -m "feat: agrega funcionalidad X"

# 5. Push y PR
git push origin feature/nueva-funcionalidad
```

---

## 📄 Licencia

MIT License - Ver [LICENSE](LICENSE) para detalles.

---

## 🙏 Agradecimientos

- **Rust Community** - Por el ecosistema robusto
- **Erlang/OTP** - Por la inspiración en patrones de actores
- **SurrealDB Team** - Base de datos nativa en Rust
- **Leptos Team** - Framework web reactivo

---

> **🏛️ OLYMPUS UCI v15: 20 dioses trabajando en armonía para gestionar pacientes críticos. Desde la supervisión divina de Zeus hasta la belleza radiante de Aphrodite, cada actor cumple su deber sagrado.**

<p align="center">
  <strong>⭐ Star este repo si te parece útil! ⭐</strong>
</p>
