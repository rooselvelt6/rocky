# 🩺 UCI - ICU Medical Scales System
### Infraestructura Crítica de Automatización Clínica para Unidades de Cuidados Intensivos

![Rust](https://img.shields.io/badge/Rust-2021-orange?logo=rust)
![Axum](https://img.shields.io/badge/Axum-0.8-blue)
![Leptos](https://img.shields.io/badge/Leptos-WASM-purple)
![SurrealDB](https://img.shields.io/badge/SurrealDB-v2.5-cc00ff)
![Portability](https://img.shields.io/badge/Portability-Universal-green?logo=docker)
![Security](https://img.shields.io/badge/HADES-Military--Grade-red)

---

## 🚀 "Born for Performance, Built for Portability"
**UCI System** es una solución de ingeniería de software de grado industrial diseñada para automatizar el cálculo e interpretación de escalas médicas críticas (Glasgow, APACHE II, SOFA, SAPS II, NEWS2). 

Tras la actualización **"Tríada Suprema"**, el sistema ha alcanzado un nivel de robustez y seguridad sin precedentes en el software médico de código abierto.

---

## 🏛️ La Tríada Suprema: El Corazón del Sistema

### ⚡ ZEUS: Omnipresencia y Resiliencia
ZEUS es el orquestador que garantiza que la aplicación corra en cualquier lugar:
- **Arranque Inteligente**: El sistema detecta su entorno y autogestiona sus dependencias.
- **Motor Dual**: Soporte nativo para **Docker** (producción) y **Modo Embebido con RocksDB** (estaciones de trabajo aisladas).
- **Auto-Instalación**: Scripts proactivos que preparan el entorno (Winget en Windows, APT/PACMAN en Linux).

### 💀 HADES: El Búnker Criptográfico
La seguridad de los datos del paciente es nuestra prioridad absoluta:
- **Cifrado ChaCha20-Poly1305**: Información sensible (identidad, diagnósticos) cifrada en reposo.
- **Protocolo Leteo (Zeroize)**: Borrado físico proactivo de la memoria RAM para evitar fugas de datos.
- **El Hilo Rojo (Integridad)**: Cada registro está protegido por un hash **BLAKE3**. Cualquier alteración no autorizada en la base de datos es detectada inmediatamente.

### 🔱 POSEIDON: Sincronización en Tiempo Real
Fluidez total entre el personal médico:
- **Wave-Sync (WebSockets)**: Sincronización instantánea de eventos (<10ms).
- **Offline-First**: La aplicación sigue funcionando sin internet y sincroniza cambios automáticamente al recuperar la conexión.
- **Arquitectura de Eventos**: Un Hub central redistribuye cada acción médica a todos los terminales conectados.

---

## 🛠️ Stack Tecnológico

| Capa | Tecnologías | Ventajas Clínicas |
| :--- | :--- | :--- |
| **Lenguaje** | Rust (Edition 2021) | Cero fallos de segmentación y máxima velocidad. |
| **Backend** | Axum + WebSockets | Manejo de cientos de peticiones distribuidas. |
| **Frontend** | Leptos (WASM) + IndexedDB | Interfaz instantánea con capacidad offline total. |
| **Base de Datos** | SurrealDB (RocksDB) | Relaciones de grafo y persistencia K/V de alta velocidad. |
| **Seguridad** | ChaCha20 + Zeroize | Privacidad total y cumplimiento de estándares médicos. |

---

## 🌀 Instalación y Operación "Zero Friction"

### 🚀 El Inicio Universal (Recomendado)
Para arrancar el sistema "Nivel Dios", simplemente ejecuta el binario de ZEUS según tu plataforma:

**Unix / Linux / macOS:**
```bash
./bin/zeus-start.sh
```

**Windows (PowerShell):**
```powershell
.\bin\zeus-start.ps1
```
*Este comando detectará Docker, lo instalará si es necesario, o en su defecto, compilará la versión nativa con base de datos embebida.*

---

## 📊 Manual de Operación (Walkthrough)

### 1. Seguridad Transparente
Como médico, no verás nada diferente, pero bajo el capó, **HADES** está denegando cualquier acceso no autorizado. Si intentas leer la base de datos directamente sin las llaves del sistema, verás datos cifrados ilegibles.

### 2. Sincronización en Directo
Si abres la aplicación en dos tablets diferentes dentro de la misma UCI, verás cómo los datos de un paciente se actualizan instantáneamente en ambas pantallas gracias a **POSEIDON**.

### 3. Modo Avión / Sin Conexión
Puedes bajar al sótano del hospital sin WiFi. Realiza tus escalas, guarda los datos. Al volver a planta, POSEIDON enviará automáticamente todos los cambios al servidor central.

---

## 👨‍💻 Autor y Visión
Desarrollado por **rooselvelt6** para democratizar la tecnología de alta precisión en entornos de cuidados críticos, manteniendo la soberanía de los datos médicos y la máxima eficiencia.

---
> [!IMPORTANT]  
> **Aviso de Seguridad:** El sistema utiliza la variable `HADES_SECRET` para el cifrado. Asegúrese de respaldar esta clave; sin ella, los datos en el disco duro serán permanentemente ilegibles.
