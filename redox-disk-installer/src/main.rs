use std::io::{self, Write};

mod disk_manager;
mod direct_installer;
mod validation;

use disk_manager::DiskManager;
use direct_installer::DirectInstaller;
use validation::SystemValidator;

fn main() {
    println!("🦀 Redox OS - Instalador en Disco v1.0.0 🦀");
    println!("===========================================");
    println!();
    
    // Verificar permisos de root
    if !is_root() {
        eprintln!("❌ Error: Este instalador debe ejecutarse como root");
        eprintln!("   Usa: sudo ./redox-disk-installer");
        std::process::exit(1);
    }
    
    // Validar sistema
    let validator = SystemValidator::new();
    if let Err(e) = validator.validate_system() {
        eprintln!("❌ Error de validación: {}", e);
        eprintln!("   Asegúrate de que todos los comandos requeridos estén instalados");
        std::process::exit(1);
    }
    
    // Verificar que Redox OS esté compilado
    if let Err(e) = validator.validate_redox_build() {
        eprintln!("⚠️  Advertencia: {}", e);
        eprintln!("   Ejecuta 'make all' para compilar Redox OS antes de continuar");
        let proceed = read_input("¿Deseas continuar de todos modos? (s/N): ");
        if proceed.trim().to_lowercase() != "s" {
            std::process::exit(0);
        }
    }
    
    // Mostrar menú principal
    loop {
        show_main_menu();
        
        let choice = read_input("Selecciona una opción: ");
        
        match choice.trim() {
            "1" => {
                install_redox_os_direct();
            }
            "2" => {
                show_disk_info();
            }
            "3" => {
                show_help();
            }
            "4" => {
                println!("¡Hasta luego! 🦀");
                break;
            }
            _ => {
                println!("❌ Opción inválida. Intenta de nuevo.");
            }
        }
        
        println!();
    }
}

fn show_main_menu() {
    println!("╔═══════════════════════════════════════════════════╗");
    println!("║          🦀 Menú Principal - Redox OS 🦀          ║");
    println!("╠═══════════════════════════════════════════════════╣");
    println!("║  1. Instalar Redox OS en disco                    ║");
    println!("║  2. Mostrar información de discos                 ║");
    println!("║  3. Ayuda                                         ║");
    println!("║  4. Salir                                         ║");
    println!("╚═══════════════════════════════════════════════════╝");
    println!();
}

fn install_redox_os_direct() {
    println!("╔═══════════════════════════════════════════════════╗");
    println!("║        Instalación de Redox OS en Disco          ║");
    println!("╚═══════════════════════════════════════════════════╝");
    println!();
    
    // Mostrar discos disponibles
    let mut disk_manager = DiskManager::new();
    let disks = disk_manager.list_disks();
    
    if disks.is_empty() {
        println!("❌ No se encontraron discos disponibles");
        return;
    }
    
    println!("💽 Discos disponibles:");
    println!("─────────────────────");
    for (i, disk) in disks.iter().enumerate() {
        println!("  {}. {} - {} ({}) - {}", 
            i + 1, 
            disk.name, 
            disk.size, 
            disk.model,
            disk.disk_type
        );
    }
    println!();
    
    // Seleccionar disco
    let disk_choice = read_input("Selecciona el número del disco donde instalar: ");
    let disk_index: usize = match disk_choice.trim().parse::<usize>() {
        Ok(n) if n > 0 && n <= disks.len() => n - 1,
        _ => {
            println!("❌ Número de disco inválido");
            return;
        }
    };
    
    let selected_disk = &disks[disk_index];
    
    // Validar disco seleccionado
    let validator = SystemValidator::new();
    if let Err(e) = validator.validate_disk(&selected_disk.name) {
        println!("❌ Error validando disco: {}", e);
        return;
    }
    
    // Verificar espacio en disco
    if let Err(e) = validator.check_disk_space(&selected_disk.name) {
        println!("❌ Error de espacio en disco: {}", e);
        return;
    }
    
    // Preguntar configuración de la instalación
    println!();
    println!("⚙️  Configuración de instalación:");
    println!("─────────────────────────────────");
    
    let config = match get_installation_config() {
        Some(cfg) => cfg,
        None => {
            println!("❌ Instalación cancelada");
            return;
        }
    };
    
    // Ejecutar instalación directa
    let direct_installer = DirectInstaller::new();
    match direct_installer.install_redox_os(selected_disk, &config) {
        Ok(_) => {
            println!();
            println!("╔═══════════════════════════════════════════════════╗");
            println!("║   ✅ Instalación completada exitosamente! ✅     ║");
            println!("╚═══════════════════════════════════════════════════╝");
            println!();
            println!("🚀 Redox OS está listo para arrancar desde {}", selected_disk.name);
            println!();
            println!("📝 Próximos pasos:");
            println!("   1. Reinicia tu computadora");
            println!("   2. Asegúrate de que UEFI esté habilitado en el BIOS");
            println!("   3. Selecciona {} como dispositivo de arranque", selected_disk.name);
            println!();
        }
        Err(e) => {
            eprintln!();
            eprintln!("❌ Error durante la instalación: {}", e);
            eprintln!("   Por favor revisa los logs y vuelve a intentar");
        }
    }
}

