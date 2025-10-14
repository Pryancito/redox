use std::fs;
use std::path::Path;
use std::process::Command;
use crate::{DiskInfo, InstallationConfig, FilesystemType};

// Rutas a las herramientas de RedoxFS
const REDOXFS_MKFS: &str = "/home/moebius/redox/redoxfs/target/release/redoxfs-mkfs";
const REDOXFS_MOUNT: &str = "/home/moebius/redox/redoxfs/target/release/redoxfs";

pub struct DirectInstaller {
    efi_mount_point: String,
    root_mount_point: String,
    redoxfs_uuid: Option<String>,
}

impl DirectInstaller {
    pub fn new() -> Self {
        Self {
            efi_mount_point: "/tmp/redox_install_efi".to_string(),
            root_mount_point: "/tmp/redox_install_root".to_string(),
            redoxfs_uuid: None,
        }
    }

    pub fn install_redox_os(&self, disk: &DiskInfo, config: &InstallationConfig) -> Result<(), String> {
        println!();
        println!("╔═══════════════════════════════════════════════════╗");
        println!("║       🦀 Instalador de Redox OS 🦀                ║");
        println!("╚═══════════════════════════════════════════════════╝");
        println!();

        // Mostrar resumen de configuración
        println!("📋 Configuración de instalación:");
        println!("   Disco:            {}", disk.name);
        println!("   Tamaño disco:     {}", disk.size);
        println!("   Partición EFI:    {} MB", config.efi_size_mb);
        println!("   Sistema archivos: {:?}", config.filesystem_type);
        println!();

        // Verificar disco
        self.verify_disk(disk)?;

        // Desmontar particiones existentes
        self.unmount_existing_partitions(disk)?;

        // Crear particiones
        println!("📦 [1/8] Creando particiones...");
        self.create_partitions(disk, config)?;
        println!("   ✅ Particiones creadas");
        println!();

        // Formatear particiones
        println!("💾 [2/8] Formateando particiones...");
        self.format_partitions(disk, config)?;
        println!("   ✅ Particiones formateadas");
        println!();

        // Montar particiones
        println!("📁 [3/8] Montando particiones...");
        self.mount_partitions(disk)?;
        println!("   ✅ Particiones montadas");
        println!();

        // Instalar bootloader
        println!("⚙️  [4/8] Instalando bootloader UEFI...");
        self.install_bootloader(disk)?;
        println!("   ✅ Bootloader instalado");
        println!();

        // Instalar sistema de archivos (crear directorios primero)
        println!("📂 [5/8] Instalando sistema de archivos...");
        self.install_filesystem(disk)?;
        println!("   ✅ Sistema de archivos instalado");
        println!();

        // Instalar kernel (después de crear directorios)
        println!("🔧 [6/8] Instalando kernel de Redox...");
        self.install_kernel(disk)?;
        println!("   ✅ Kernel instalado");
        println!();

        // Crear configuración
        println!("⚙️  [7/8] Creando configuración de arranque...");
        self.create_config_files(disk)?;
        println!("   ✅ Configuración creada");
        println!();

        // Desmontar particiones
        println!("🔓 [8/8] Desmontando particiones...");
        self.unmount_partitions(disk)?;
        println!("   ✅ Particiones desmontadas");
        println!();

        // Resumen final
        self.print_installation_summary(disk, config)?;

        Ok(())
    }

    fn verify_disk(&self, disk: &DiskInfo) -> Result<(), String> {
        if !Path::new(&disk.name).exists() {
            return Err(format!("{} no existe", disk.name));
        }

        println!("🔍 Verificando disco {}...", disk.name);
        
        // Verificar que no esté montado leyendo /proc/mounts
        let mounts = std::fs::read_to_string("/proc/mounts")
            .map_err(|e| format!("Error leyendo /proc/mounts: {}", e))?;
        
        if mounts.contains(&disk.name) {
            println!("   ⚠️  El disco está montado, desmontando...");
            self.unmount_existing_partitions(disk)?;
        }

        println!("   ✅ Disco verificado");
        Ok(())
    }

    fn unmount_existing_partitions(&self, disk: &DiskInfo) -> Result<(), String> {
        let device_name = disk.name.trim_start_matches("/dev/");
        
        // Buscar particiones montadas
        let output = Command::new("mount")
            .output()
            .map_err(|e| format!("Error ejecutando mount: {}", e))?;
        
        let output_str = String::from_utf8_lossy(&output.stdout);
        
        for line in output_str.lines() {
            if line.contains(device_name) {
                if let Some(partition) = line.split_whitespace().next() {
                    println!("   Desmontando {}...", partition);
                    let _ = Command::new("umount")
                        .arg("-f")
                        .arg(partition)
                        .output();
                }
            }
        }
        
        // Esperar un poco para que se complete el desmontaje
        std::thread::sleep(std::time::Duration::from_secs(1));
        
        Ok(())
    }

