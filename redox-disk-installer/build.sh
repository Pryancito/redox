#!/bin/bash
# Script de compilación para el instalador de Redox OS

set -e

echo "╔═══════════════════════════════════════════════════╗"
echo "║  🦀 Compilando Instalador de Redox OS 🦀         ║"
echo "╚═══════════════════════════════════════════════════╝"
echo

# Verificar que estamos en el directorio correcto
if [[ ! -f "Cargo.toml" ]]; then
    echo "❌ Error: Ejecuta este script desde el directorio redox-disk-installer"
    exit 1
fi

# Compilar en modo release
echo "🔨 Compilando instalador en modo release..."
cargo build --release

if [[ $? -eq 0 ]]; then
    echo
    echo "✅ Compilación exitosa!"
    echo
    echo "📦 Binario generado en: target/release/redox-disk-installer"
    echo
    echo "Para ejecutar el instalador:"
    echo "  sudo ./target/release/redox-disk-installer"
    echo
else
    echo
    echo "❌ Error durante la compilación"
    exit 1
fi

