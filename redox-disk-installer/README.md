# 🦀 Redox OS - Instalador en Disco

Sistema de instalación completo para Redox OS que permite instalar el sistema operativo en un disco duro.

## 📋 Características

- **Instalación Completa en Disco** - Instala Redox OS en disco duro/SSD
- **Interfaz Interactiva** - Menú de opciones intuitivo
- **Particionado Automático** - Crea particiones GPT (EFI + Root)
- **Bootloader UEFI** - Instala bootloader compatible con UEFI  
- **Sistema de Archivos Flexible** - Soporta RedoxFS y ext4
- **Validación Completa** - Verifica requisitos del sistema
- **Instalación Segura** - Confirmaciones antes de modificar disco

## 🚀 Instalación Rápida

### 1. Compilar Redox OS

Primero, asegúrate de que Redox OS esté compilado:

```bash
cd /home/moebius/redox
make all
```

### 2. Compilar el Instalador

```bash
cd redox-disk-installer
cargo build --release
```

### 3. Ejecutar el Instalador

```bash
sudo ./target/release/redox-disk-installer
```

## 📦 Requisitos del Sistema

### Mínimos
- Disco duro con al menos 2GB de espacio libre
- Sistema UEFI compatible
- Privilegios de root/sudo
- Redox OS compilado

### Dependencias del Sistema
- `parted` - Particionado de discos
- `mkfs.vfat` - Formateo FAT32
- `lsblk` - Listado de discos
- `mount/umount` - Montaje de particiones

## 🔧 Uso

### Instalación Interactiva

```bash
sudo ./target/release/redox-disk-installer
```

El instalador te guiará a través de:

1. **Selección de disco** - Lista todos los discos disponibles
2. **Configuración** - Tamaño de partición EFI, sistema de archivos
3. **Confirmación** - Verifica los cambios antes de aplicarlos
4. **Instalación** - Proceso automatizado de 8 pasos
5. **Resumen** - Información de la instalación completada

### Opciones de Configuración

Durante la instalación, puedes configurar:

- **Tamaño de Partición EFI**: Entre 100 MB y 2 GB (recomendado: 512 MB)
- **Sistema de Archivos Root**:
  - `redoxfs` - Sistema de archivos nativo de Redox (recomendado)
  - `ext4` - Sistema de archivos Linux estándar

## 📊 Proceso de Instalación

El instalador realiza los siguientes pasos:

1. ✅ Verificación del disco y desmontaje de particiones
2. 📦 Creación de particiones GPT (EFI + Root)
3. 💾 Formateo de particiones (FAT32 + RedoxFS/ext4)
4. 📁 Montaje de particiones temporales
5. ⚙️  Instalación del bootloader UEFI
6. 🔧 Copia del kernel de Redox
7. 📂 Instalación del sistema de archivos
8. ⚙️  Creación de configuración de arranque
9. 🔓 Desmontaje de particiones

## ⚠️  Advertencias Importantes

### ANTES DE INSTALAR:
- **Haz una copia de seguridad** de todos tus datos importantes
- **Verifica el disco correcto** - la instalación borrará TODOS los datos
- **Asegúrate de que UEFI esté habilitado** en tu BIOS/UEFI
- **Desmonta todas las particiones** del disco de destino

### DURANTE LA INSTALACIÓN:
- **No interrumpas el proceso** una vez iniciado
- **No uses el disco** mientras se está instalando
- **Mantén la alimentación** del sistema

## 📁 Estructura de Particiones

Después de la instalación:

```
/dev/sdX
├── /dev/sdX1    # Partición EFI (FAT32, 512 MB por defecto)
│   ├── /EFI/BOOT/BOOTX64.EFI
│   ├── /EFI/redox/redox-bootloader.efi
│   ├── /redox_kernel
│   ├── /initfs.img (si existe)
│   ├── /redox.conf
│   └── /startup.nsh
└── /dev/sdX2    # Partición root (RedoxFS/ext4, resto del disco)
    └── (sistema de archivos de Redox OS)
```

## 🛠️ Resolución de Problemas

### Redox OS no arranca

1. Verifica que UEFI esté habilitado en el BIOS
2. Asegúrate de que el disco esté en la lista de arranque UEFI
3. Verifica que Secure Boot esté deshabilitado
4. Comprueba las particiones con `lsblk`

### Error "Kernel no encontrado"

```bash
# Compila Redox OS primero
cd /home/moebius/redox
make all
```

### Error "Comando no encontrado"

```bash
# Instala dependencias en Ubuntu/Debian
sudo apt install parted dosfstools

# Instala dependencias en Fedora
sudo dnf install parted dosfstools
```

### Particiones no se crean correctamente

1. Verifica que el disco no esté montado
2. Intenta limpiar el disco manualmente:
   ```bash
   sudo wipefs -a /dev/sdX
   ```

## 💡 Consejos

- **Sistema de Archivos**: Usa RedoxFS para mejor compatibilidad con Redox OS
- **Tamaño EFI**: 512 MB es suficiente para la mayoría de casos
- **Discos Virtuales**: Funciona perfectamente con QEMU, VirtualBox, etc.
- **Hardware Real**: Probado en hardware x86_64 con UEFI

## 🔍 Verificación Post-Instalación

Después de la instalación, verifica:

```bash
# Ver particiones creadas
lsblk /dev/sdX

# Montar y verificar contenido EFI
sudo mount /dev/sdX1 /mnt
ls -la /mnt/EFI/
sudo umount /mnt
```

## 🎯 Características Futuras

- [ ] Soporte para BIOS legacy (además de UEFI)
- [ ] Instalación dual-boot con otros sistemas operativos
- [ ] Configuración de red durante la instalación
- [ ] Creación de usuarios durante la instalación
- [ ] Soporte para RAID
- [ ] Encriptación de disco
- [ ] ISO de instalación booteable

## 📝 Licencia

Este proyecto está licenciado bajo la Licencia MIT.

## 🤝 Contribuciones

Las contribuciones son bienvenidas. Por favor:

1. Fork el proyecto
2. Crea una rama para tu característica
3. Commit tus cambios
4. Push a la rama
5. Abre un Pull Request

## 📞 Soporte

Si tienes problemas con la instalación:

1. Revisa la sección de resolución de problemas
2. Verifica que cumples todos los requisitos
3. Asegúrate de que Redox OS esté compilado
4. Abre un issue en el repositorio de Redox OS

---

**¡Disfruta usando Redox OS!** 🦀✨