    fn create_partitions(&self, disk: &DiskInfo, config: &InstallationConfig) -> Result<(), String> {
        // Limpiar tabla de particiones
        println!("   Limpiando tabla de particiones...");
        let _ = Command::new("wipefs")
            .args(&["-a", &disk.name])
            .output();

        std::thread::sleep(std::time::Duration::from_secs(1));

        // Crear tabla GPT
        println!("   Creando tabla de particiones GPT...");
        let output = Command::new("parted")
            .args(&["-s", &disk.name, "mklabel", "gpt"])
            .output()
            .map_err(|e| format!("Error ejecutando parted: {}", e))?;

        if !output.status.success() {
            return Err(format!("Error creando tabla GPT: {}", String::from_utf8_lossy(&output.stderr)));
        }

        std::thread::sleep(std::time::Duration::from_secs(1));

        // Crear partición EFI
        let efi_end = format!("{}MiB", config.efi_size_mb);
        println!("   Creando partición EFI ({})...", efi_end);
        
        let output = Command::new("parted")
            .args(&["-s", &disk.name, "mkpart", "primary", "fat32", "1MiB", &efi_end])
            .output()
            .map_err(|e| format!("Error creando partición EFI: {}", e))?;

        if !output.status.success() {
            return Err(format!("Error creando partición EFI: {}", String::from_utf8_lossy(&output.stderr)));
        }

        // Marcar partición como ESP
        let output = Command::new("parted")
            .args(&["-s", &disk.name, "set", "1", "esp", "on"])
            .output()
            .map_err(|e| format!("Error marcando ESP: {}", e))?;

        if !output.status.success() {
            return Err(format!("Error marcando partición como ESP: {}", String::from_utf8_lossy(&output.stderr)));
        }

        std::thread::sleep(std::time::Duration::from_secs(1));

        // Crear partición root (resto del disco)
        println!("   Creando partición root (resto del disco)...");
        let output = Command::new("parted")
            .args(&["-s", &disk.name, "mkpart", "primary", &efi_end, "100%"])
            .output()
            .map_err(|e| format!("Error creando partición root: {}", e))?;

        if !output.status.success() {
            return Err(format!("Error creando partición root: {}", String::from_utf8_lossy(&output.stderr)));
        }

        // Sincronizar y esperar
        Command::new("sync").output().ok();
        std::thread::sleep(std::time::Duration::from_secs(2));
        
        let _ = Command::new("partprobe")
            .arg(&disk.name)
            .output();
        
        std::thread::sleep(std::time::Duration::from_secs(2));

        // Verificar que las particiones existen
        let (part1, part2) = self.get_partition_names(disk);
        
        if !Path::new(&part1).exists() || !Path::new(&part2).exists() {
            return Err("Las particiones no se crearon correctamente".to_string());
        }

        Ok(())
    }

