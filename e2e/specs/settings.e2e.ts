import { browser, expect } from '@wdio/globals';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';

type ConfigView = {
  revision: number;
  general: { log_level: string } & Record<string, unknown>;
} & Record<string, unknown>;

type PatchArgs = { patch: ConfigView & { expected: number } };

/**
 * config.toml の [general] セクションから log_level だけを返す。
 * 秘密値を含み得る他セクションの行は値を参照しない。
 * 読み取り失敗・キー不在は固定文言で失敗させ、パスを出力に出さない。
 */
function persistedLogLevel(): string {
  const configHome =
    process.env.XDG_CONFIG_HOME || path.join(os.homedir(), '.config');
  const configFile = path.join(configHome, 'openrustbooks', 'config.toml');
  let contents: string;
  try {
    contents = fs.readFileSync(configFile, 'utf8');
  } catch {
    throw new Error('config.toml を読み取れませんでした');
  }

  let inGeneral = false;
  for (const line of contents.split(/\r?\n/)) {
    const trimmed = line.trim();
    if (/^\[[^\]]+\]$/.test(trimmed)) {
      inGeneral = trimmed === '[general]';
      continue;
    }
    if (inGeneral) {
      const match = trimmed.match(/^log_level\s*=\s*"([^"]*)"/);
      if (match) return match[1];
    }
  }
  throw new Error('config.toml の [general] に log_level がありません');
}

describe('wdio: real Tauri WebView settings', () => {
  it('shows editable settings and reads config through real IPC', async () => {
    await browser.url('http://localhost:1420/settings');

    const heading = await browser.$('h1');
    await expect(heading).toHaveText('Settings');

    const revision = await browser.$('#settings-revision');
    await expect(revision).toBeDisplayed();

    const language = await browser.$('#settings-language');
    await expect(language).toBeDisplayed();
    await expect(language).toBeEnabled();

    const hasUpdaterNotice = await browser.execute(() =>
      document.body.innerText.includes('更新確認: 未配線')
    );
    expect(hasUpdaterNotice).toBe(true);

    const getConfig = () =>
      browser.execute(async () => {
        const tauri = (window as any).__TAURI__;
        return tauri.core.invoke('get_config') as Promise<ConfigView>;
      });

    const patchConfig = (args: PatchArgs) =>
      browser.execute(async (patchArgs: PatchArgs) => {
        const tauri = (window as any).__TAURI__;
        return tauri.core.invoke(
          'update_config_patch',
          patchArgs
        ) as Promise<ConfigView>;
      }, args);

    const view = await getConfig();
    expect(typeof view.revision).toBe('number');

    const originalLogLevel = view.general.log_level;
    let attempted = false;
    let primaryFailed = false;
    try {
      attempted = true;
      const updated = await patchConfig({
        patch: {
          ...view,
          expected: view.revision,
          general: { ...view.general, log_level: 'debug' },
        },
      });
      expect(updated.revision).toBe(view.revision + 1);
      expect(persistedLogLevel()).toBe('debug');
    } catch (error) {
      primaryFailed = true;
      throw error;
    } finally {
      if (attempted) {
        try {
          // 1 回目が reject しても最新 revision で復元できるよう再取得する。
          const current = await getConfig();
          const restored = await patchConfig({
            patch: {
              ...current,
              expected: current.revision,
              general: { ...current.general, log_level: originalLogLevel },
            },
          });
          expect(restored.revision).toBe(current.revision + 1);
          expect(persistedLogLevel()).toBe(originalLogLevel);
        } catch (restoreError) {
          // 元の失敗をマスクしない。元処理が成功した場合のみ復元失敗を伝える。
          if (!primaryFailed) throw restoreError;
        }
      }
    }
  });
});
