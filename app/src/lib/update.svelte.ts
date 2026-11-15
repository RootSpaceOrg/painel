import { updateCheck, updateInstall, type UpdateInfo } from './api';

// Estado de atualização compartilhado entre o aviso (topo) e as configurações.
export const upd = $state({ info: null as UpdateInfo | null, checking: false, installing: false, dismissed: '', error: '' });

export async function check() {
  upd.checking = true;
  try { upd.info = await updateCheck(); } finally { upd.checking = false; }
}

export async function install() {
  if (!upd.info?.url) return;
  upd.installing = true;
  upd.error = '';
  try { await updateInstall(upd.info.url); } catch (e) { upd.error = String(e); upd.installing = false; }
}

/** Verifica 20s depois de abrir e a cada 6h. */
export function schedule() {
  setTimeout(check, 20_000);
  setInterval(check, 6 * 3600_000);
}