    fn format_partitions(&self, disk: &DiskInfo, config: &InstallationConfig) -> Result<(), String> {
        let (efi_partition, root_partition) = self.get_partition_names(disk);

        // Formatear partición EFI como FAT32
        println!("   Formateando {} como FAT32...", efi_partition);
        let output = Command::new("mkfs.vfat")
            .args(&["-F", "32", "-n", "REDOX_EFI", &efi_partition])
            .output()
            .map_err(|e| format!("Error formateando EFI: {}", e))?;

        if !output.status.success() {
            return Err(format!("Error formateando partición EFI: {}", String::from_utf8_lossy(&output.stderr)));
        }

        // Formatear partición root según configuración
        match config.filesystem_type {
            FilesystemType::RedoxFS => {
                println!("   Formateando {} como RedoxFS...", root_partition);
                
                // Verificar que redoxfs-mkfs existe
                if !Path::new(REDOXFS_MKFS).exists() {
                    return Err(format!(
                        "redoxfs-mkfs no encontrado en {}\n   Compila RedoxFS primero: cd /home/moebius/redox/redoxfs && cargo build --release",
                        REDOXFS_MKFS
                    ));
                }
                
                // Verificar que la partición existe y obtener su tamaño
                println!("   Verificando partición {}...", root_partition);
                
                if !Path::new(&root_partition).exists() {
                    return Err(format!("La partición {} no existe", root_partition));
                }
                
                // Obtener tamaño del dispositivo de bloques usando blockdev
                let size_output = Command::new("blockdev")
                    .args(&["--getsize64", &root_partition])
                    .output()
                    .map_err(|e| format!("Error obteniendo tamaño de {}: {}", root_partition, e))?;
                
                if !size_output.status.success() {
                    return Err(format!("No se pudo obtener el tamaño de {}", root_partition));
                }
                
                let size_str = String::from_utf8_lossy(&size_output.stdout);
                let size_bytes: u64 = size_str.trim()
                    .parse()
                    .map_err(|_| format!("Error parseando tamaño de partición: {}", size_str))?;
                
                if size_bytes == 0 {
                    return Err(format!("La partición {} tiene tamaño 0 bytes", root_partition));
                }
                
                let size_mb = size_bytes / 1024 / 1024;
                let size_gb = size_bytes as f64 / 1024.0 / 1024.0 / 1024.0;
                
                println!("   ✅ Partición válida");
                println!("   Tamaño: {} bytes ({} MB / {:.2} GB)", size_bytes, size_mb, size_gb);
                
                // IMPORTANTE: Limpiar metadata anterior de la partición
                println!("   Limpiando metadata anterior de la partición...");
                let wipefs_output = Command::new("wipefs")
                    .args(&["-a", &root_partition])
                    .output();
                
                match wipefs_output {
                    Ok(output) if output.status.success() => {
                        println!("   ✅ Metadata anterior limpiada");
                    }
                    Ok(output) => {
                        let stderr = String::from_utf8_lossy(&output.stderr);
                        if !stderr.is_empty() {
                            println!("   ⚠️  wipefs: {}", stderr.trim());
                        }
                    }
                    Err(e) => {
                        println!("   ⚠️  No se pudo ejecutar wipefs: {}", e);
                    }
                }
                
                // Escribir ceros al inicio de la partición para asegurar limpieza
                println!("   Escribiendo ceros al inicio de la partición...");
                let dd_output = Command::new("dd")
                    .args(&[
                        "if=/dev/zero",
                        &format!("of={}", root_partition),
                        "bs=1M",
                        "count=10",
                        "conv=notrunc"
                    ])
                    .output();
                
                match dd_output {
                    Ok(output) if output.status.success() => {
                        println!("   ✅ Partición limpiada");
                    }
                    Ok(_) => {
                        println!("   ⚠️  Advertencia: No se pudo limpiar completamente la partición");
                    }
                    Err(e) => {
                        println!("   ⚠️  Error ejecutando dd: {}", e);
                    }
                }
                
                // Sincronizar antes de formatear
                Command::new("sync").output().ok();
                std::thread::sleep(std::time::Duration::from_secs(1));
                
                println!("   Usando: {}", REDOXFS_MKFS);
                println!("   Ejecutando: {} {}", REDOXFS_MKFS, root_partition);
                
                // Usar redoxfs-mkfs de la carpeta redoxfs
                // Nota: redoxfs-mkfs imprime mensajes a stderr incluso en éxito
                let output = Command::new(REDOXFS_MKFS)
                    .arg(&root_partition)
                    .output()
                    .map_err(|e| format!("Error ejecutando redoxfs-mkfs: {}", e))?;
                
                // Mostrar salida de redoxfs-mkfs para debugging
                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);
                
                // redoxfs-mkfs imprime a stderr tanto éxitos como errores
                if !stderr.is_empty() {
                    for line in stderr.lines() {
                        println!("   redoxfs-mkfs: {}", line);
                    }
                }
                
                if !stdout.is_empty() {
                    println!("   Stdout: {}", stdout.trim());
                }
                
                if !output.status.success() {
                    return Err(format!(
                        "redoxfs-mkfs falló (código: {:?})\n   Ver mensajes arriba para más detalles",
                        output.status.code()
                    ));
                }
                
                // Verificar que el mensaje de éxito apareció
                if !stderr.contains("created filesystem") {
                    return Err(format!(
                        "redoxfs-mkfs no reportó éxito. Salida:\n{}",
                        stderr
                    ));
                }
                
                // Extraer UUID del mensaje de éxito
                let redoxfs_uuid = if let Some(uuid_line) = stderr.lines().find(|line| line.contains("uuid")) {
                    if let Some(uuid_part) = uuid_line.split_whitespace().last() {
                        uuid_part.to_string()
                    } else {
                        return Err("No se pudo extraer UUID de redoxfs-mkfs".to_string());
                    }
                } else {
                    return Err("No se encontró UUID en la salida de redoxfs-mkfs".to_string());
                };
                
                println!("   ✅ RedoxFS formateado exitosamente con UUID: {}", redoxfs_uuid);
                
                // Almacenar el UUID para usar en la configuración
                // Nota: necesitamos una forma de pasar este UUID a create_config_files
                // Por ahora, lo guardamos en un archivo temporal
                let uuid_file = "/tmp/redox_install_uuid";
                fs::write(uuid_file, &redoxfs_uuid)
                    .map_err(|e| format!("Error guardando UUID: {}", e))?;
                
                // Sincronizar para asegurar que los cambios se escribieron al disco
                println!("   Sincronizando datos al disco...");
                Command::new("sync").output().ok();
                std::thread::sleep(std::time::Duration::from_secs(2));
                println!("   ✅ Sincronización completada");
            }
            FilesystemType::Ext4 => {
                println!("   Formateando {} como ext4...", root_partition);
                let output = Command::new("mkfs.ext4")
                    .args(&["-F", "-L", "REDOX_ROOT", &root_partition])
                    .output()
                    .map_err(|e| format!("Error formateando root: {}", e))?;
                
                if !output.status.success() {
                    return Err(format!("Error formateando partición root: {}", String::from_utf8_lossy(&output.stderr)));
                }
            }
        }

