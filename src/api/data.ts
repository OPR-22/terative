import { invoke } from "@tauri-apps/api/core";

export const dataApi = {
  export: (destination: string) =>
    invoke<string>("data_export", { destination }),
  backup: (backupDir?: string) =>
    invoke<string>("data_backup", { backupDir: backupDir ?? null }),
  restore: (source: string) => invoke<void>("data_restore", { source }),
  defaultBackupDir: () => invoke<string>("data_default_backup_dir"),
};
