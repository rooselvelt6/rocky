# UCI - ICU Medical Scales Automation & Optimization

A Rust-based web application for automating and optimizing medical scale calculations in Intensive Care Units (ICUs).

## 🏥 Overview

UCI is a medical software system designed to streamline the calculation and interpretation of critical medical scales used in ICU settings. The system provides accurate, fast, and reliable assessments to support healthcare professionals in making informed clinical decisions.

## ✨ Features

### Currently Implemented
- **Glasgow Coma Scale (GCS)** - Neurological assessment tool
  - Eye opening response evaluation
  - Verbal response evaluation
  - Motor response evaluation
  - Automatic severity classification (Mild, Moderate, Severe TBI)
  - Clinical recommendations based on score

### Planned Features
- APACHE II Score
- SOFA Score (Sequential Organ Failure Assessment)
- SAPS II (Simplified Acute Physiology Score)
- Patient registration system
- Results history and tracking
- Multi-language support

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

### Running the Development Server

```bash
cargo run
```

The server will start on `http://localhost:3000`

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

- [ ] RESTful API endpoints for scale calculations
- [ ] Interactive web UI with form inputs
- [ ] Patient registration system
- [ ] Results history and persistence
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

[Choose your license - MIT, Apache-2.0, GPL-3.0, etc.]

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