        Ok(())
    }

    fn mount_partitions(&self, disk: &DiskInfo) -> Result<(), String> {
        let (efi_partition, root_partition) = self.get_partition_names(disk);

        // Crear directorios de montaje
        fs::create_dir_all(&self.efi_mount_point)
            .map_err(|e| format!("Error creando directorio EFI: {}", e))?;
        fs::create_dir_all(&self.root_mount_point)
            .map_err(|e| format!("Error creando directorio root: {}", e))?;

        // Montar partición EFI
        println!("   Montando {} en {}...", efi_partition, self.efi_mount_point);
        let output = Command::new("mount")
            .args(&[&efi_partition, &self.efi_mount_point])
            .output()
            .map_err(|e| format!("Error montando EFI: {}", e))?;

        if !output.status.success() {
            return Err(format!("Error montando partición EFI: {}", String::from_utf8_lossy(&output.stderr)));
        }

        // Montar partición root
        println!("   Montando {} en {}...", root_partition, self.root_mount_point);
        
        // Para RedoxFS, necesitamos usar un enfoque diferente
        // RedoxFS es un sistema de archivos FUSE que se ejecuta en foreground por defecto
        // Por ahora, usaremos mount estándar que puede manejar RedoxFS si está registrado
        
        println!("   ⚠️  Nota: RedoxFS requiere ejecución en segundo plano (FUSE)");
        println!("   Usando mount estándar para compatibilidad...");
        
        let output = Command::new("mount")
            .args(&["-t", "auto", &root_partition, &self.root_mount_point])
            .output()
            .map_err(|e| format!("Error montando root: {}", e))?;

        if !output.status.success() {
            // Si mount falla, intentar con redoxfs en background usando spawn
            println!("   ⚠️  Mount estándar falló, intentando RedoxFS en background...");
            
            if Path::new(REDOXFS_MOUNT).exists() {
                println!("   Iniciando RedoxFS en modo background: {}", REDOXFS_MOUNT);
                
                // Iniciar redoxfs como proceso en background
                let child = Command::new(REDOXFS_MOUNT)
                    .args(&[&root_partition, &self.root_mount_point])
                    .spawn()
                    .map_err(|e| format!("Error iniciando redoxfs: {}", e))?;
                
                // Dar tiempo para que monte
                println!("   Esperando que RedoxFS se monte...");
                std::thread::sleep(std::time::Duration::from_secs(3));
                
                // Verificar que el directorio esté montado
                let mount_check = Command::new("mountpoint")
                    .arg(&self.root_mount_point)
                    .output();
                
                match mount_check {
                    Ok(output) if output.status.success() => {
                        println!("   ✅ Partición montada con RedoxFS en background (PID: {})", child.id());
                        
                        // Verificar que podemos acceder al directorio
                        if fs::metadata(&self.root_mount_point).is_ok() {
                            println!("   ✅ Directorio de montaje accesible");
                            
                            // Intentar crear un directorio de prueba
                            let test_dir = format!("{}/test_mount", self.root_mount_point);
                            match fs::create_dir(&test_dir) {
                                Ok(_) => {
                                    let _ = fs::remove_dir(&test_dir);
                                    println!("   ✅ RedoxFS funciona correctamente");
                                    return Ok(());
                                }
                                Err(e) => {
                                    println!("   ⚠️  Error escribiendo en RedoxFS: {}", e);
                                    return Err("RedoxFS montado pero no accesible para escritura".to_string());
                                }
                            }
                        } else {
                            println!("   ⚠️  No se puede acceder al directorio de montaje");
                            return Err("Directorio de montaje no accesible".to_string());
                        }
                    }
                    Ok(output) => {
                        println!("   ⚠️  mountpoint falló: {}", String::from_utf8_lossy(&output.stderr));
                        return Err("RedoxFS no se montó correctamente".to_string());
                    }
                    Err(e) => {
                        println!("   ⚠️  Error verificando mountpoint: {}", e);
                        return Err("No se pudo verificar el montaje de RedoxFS".to_string());
                    }
                }
            } else {
                return Err(format!("Error montando partición root: {}", String::from_utf8_lossy(&output.stderr)));
            }
        }
        
        println!("   ✅ Partición montada exitosamente");

        Ok(())
    }

    fn install_bootloader(&self, disk: &DiskInfo) -> Result<(), String> {
        // Crear estructura EFI
        let efi_boot_dir = format!("{}/EFI/BOOT", self.efi_mount_point);
        let efi_redox_dir = format!("{}/EFI/redox", self.efi_mount_point);
        
        fs::create_dir_all(&efi_boot_dir)
            .map_err(|e| format!("Error creando directorio EFI/BOOT: {}", e))?;
        fs::create_dir_all(&efi_redox_dir)
            .map_err(|e| format!("Error creando directorio EFI/redox: {}", e))?;

        // Buscar bootloader compilado
        let bootloader_paths = vec![
            "cookbook/recipes/core/bootloader/target/x86_64-unknown-redox/build/bootloader.efi",
            "cookbook/recipes/core/bootloader/target/x86_64-unknown-redox/stage/boot/bootloader.efi",
            "build/x86_64/desktop/bootloader-live.efi",
            "build/x86_64/desktop/bootloader.efi",
            "cookbook/recipes/core/bootloader/source/build/bootloader_x86_64-unknown-uefi.efi",
            "build/bootloader.efi",
        ];
        
        let bootloader_source = bootloader_paths.iter()
            .find(|p| Path::new(p).exists())
            .ok_or_else(|| {
                format!(
                    "Bootloader no encontrado en ninguna ubicación esperada.\n   Rutas buscadas:\n{}",
                    bootloader_paths.iter()
                        .map(|p| format!("     - {}", p))
                        .collect::<Vec<_>>()
                        .join("\n")
                )
            })?;

        println!("   Encontrado bootloader: {}", bootloader_source);
        
        // Copiar bootloader
        let bootx64_path = format!("{}/BOOTX64.EFI", efi_boot_dir);
        let redox_boot_path = format!("{}/redox-bootloader.efi", efi_redox_dir);
        
        fs::copy(bootloader_source, &bootx64_path)
            .map_err(|e| format!("Error copiando bootloader a BOOTX64.EFI: {}", e))?;
        
        fs::copy(bootloader_source, &redox_boot_path)
            .map_err(|e| format!("Error copiando bootloader a redox/: {}", e))?;

        // Crear entrada de arranque con efibootmgr (opcional, puede fallar en VMs)
        let disk_name = disk.name.trim_end_matches(char::is_numeric);
        let _ = Command::new("efibootmgr")
            .args(&[
                "--create",
                "--disk", disk_name,
                "--part", "1",
                "--label", "Redox OS",
                "--loader", "\\EFI\\redox\\redox-bootloader.efi",
            ])
            .output();

        Ok(())
    }

    fn install_kernel(&self, _disk: &DiskInfo) -> Result<(), String> {
        // Buscar kernel compilado
        let kernel_paths = vec![
            "cookbook/recipes/core/kernel/target/x86_64-unknown-redox/build/kernel",
            "cookbook/recipes/core/kernel/target/x86_64-unknown-redox/stage/boot/kernel",
            "build/x86_64/desktop/kernel",
            "build/x86_64/desktop/harddrive/kernel",
            "cookbook/recipes/core/kernel/source/target/x86_64-unknown-redox/release/kernel",
        ];
        
        let kernel_source = kernel_paths.iter()
            .find(|p| Path::new(p).exists())
            .ok_or_else(|| {
                format!(
                    "Kernel no encontrado en ninguna ubicación esperada.\n   Rutas buscadas:\n{}",
                    kernel_paths.iter()
                        .map(|p| format!("     - {}", p))
                        .collect::<Vec<_>>()
                        .join("\n")
                )
            })?;

        println!("   Encontrado kernel: {}", kernel_source);
        
        // El kernel debe estar en la partición RedoxFS (segunda partición)
        // El bootloader monta RedoxFS y busca el kernel ahí
        let kernel_dest_root = format!("{}/boot/kernel", self.root_mount_point);
        fs::copy(kernel_source, &kernel_dest_root)
            .map_err(|e| format!("Error copiando kernel a /boot/kernel en RedoxFS: {}", e))?;
        
        println!("   ✅ Kernel copiado a /boot/kernel en partición RedoxFS");

        // Buscar e instalar initfs si existe
        let initfs_paths = vec![
            "cookbook/recipes/core/base-initfs/target/x86_64-unknown-redox/build/initfs.img",
            "build/x86_64/desktop/initfs.img",
            "build/x86_64/desktop/harddrive/initfs.img",
        ];
        
        for initfs_path in initfs_paths {
            if Path::new(initfs_path).exists() {
                println!("   Encontrado initfs: {}", initfs_path);
                
                // Verificar tamaño del initfs
                let initfs_size = fs::metadata(initfs_path)
                    .map_err(|e| format!("Error obteniendo tamaño del initfs: {}", e))?
                    .len();
                println!("   Tamaño del initfs: {} bytes", initfs_size);
                
                // El initfs debe estar en la partición RedoxFS (segunda partición)
                // El bootloader monta RedoxFS y busca el initfs ahí como "initfs" (sin extensión)
                let initfs_dest_root = format!("{}/boot/initfs", self.root_mount_point);
                fs::copy(initfs_path, &initfs_dest_root)
                    .map_err(|e| format!("Error copiando initfs a /boot/initfs en RedoxFS: {}", e))?;
                
                // Verificar que se copió correctamente
                let copied_size = fs::metadata(&initfs_dest_root)
                    .map_err(|e| format!("Error verificando initfs copiado: {}", e))?
                    .len();
                println!("   Initfs copiado: {} bytes", copied_size);
                
                if initfs_size == copied_size {
                    println!("   ✅ Initfs copiado correctamente a /boot/initfs en partición RedoxFS");
                } else {
                    return Err(format!("Error: initfs no se copió correctamente ({} vs {} bytes)", initfs_size, copied_size));
                }
                break;
            }
        }

        Ok(())
    }

    fn install_filesystem(&self, _disk: &DiskInfo) -> Result<(), String> {
        println!("   Instalando sistema de archivos Redox (igual que harddrive.img)...");
        
        // Replicar exactamente la estructura del harddrive.img oficial
        self.install_redox_filesystem_structure()?;

        Ok(())
    }

    fn install_redox_filesystem_structure(&self) -> Result<(), String> {
        // Crear estructura de directorios exacta como el instalador oficial
        println!("   Creando estructura de directorios Redox...");
        
        // Directorios principales (como en config/base.toml)
        let redox_dirs = vec![
            "/boot", "/usr", "/usr/bin", "/usr/lib", "/usr/libexec", "/usr/share", "/usr/include",
            "/etc", "/var", "/var/log", "/var/lib", "/var/lib/pkg",
            "/tmp", "/home", "/root",
            "/proc", "/sys", "/dev", "/mnt", "/opt",
        ];
        
        for dir in redox_dirs {
            let full_path = format!("{}{}", self.root_mount_point, dir);
            fs::create_dir_all(&full_path)
                .map_err(|e| format!("Error creando directorio {}: {}", dir, e))?;
        }

        // Crear enlaces simbólicos (usrmerge como en Redox oficial)
        self.create_redox_symlinks()?;
        
        // Crear archivos de configuración del sistema
        self.create_redox_config_files()?;
        
        // Crear directorio /boot/ en la partición raíz (requerido por Redox)
        self.create_boot_directory()?;
        
        // Instalar aplicaciones compiladas
        println!("   Instalando aplicaciones de Redox...");
        self.install_redox_applications()?;
        
        Ok(())
    }

    fn create_redox_symlinks(&self) -> Result<(), String> {
        println!("   Creando enlaces simbólicos (usrmerge)...");
        
        // Crear enlaces simbólicos como en Redox oficial
        let symlinks = vec![
            ("/bin", "/usr/bin"),
            ("/lib", "/usr/lib"),
            ("/include", "/usr/include"),
            ("/sbin", "/usr/sbin"),
        ];
        
        for (link, target) in symlinks {
            let link_path = format!("{}{}", self.root_mount_point, link);
            let target_path = format!("{}{}", self.root_mount_point, target);
            
            // Eliminar si existe
            let _ = fs::remove_file(&link_path);
            let _ = fs::remove_dir(&link_path);
            
            // Crear enlace simbólico
            std::os::unix::fs::symlink(target, &link_path)
                .map_err(|e| format!("Error creando enlace {} -> {}: {}", link, target, e))?;
        }
        
        println!("   ✅ Enlaces simbólicos creados");
        Ok(())
    }

    fn create_redox_config_files(&self) -> Result<(), String> {
        println!("   Creando archivos de configuración Redox...");
        
        // /etc/hostname
        let hostname_path = format!("{}/etc/hostname", self.root_mount_point);
        fs::write(&hostname_path, "redox")
            .map_err(|e| format!("Error creando /etc/hostname: {}", e))?;

        // /usr/lib/os-release
        let os_release = r#"PRETTY_NAME="Redox OS 0.9.0"
NAME="Redox OS"
VERSION_ID="0.9.0"
VERSION="0.9.0"
ID="redox-os"

HOME_URL="https://redox-os.org/"
DOCUMENTATION_URL="https://redox-os.org/docs/"
SUPPORT_URL="https://redox-os.org/community/"
"#;
        let os_release_path = format!("{}/usr/lib/os-release", self.root_mount_point);
        fs::write(&os_release_path, os_release)
            .map_err(|e| format!("Error creando /usr/lib/os-release: {}", e))?;

        // /etc/os-release (enlace simbólico)
        let etc_os_release_path = format!("{}/etc/os-release", self.root_mount_point);
        std::os::unix::fs::symlink("../usr/lib/os-release", &etc_os_release_path)
            .map_err(|e| format!("Error creando enlace /etc/os-release: {}", e))?;

        // /etc/pkg.d/50_redox
        let pkg_path = format!("{}/etc/pkg.d", self.root_mount_point);
        fs::create_dir_all(&pkg_path)
            .map_err(|e| format!("Error creando directorio /etc/pkg.d: {}", e))?;
        
        let redox_pkg_path = format!("{}/etc/pkg.d/50_redox", self.root_mount_point);
        fs::write(&redox_pkg_path, "https://static.redox-os.org/pkg")
            .map_err(|e| format!("Error creando /etc/pkg.d/50_redox: {}", e))?;

        // Scripts de inicialización
        self.create_init_scripts()?;
        
        println!("   ✅ Archivos de configuración creados");
        Ok(())
    }

    fn create_init_scripts(&self) -> Result<(), String> {
        // /usr/lib/init.d/00_base
        let init_base = r#"# clear and recreate tmpdir with 0o1777 permission
/usr/bin/rm -r /tmp
/usr/bin/mkdir -m a=rwxt /tmp

/usr/bin/ipcd
/usr/bin/ptyd
/usr/bin/sudo --daemon
"#;
        let init_base_dir = format!("{}/usr/lib/init.d", self.root_mount_point);
        fs::create_dir_all(&init_base_dir)
            .map_err(|e| format!("Error creando directorio /usr/lib/init.d: {}", e))?;
        
        let init_base_path = format!("{}/usr/lib/init.d/00_base", self.root_mount_point);
        fs::write(&init_base_path, init_base)
            .map_err(|e| format!("Error creando /usr/lib/init.d/00_base: {}", e))?;

        // /usr/lib/init.d/00_drivers
        let init_drivers = r#"/usr/bin/pcid-spawner /etc/pcid.d/
"#;
        let init_drivers_path = format!("{}/usr/lib/init.d/00_drivers", self.root_mount_point);
        fs::write(&init_drivers_path, init_drivers)
            .map_err(|e| format!("Error creando /usr/lib/init.d/00_drivers: {}", e))?;

        Ok(())
    }
    
    fn create_boot_directory(&self) -> Result<(), String> {
        println!("   Creando directorio /boot/ en partición raíz...");
        
        // Crear directorio /boot/ en la partición raíz (donde Redox lo busca)
        let boot_dir = format!("{}/boot", self.root_mount_point);
        fs::create_dir_all(&boot_dir)
            .map_err(|e| format!("Error creando directorio /boot en raíz: {}", e))?;
        
        // Crear archivo placeholder para indicar que el directorio existe
        let placeholder_path = format!("{}/boot/.redox_boot", self.root_mount_point);
        fs::write(&placeholder_path, "Redox OS Boot Directory\nCreated by installer\n")
            .map_err(|e| format!("Error creando placeholder en /boot: {}", e))?;
        
        println!("   ✅ Directorio /boot/ creado en partición raíz");
        Ok(())
    }
    
    fn install_redox_applications(&self) -> Result<(), String> {
        let mut total_apps = 0;
        
        // Lista de recetas core con aplicaciones compiladas
        // Orden importante: drivers-initfs debe ir antes que drivers para evitar conflictos
        let core_recipes = vec![
            "uutils",           // rm, mkdir, ls, etc.
            "base",             // ipcd, ptyd
            "userutils",        // sudo
            "coreutils",        // utilidades básicas
            "drivers-initfs",   // drivers (pcid-spawner en /bin/)
            "drivers",          // drivers adicionales
            "ion",              // shell
            "extrautils",       // utilidades adicionales
            "netutils",         // utilidades de red
        ];
        
        for recipe in core_recipes {
            let stage_path = format!(
                "cookbook/recipes/core/{}/target/x86_64-unknown-redox/stage",
                recipe
            );
            
            if Path::new(&stage_path).exists() {
                println!("     Instalando {} ...", recipe);
                let count = self.install_stage_directory(&stage_path)?;
                if count > 0 {
                    println!("     ✅ {} - {} archivos instalados", recipe, count);
                    total_apps += count;
                }
            }
        }
        
        if total_apps > 0 {
            println!("   ✅ {} archivos de aplicaciones instalados en total", total_apps);
        } else {
            println!("   ⚠️  No se encontraron aplicaciones compiladas");
            println!("   Ejecuta 'make' para compilar las aplicaciones de Redox");
        }
        
        Ok(())
    }

    fn install_stage_directory(&self, stage_path: &str) -> Result<usize, String> {
        let mut file_count = 0;
        
        // Buscar subdirectorios en stage (bin, usr/bin, etc.)
        let subdirs_to_check = vec![
            ("bin", "/bin"),
            ("sbin", "/sbin"),
            ("usr/bin", "/usr/bin"),
            ("usr/sbin", "/usr/sbin"),
            ("usr/lib", "/usr/lib"),
            ("etc", "/etc"),
        ];
        
        for (subdir, dest_base) in subdirs_to_check {
            let source_dir = format!("{}/{}", stage_path, subdir);
            
            if Path::new(&source_dir).exists() {
                let dest_dir = format!("{}{}", self.root_mount_point, dest_base);
                
                // Asegurar que el directorio destino existe
                fs::create_dir_all(&dest_dir)
                    .map_err(|e| format!("Error creando directorio {}: {}", dest_base, e))?;
                
                // Copiar archivos del directorio
                let entries = fs::read_dir(&source_dir)
                    .map_err(|e| format!("Error leyendo directorio {}: {}", source_dir, e))?;
                
                for entry in entries {
                    if let Ok(entry) = entry {
                        let source_file = entry.path();
                        if source_file.is_file() {
                            let file_name = source_file.file_name().unwrap();
                            let dest_file = format!("{}/{}", dest_dir, file_name.to_string_lossy());
                            
                            fs::copy(&source_file, &dest_file)
                                .map_err(|e| format!("Error copiando {}: {}", file_name.to_string_lossy(), e))?;
                            
                            file_count += 1;
                        }
                    }
                }
            }
        }
        
        Ok(file_count)
    }

    fn create_config_files(&self, disk: &DiskInfo) -> Result<(), String> {
        let (_efi_partition, root_partition) = self.get_partition_names(disk);
        
        // Crear directorio boot/ (requerido por Redox)
        let boot_dir = format!("{}/boot", self.efi_mount_point);
        fs::create_dir_all(&boot_dir)
            .map_err(|e| format!("Error creando directorio /boot: {}", e))?;
        
        // El directorio boot/ ya se creó en install_kernel()
        // Los archivos kernel e initfs ya están en /boot/ donde el bootloader los busca
        
        // Leer UUID del RedoxFS creado
        let redoxfs_uuid = match fs::read_to_string("/tmp/redox_install_uuid") {
            Ok(uuid) => {
                let uuid_clean = uuid.trim().to_string();
                println!("   UUID del RedoxFS: {}", uuid_clean);
                uuid_clean
            },
            Err(_) => {
                println!("   ⚠️  No se pudo leer UUID, usando partición por defecto");
                root_partition.clone()
            }
        };
        
        // Crear archivo de configuración de arranque (con rutas correctas)
        // El bootloader busca el kernel en /boot/kernel y /boot/initfs
        let boot_conf = format!(
r#"# Redox OS Boot Configuration
kernel=/boot/kernel
root={}
initfs=/boot/initfs
"#,
            root_partition
        );

        // Crear configuración en partición EFI (para bootloader)
        let boot_conf_path_efi = format!("{}/boot/redox.conf", self.efi_mount_point);
        fs::write(&boot_conf_path_efi, &boot_conf)
            .map_err(|e| format!("Error creando /boot/redox.conf en EFI: {}", e))?;
        
        // Crear configuración en partición raíz (donde Redox la busca)
        let boot_conf_path_root = format!("{}/boot/redox.conf", self.root_mount_point);
        fs::write(&boot_conf_path_root, &boot_conf)
            .map_err(|e| format!("Error creando /boot/redox.conf en raíz: {}", e))?;
        
        // También crear en la raíz del sistema de archivos
        let boot_conf_path_root_alt = format!("{}/redox.conf", self.root_mount_point);
        fs::write(&boot_conf_path_root_alt, &boot_conf)
            .map_err(|e| format!("Error creando redox.conf en raíz: {}", e))?;

        // Crear startup.nsh para arranque automático en UEFI
        let startup_script = "\\EFI\\BOOT\\BOOTX64.EFI\n";
        let startup_path = format!("{}/startup.nsh", self.efi_mount_point);
        fs::write(&startup_path, startup_script)
            .map_err(|e| format!("Error creando startup.nsh: {}", e))?;

        // Crear README
        let readme = r#"Redox OS - Sistema Operativo en Rust
====================================

Este disco contiene una instalación completa de Redox OS.

Estructura de archivos:
/boot/redox_kernel    - Kernel de Redox OS
/boot/initfs.img      - Imagen initfs (si existe)
/boot/redox.conf      - Configuración de arranque
/EFI/BOOT/BOOTX64.EFI - Bootloader UEFI

Para arrancar:
1. Reinicia tu computadora
2. Asegúrate de que UEFI esté habilitado
3. Selecciona este disco como dispositivo de arranque

Documentación: https://doc.redox-os.org
Sitio web: https://www.redox-os.org

Desarrollado con ❤️ en Rust
"#;

        let readme_path = format!("{}/README.txt", self.efi_mount_point);
        fs::write(&readme_path, readme)
            .map_err(|e| format!("Error creando README.txt: {}", e))?;

        println!("   ✅ Estructura de arranque creada:");
        println!("     - /EFI/BOOT/ (bootloader)");
        println!("     - /boot/kernel (kernel donde bootloader lo busca)");
        println!("     - /boot/initfs (initfs donde bootloader lo busca)");
        println!("     - /boot/redox.conf (configuración en EFI)");
        println!("     - /boot/redox.conf (configuración en raíz)");
        println!("     - /redox.conf (configuración alternativa en raíz)");

        Ok(())
    }

    fn unmount_partitions(&self, _disk: &DiskInfo) -> Result<(), String> {
        // Sincronizar datos
        Command::new("sync").output().ok();
        std::thread::sleep(std::time::Duration::from_secs(1));

        // Desmontar partición root
        println!("   Desmontando {}...", self.root_mount_point);
        let _ = Command::new("umount")
            .arg(&self.root_mount_point)
            .output();

        // Desmontar partición EFI
        println!("   Desmontando {}...", self.efi_mount_point);
        let _ = Command::new("umount")
            .arg(&self.efi_mount_point)
            .output();

        // Limpiar directorios de montaje
        let _ = fs::remove_dir(&self.root_mount_point);
        let _ = fs::remove_dir(&self.efi_mount_point);

        Ok(())
    }

    fn print_installation_summary(&self, disk: &DiskInfo, config: &InstallationConfig) -> Result<(), String> {
        let (efi_partition, root_partition) = self.get_partition_names(disk);
        
        println!("╔═══════════════════════════════════════════════════╗");
        println!("║          📊 Resumen de Instalación 📊            ║");
        println!("╠═══════════════════════════════════════════════════╣");
        println!("║  Disco:              {}                  ", disk.name);
        println!("║  Partición EFI:      {} (FAT32, {} MB)", efi_partition, config.efi_size_mb);
        println!("║  Partición root:     {} ({:?})      ", root_partition, config.filesystem_type);
        println!("║  Bootloader:         UEFI (BOOTX64.EFI)          ║");
        println!("║  Kernel:             Redox OS                     ║");
        println!("╚═══════════════════════════════════════════════════╝");
        
        Ok(())
    }

    fn get_partition_names(&self, disk: &DiskInfo) -> (String, String) {
        if disk.name.contains("nvme") || disk.name.contains("mmcblk") {
            (
                format!("{}p1", disk.name),
                format!("{}p2", disk.name),
            )
        } else {
            (
                format!("{}1", disk.name),
                format!("{}2", disk.name),
            )
        }
    }

    fn copy_directory_recursive(&self, src: &str, dest: &str) -> Result<(), String> {
        let entries = fs::read_dir(src)
            .map_err(|e| format!("Error leyendo directorio {}: {}", src, e))?;

        for entry in entries {
            let entry = entry.map_err(|e| format!("Error leyendo entrada: {}", e))?;
            let path = entry.path();
            let file_name = path.file_name().unwrap();
            let dest_path = format!("{}/{}", dest, file_name.to_string_lossy());

            if path.is_dir() {
                fs::create_dir_all(&dest_path)
                    .map_err(|e| format!("Error creando directorio {}: {}", dest_path, e))?;
                self.copy_directory_recursive(&path.to_string_lossy(), &dest_path)?;
            } else {
                fs::copy(&path, &dest_path)
                    .map_err(|e| format!("Error copiando archivo {}: {}", file_name.to_string_lossy(), e))?;
            }
        }

        Ok(())
    }
}

