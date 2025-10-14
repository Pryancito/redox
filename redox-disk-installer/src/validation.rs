use std::path::Path;
use std::process::Command;
use std::os::unix::fs::FileTypeExt;

pub struct SystemValidator;

impl SystemValidator {
    pub fn new() -> Self {
        Self
    }
    
    pub fn validate_system(&self) -> Result<(), String> {
        // Verificar comandos necesarios
        let required_commands = vec![
            "parted",
            "mkfs.vfat",
            "lsblk",
            "mount",
            "umount",
            "sync",
        ];
        
        for cmd in required_commands {
            if !self.command_exists(cmd) {
                return Err(format!("Comando requerido no encontrado: {}", cmd));
            }
        }
        
        Ok(())
    }
    
    pub fn validate_redox_build(&self) -> Result<(), String> {
        // Verificar que existan archivos compilados de Redox
        let paths_to_check = vec![
            "build/x86_64",
            "cookbook/recipes/core/kernel",
            "cookbook/recipes/core/bootloader",
        ];
        
        for path in paths_to_check {
            if !Path::new(path).exists() {
                return Err(format!("Directorio de compilación no encontrado: {}", path));
            }
        }
        
        // Verificar que RedoxFS esté compilado
        let redoxfs_mkfs = "/home/moebius/redox/redoxfs/target/release/redoxfs-mkfs";
        let redoxfs_mount = "/home/moebius/redox/redoxfs/target/release/redoxfs";
        
        if !Path::new(redoxfs_mkfs).exists() || !Path::new(redoxfs_mount).exists() {
            return Err(format!(
                "RedoxFS no está compilado.\n   Compílalo con: cd /home/moebius/redox/redoxfs && cargo build --release"
            ));
        }
        
        println!("✅ RedoxFS encontrado:");
        println!("   - {}", redoxfs_mkfs);
        println!("   - {}", redoxfs_mount);
        
        Ok(())
    }
    
    pub fn validate_disk(&self, disk_path: &str) -> Result<(), String> {
        if !Path::new(disk_path).exists() {
            return Err(format!("{} no existe", disk_path));
        }
        
        // Verificar que sea un dispositivo de bloques
        let metadata = std::fs::metadata(disk_path)
            .map_err(|e| format!("Error leyendo metadata de {}: {}", disk_path, e))?;
        
        if !metadata.file_type().is_block_device() {
            return Err(format!("{} no es un dispositivo de bloques", disk_path));
        }
        
        Ok(())
    }
    
    pub fn check_disk_space(&self, disk_path: &str) -> Result<(), String> {
        // Obtener tamaño del disco usando blockdev
        let output = Command::new("blockdev")
            .args(&["--getsize64", disk_path])
            .output()
            .map_err(|e| format!("Error ejecutando blockdev: {}", e))?;
        
        if !output.status.success() {
            return Err("No se pudo obtener el tamaño del disco".to_string());
        }
        
        let size_str = String::from_utf8_lossy(&output.stdout);
        let size_bytes: u64 = size_str.trim()
            .parse()
            .map_err(|_| "Error parseando tamaño del disco".to_string())?;
        
        let size_gb = size_bytes as f64 / 1024.0 / 1024.0 / 1024.0;
        
        // Redox OS requiere al menos 2 GB
        if size_gb < 2.0 {
            return Err(format!(
                "El disco es demasiado pequeño ({:.2} GB). Se requieren al menos 2 GB",
                size_gb
            ));
        }
        
        println!("✅ Espacio en disco: {:.2} GB (suficiente)", size_gb);
        Ok(())
    }
    
    fn command_exists(&self, cmd: &str) -> bool {
        Command::new("which")
            .arg(cmd)
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }
}

