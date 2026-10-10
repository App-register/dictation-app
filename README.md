# Dicto — Push-to-Talk Speech-to-Text

**Dicto** est une application de dictée vocale *push-to-talk* légère, écrite en **Rust**, compatible **Windows 11**.

## Caractéristiques

- **🏎️ Ultra-rapide** : modèle Whisper tiny ou base, transcription en < 0.5s
- **🧠 Local & Privé** : tout sur CPU, rien ne quitte votre machine
- **🔤 Bilingue EN/FR** : détection automatique ou sélection manuelle
- **📖 Dictionnaire personnalisable** : mots avec chiffres acceptés (ex: `c4` → `Competition 4 ever`)
- **⚙️ Règles de formatage** : comme Murmure — capitalisation, nombres, ponctuation française, regex
- **🎤 Sélection du micro** : choisissez votre périphérique d'entrée
- **⌨️ Raccourci personnalisable** : configurez la touche PTT
- **🪟 Overlay temps réel** : fenêtre transparente affichant la transcription
- **📋 Collage automatique** : le texte transcrit est collé directement à la position curseur

## Architecture

```
dicto/
├── crates/
│   ├── dicto-core/     # Moteur: audio, STT (whisper.cpp), post-processing, output
│   ├── dicto-gui/      # Interface native Windows (NWG) + overlay + hotkeys
│   └── dicto-app/      # Binaire final
├── models/             # Modèles Whisper (téléchargés au premier lancement)
└── README.md
```

## Prérequis

- Windows 11 (ou Windows 10 x64)
- [Rust](https://rustup.rs/) (toolchain `stable-msvc`)
- ~1 Go d'espace disque (modèle tiny: 75 Mo, base: 142 Mo)

## Compilation

```bash
cargo build --release
```

Le binaire se trouve dans `target/release/dicto.exe`.

## Configuration

Les paramètres sont stockés dans `%APPDATA%/Dicto/settings.json` :
- Modèle : `tiny` (recommandé) ou `base`
- Langue : `auto`, `en`, `fr`
- Touche PTT : par défaut `Ctrl+Shift+F4`
- Sélection micro : par nom partiel

## Licence

MIT