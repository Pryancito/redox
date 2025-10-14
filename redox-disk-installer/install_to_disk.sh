#!/bin/bash
# Script rápido para instalar Redox OS en disco
# Este script compila el instalador si es necesario y lo ejecuta

set -e

echo "╔═══════════════════════════════════════════════════╗"
echo "║  🦀 Instalador Rápido de Redox OS 🦀             ║"
echo "╚═══════════════════════════════════════════════════╝"
echo

# Verificar permisos de root
if [[ $EUID -ne 0 ]]; then
   echo "❌ Error: Este script debe ejecutarse como root"
   echo "   Usa: sudo $0"
   exit 1
fi

# Cambiar al directorio del instalador
cd "$(dirname "$0")"

# Compilar si es necesario
if [[ ! -f "target/release/redox-disk-installer" ]]; then
    echo "🔨 Compilando instalador por primera vez..."
    echo
    cargo build --release
    echo
fi

# Ejecutar instalador
echo "🚀 Iniciando instalador de Redox OS..."
echo
./target/release/redox-disk-installer