fn get_installation_config() -> Option<InstallationConfig> {
    let efi_size = read_input("Tamaño de partición EFI en MB (por defecto: 512): ");
    let efi_size_mb = if efi_size.trim().is_empty() {
        512
    } else {
        match efi_size.trim().parse::<u64>() {
            Ok(n) if n >= 100 => n,
            _ => {
                println!("⚠️  Tamaño inválido, usando 512 MB");
                512
            }
        }
    };
    
    let filesystem = read_input("Sistema de archivos para root (redoxfs/ext4) [redoxfs]: ");
    let filesystem_type = if filesystem.trim().is_empty() || filesystem.trim().to_lowercase() == "redoxfs" {
        FilesystemType::RedoxFS
    } else if filesystem.trim().to_lowercase() == "ext4" {
        FilesystemType::Ext4
    } else {
        println!("⚠️  Sistema de archivos inválido, usando RedoxFS");
        FilesystemType::RedoxFS
    };
    
    println!();
    println!("⚠️  ¡ADVERTENCIA! ⚠️");
    println!("═══════════════════════════════════════════════════");
    println!("Esta operación BORRARÁ TODOS los datos en el disco seleccionado");
    println!("Las particiones existentes serán ELIMINADAS");
    println!("═══════════════════════════════════════════════════");
    println!();
    
    let confirm = read_input("¿Estás COMPLETAMENTE seguro? (escribe 'SI' en mayúsculas): ");
    if confirm.trim() != "SI" {
        return None;
    }
    
    Some(InstallationConfig {
        efi_size_mb,
        filesystem_type,
    })
}

fn show_disk_info() {
    println!("╔═══════════════════════════════════════════════════╗");
    println!("║          Información de Discos                    ║");
    println!("╚═══════════════════════════════════════════════════╝");
    println!();
    
    let mut disk_manager = DiskManager::new();
    let disks = disk_manager.list_disks();
    
    if disks.is_empty() {
        println!("❌ No se encontraron discos");
        return;
    }
    
    for (i, disk) in disks.iter().enumerate() {
        println!("┌─── Disco #{} ────────────────────────────────────", i + 1);
        println!("│ Dispositivo: {}", disk.name);
        println!("│ Tamaño:      {}", disk.size);
        println!("│ Modelo:      {}", disk.model);
        println!("│ Tipo:        {}", disk.disk_type);
        
        if disk_manager.is_disk_mounted(&disk.name) {
            println!("│ Estado:      ⚠️  MONTADO");
        } else {
            println!("│ Estado:      ✅ Disponible");
        }
        println!("└──────────────────────────────────────────────────");
        println!();
    }
}

fn show_help() {
    println!("╔═══════════════════════════════════════════════════╗");
    println!("║      🦀 Ayuda del Instalador de Redox OS 🦀      ║");
    println!("╚═══════════════════════════════════════════════════╝");
    println!();
    println!("📘 DESCRIPCIÓN");
    println!("───────────────");
    println!("Este instalador te permite instalar Redox OS completo en un disco duro.");
    println!();
    println!("⚙️  REQUISITOS");
    println!("──────────────");
    println!("  • Disco duro con al menos 2GB de espacio libre");
    println!("  • Sistema UEFI compatible");
    println!("  • Redox OS compilado (ejecuta 'make all' primero)");
    println!("  • Privilegios de root/sudo");
    println!();
    println!("⚠️  ADVERTENCIAS IMPORTANTES");
    println!("─────────────────────────────");
    println!("  • La instalación BORRARÁ todos los datos del disco seleccionado");
    println!("  • Haz una copia de seguridad de tus datos importantes");
    println!("  • Asegúrate de seleccionar el disco correcto");
    println!("  • No interrumpas el proceso de instalación");
    println!();
    println!("📋 PROCESO DE INSTALACIÓN");
    println!("──────────────────────────");
    println!("  1. Selección del disco de destino");
    println!("  2. Configuración (tamaño EFI, sistema de archivos)");
    println!("  3. Creación de particiones GPT (EFI + Root)");
    println!("  4. Formateo de particiones");
    println!("  5. Instalación del bootloader UEFI");
    println!("  6. Instalación del kernel de Redox");
    println!("  7. Copia de archivos del sistema");
    println!("  8. Creación de configuración de arranque");
    println!("  9. Verificación de la instalación");
    println!();
    println!("🎯 SISTEMAS DE ARCHIVOS SOPORTADOS");
    println!("────────────────────────────────────");
    println!("  • RedoxFS (recomendado) - Sistema de archivos nativo de Redox");
    println!("  • ext4 - Sistema de archivos Linux estándar");
    println!();
    println!("💡 CONSEJOS");
    println!("───────────");
    println!("  • Usa RedoxFS para mejor rendimiento con Redox OS");
    println!("  • El tamaño mínimo de la partición EFI es 100 MB");
    println!("  • Asegúrate de que UEFI esté habilitado en tu BIOS");
    println!("  • Si el sistema no arranca, verifica la configuración UEFI");
    println!();
}

fn is_root() -> bool {
    unsafe {
        libc::getuid() == 0
    }
}

fn read_input(prompt: &str) -> String {
    print!("{}", prompt);
    io::stdout().flush().unwrap();
    
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    input
}

#[derive(Debug, Clone)]
pub struct DiskInfo {
    pub name: String,
    pub size: String,
    pub model: String,
    pub disk_type: String,
}

#[derive(Debug, Clone)]
pub enum FilesystemType {
    RedoxFS,
    Ext4,
}

#[derive(Debug, Clone)]
pub struct InstallationConfig {
    pub efi_size_mb: u64,
    pub filesystem_type: FilesystemType,
}

