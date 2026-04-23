import fs from 'fs';
import path from 'path';
import os from 'os';

/**
 * Get the default SQLite database path (where Tauri stores it)
 * On Windows: %APPDATA%/bsr-monitor/bsr_monitor.db
 */
export function getDbPath(): string {
  const appData = process.env.APPDATA || path.join(os.homedir(), 'AppData', 'Roaming');
  return path.join(appData, 'bsr-monitor', 'bsr_monitor.db');
}

/**
 * Backup the current database to a test fixture file
 */
export function backupDb(backupPath: string = getBackupPath()): string {
  const dbPath = getDbPath();
  if (!fs.existsSync(dbPath)) {
    console.log('  No database found to backup, starting fresh.');
    return backupPath;
  }
  fs.copyFileSync(dbPath, backupPath);
  console.log(`  Database backed up to: ${backupPath}`);
  return backupPath;
}

/**
 * Restore the database from a backup
 */
export function restoreDb(backupPath: string): void {
  const dbPath = getDbPath();
  const dbDir = path.dirname(dbPath);

  if (!fs.existsSync(backupPath)) {
    console.log(`  No backup found at: ${backupPath}, starting fresh.`);
    return;
  }

  // Ensure directory exists
  if (!fs.existsSync(dbDir)) {
    fs.mkdirSync(dbDir, { recursive: true });
  }

  fs.copyFileSync(backupPath, dbPath);
  console.log(`  Database restored from: ${backupPath}`);
}

/**
 * Delete the current database (clean start)
 */
export function cleanDb(): void {
  const dbPath = getDbPath();
  if (fs.existsSync(dbPath)) {
    fs.unlinkSync(dbPath);
    console.log(`  Database deleted: ${dbPath}`);
  }
  // Also delete -shm and -wal files (SQLite write-ahead log)
  for (const suffix of ['-shm', '-wal']) {
    const p = dbPath + suffix;
    if (fs.existsSync(p)) {
      fs.unlinkSync(p);
    }
  }
}

/**
 * Generate a backup path in the fixtures directory
 */
export function getBackupPath(name: string = 'pre-test'): string {
  return path.resolve(__dirname, '../fixtures', `db-${name}.db`);
}
