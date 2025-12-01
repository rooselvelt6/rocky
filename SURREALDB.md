# 🎯 SurrealDB - Guía de Inicio Rápido

## ✅ Instalación Completada

SurrealDB ya está instalado en tu proyecto como `surreal.exe`.

---

## 🚀 Cómo Iniciar la Base de Datos

### Opción 1: Usando el script (Recomendado)
```powershell
.\start-db.ps1
```

### Opción 2: Comando manual
```powershell
.\surreal.exe start --log info --user root --pass root file:uci.db
```

---

## 🌐 Interfaz Web

Una vez iniciado, abre tu navegador en:
- **URL**: http://localhost:8000
- **Usuario**: `root`
- **Contraseña**: `root`
- **Namespace**: `hospital`
- **Database**: `uci`

---

## 📊 Importar el Esquema Inicial

Con la base de datos corriendo, en otra terminal ejecuta:

```powershell
.\surreal.exe import --conn http://localhost:8000 --user root --pass root --ns hospital --db uci db\schema.surql
```

---

## 🔍 Comandos Útiles

### Ver versión
```powershell
.\surreal.exe version
```

### Hacer backup
```powershell
.\surreal.exe export --conn http://localhost:8000 --user root --pass root --ns hospital --db uci backup.surql
```

### Consultar datos (SQL en la terminal)
```powershell
.\surreal.exe sql --conn http://localhost:8000 --user root --pass root --ns hospital --db uci
```

Luego puedes ejecutar queries como:
```sql
SELECT * FROM patients;
SELECT * FROM glasgow_assessments;
```

---

## 📁 Estructura de Archivos

```
uci/
├── surreal.exe          ← Binario de SurrealDB
├── start-db.ps1         ← Script para iniciar DB
├── uci.db/              ← Datos persistentes (se crea automáticamente)
└── db/
    └── schema.surql     ← Esquema de la base de datos
```

---

## 🔧 Próximos Pasos

1. **Iniciar la DB**: `.\start-db.ps1`
2. **Importar esquema**: Usar el comando de importación arriba
3. **Explorar en el navegador**: http://localhost:8000
4. **Integrar con Rust**: Seguir la guía de integración

---

## 💡 Tips

- La base de datos se guarda en `uci.db/` (persistente)
- Puedes tener múltiples databases en el mismo namespace
- La interfaz web es muy útil para explorar datos y probar queries
- Presiona `Ctrl+C` en la terminal para detener el servidor

---

## 📚 Recursos

- [Documentación oficial](https://surrealdb.com/docs)
- [SurrealQL (lenguaje de queries)](https://surrealdb.com/docs/surrealql)
- [Rust SDK](https://github.com/surrealdb/surrealdb.rs)
