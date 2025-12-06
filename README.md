# UCI - ICU Medical Scales Automation & Optimization

A Rust-based web application for automating and optimizing medical scale calculations in Intensive Care Units (ICUs).

## 🏥 Overview

UCI is a medical software system designed to streamline the calculation and interpretation of critical medical scales used in ICU settings. The system provides accurate, fast, and reliable assessments to support healthcare professionals in making informed clinical decisions.

## ✨ Features

### Currently Implemented
- **Glasgow Coma Scale (GCS)** - Neurological assessment tool ✅
  - Eye opening response evaluation
  - Verbal response evaluation
  - Motor response evaluation
  - Automatic severity classification (Mild, Moderate, Severe TBI)
  - Clinical recommendations based on score
  - Full frontend form with real-time calculation

- **APACHE II Score** - Acute Physiology and Chronic Health Evaluation ✅
  - 12 physiological parameters evaluation
  - Age and chronic health assessment
  - Predicted mortality calculation
  - Severity classification with recommendations
  - Backend API endpoint functional

- **SOFA Score** - Sequential Organ Failure Assessment ✅
  - 6 organ systems evaluation
  - Respiratory, coagulation, liver, cardiovascular, CNS, renal scoring
  - Calculation logic complete
  - Ready for frontend integration

- **SAPS II Score** - Simplified Acute Physiology Score ✅
  - 15 parameters evaluation
  - Advanced mortality prediction using logistic regression
  - Severity classification
  - Calculation logic complete

- **Patient Registration System** ✅
  - Complete patient data entry form
  - Database storage with SurrealDB
  - Patient listing API

### In Progress / Planned Features
- Frontend forms for APACHE II, SOFA, SAPS II
- Patient list with search functionality
- Dashboard with statistics
- Assessment history per patient
- Multi-language support (ES/EN)
- PDF export functionality
- User authentication
- Data visualization

## 🚀 Prerequisites

- **Rust** 1.70+ ([Install Rust](https://rustup.rs/))
- **Cargo** (comes with Rust)

## 📦 Installation

1. Clone the repository:
```bash
git clone <your-repo-url>
cd uci
```

2. Build the project:
```bash
cargo build --release
```

## 🎯 Usage

### Building the Frontend

First, install Trunk if you haven't already:
```bash
cargo install trunk
```

Build the Leptos frontend:
```bash
trunk build
```

For development with hot reload:
```bash
trunk serve
# Frontend will be available at http://localhost:8080
```

### Running the Backend Server

Make sure SurrealDB is running first:
```bash
.\start-db.ps1
# Or manually: .\surreal.exe start --user root --pass root file:uci.db
```

Then start the Axum backend:
```bash
cargo run
# Server will start on http://localhost:3000
```

### Accessing the Application

- **Web Interface**: `http://localhost:3000`
- **Static CSS**: `http://localhost:3000/style.css`

## 📁 Project Structure

```
uci/
├── src/
│   ├── main.rs           # Web server configuration
│   ├── uci.rs            # Main module
│   └── uci/
│       ├── scale.rs      # Scales module entry point
│       └── scale/
│           └── glasgow.rs # Glasgow Coma Scale implementation
├── assets/
│   ├── index.html        # Web UI
│   └── style.css         # Styling
├── Cargo.toml            # Project dependencies
└── README.md             # This file
```

## 🛠️ Technologies Used

### Backend
- **[Rust](https://www.rust-lang.org/)** - Systems programming language
- **[Axum](https://github.com/tokio-rs/axum)** - Web framework
- **[Tokio](https://tokio.rs/)** - Async runtime
- **[Tower-HTTP](https://github.com/tower-rs/tower-http)** - HTTP middleware

### Frontend
- HTML5
- CSS3
- JavaScript (planned)

## 📊 Glasgow Coma Scale

The Glasgow Coma Scale (GCS) is a neurological scale that aims to give a reliable and objective way of recording the conscious state of a person.

### Components

1. **Eye Opening Response (1-4 points)**
   - 4: Spontaneous
   - 3: To verbal command
   - 2: To pain
   - 1: No response

2. **Verbal Response (1-5 points)**
   - 5: Oriented and conversing
   - 4: Disoriented and conversing
   - 3: Inappropriate words
   - 2: Incomprehensible sounds
   - 1: No response

3. **Motor Response (1-6 points)**
   - 6: Obeys commands
   - 5: Localizes pain
   - 4: Withdrawal from pain
   - 3: Flexion to pain
   - 2: Extension to pain
   - 1: No response

### Interpretation

- **15**: Normal - Alert and Oriented
- **13-14**: Mild TBI - Clinical observation or discharge with clear instructions
- **9-12**: Moderate TBI - Requires CT scan and/or hospitalization
- **3-8**: Severe TBI - Requires immediate resuscitation, airway management (intubation), and ICU admission

## 🗺️ Roadmap

### Completed ✅
- [x] Glasgow Coma Scale with full frontend
- [x] Patient registration system
- [x] APACHE II calculation logic & backend API
- [x] SOFA calculation logic
- [x] SAPS II calculation logic
- [x] SurrealDB integration
- [x] Database schema (patients, glasgow_assessments, apache_assessments)
- [x] Trunk build configuration

### In Progress 🚧
- [ ] APACHE II frontend form
- [ ] SOFA frontend form & backend API
- [ ] SAPS II frontend form & backend API
- [ ] Patient list/search interface
- [ ] Dashboard with statistics

### Planned 📋
- [ ] Assessment history view per patient
- [ ] Link assessments to specific patients
- [ ] Data visualization (charts/graphs)
- [ ] User authentication
- [ ] Multiple language support (ES, EN)
- [ ] Export results to PDF
- [ ] Integration with hospital information systems

## 🧪 Development

### Running Tests
```bash
cargo test
```

### Building for Production
```bash
cargo build --release
```

The optimized binary will be in `target/release/uci`

## 📝 License

This project is licensed under the [GNU General Public License v3.0](LICENSE).

Permissions of this strong copyleft license are conditioned on making available complete source code of licensed works and modifications, which include larger works using a licensed work, under the same license. Copyright and license notices must be preserved. Contributors provide an express grant of patent rights.

## 👨‍💻 Author

**Your Name**
- GitHub: [@rooselvelt6]
- Email: rooselvelt6@gmail.com

## 🤝 Contributing

Contributions, issues, and feature requests are welcome!

## ⚠️ Disclaimer

This software is intended for educational and research purposes. It should not replace professional medical judgment. Always consult with qualified healthcare professionals for clinical decisions.

---

**Made with ❤️ for improving ICU care**
