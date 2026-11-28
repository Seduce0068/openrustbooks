import { browser, expect } from '@wdio/globals';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';

type ConfigView = {
  revision: number;
  general: {
    language: string;
    theme: string;
    log_level: string;
  } & Record<string, unknown>;
  ai?: { openai: { has_api_key: boolean } & Record<string, unknown> } & Record<string, unknown>;
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
  const getConfig = () =>
    browser.execute(async () => {
      const tauri = (window as any).__TAURI__;
      return tauri.core.invoke('get_config') as Promise<ConfigView>;
    });

  const patchConfig = (args: PatchArgs) =>
    browser.execute(async (patchArgs: PatchArgs) => {
      const tauri = (window as any).__TAURI__;
      return tauri.core.invoke('update_config_patch', patchArgs) as Promise<ConfigView>;
    }, args);

  const statusText = () =>
    browser.execute(
      () => document.querySelector('p.settings-notice[role="status"]')?.textContent || ''
    );

  const setInputValue = async (selector: string, value: string) => {
    await browser.waitUntil(
      () => browser.execute((sel: string) => !!document.querySelector(sel), selector),
      { timeout: 10000 }
    );
    await browser.execute(
      (args: { sel: string; val: string }) => {
        const el = document.querySelector(args.sel) as HTMLInputElement;
        el.value = args.val;
        el.dispatchEvent(new Event('input', { bubbles: true }));
      },
      { sel: selector, val: value }
    );
  };

  const getInputValue = async (selector: string) => {
    await browser.waitUntil(
      () => browser.execute((sel: string) => !!document.querySelector(sel), selector),
      { timeout: 10000 }
    );
    return browser.execute(
      (sel: string) => (document.querySelector(sel) as HTMLInputElement).value,
      selector
    );
  };

  const isReadOnly = async (selector: string) => {
    await browser.waitUntil(
      () => browser.execute((sel: string) => !!document.querySelector(sel), selector),
      { timeout: 10000 }
    );
    return browser.execute(
      (sel: string) => (document.querySelector(sel) as HTMLInputElement).readOnly,
      selector
    );
  };

  const restoreLogLevel = async (original: string) => {
    const current = await getConfig();
    const restored = await patchConfig({
      patch: {
        ...current,
        expected: current.revision,
        general: { ...current.general, log_level: original },
      },
    });
    expect(restored.revision).toBe(current.revision + 1);
    expect(persistedLogLevel()).toBe(original);
  };

  it('shows settings and reads config through real IPC', async () => {
    await browser.url('http://localhost:1420/settings');

    const heading = await browser.$('h1');
    await expect(heading).toHaveText('Settings');

    const revision = await browser.$('#settings-revision');
    await expect(revision).toBeDisplayed();

    const language = await browser.$('#settings-language');
    await expect(language).toBeDisplayed();

    const hasUpdaterNotice = await browser.execute(() =>
      document.body.innerText.includes('更新確認: 未配線')
    );
    expect(hasUpdaterNotice).toBe(true);

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
      // 死設定の general 3 項目は IPC が成功しても現在値のまま。
      expect(updated.general.language).toBe(view.general.language);
      expect(updated.general.theme).toBe(view.general.theme);
      expect(updated.general.log_level).toBe(originalLogLevel);
      expect(persistedLogLevel()).toBe(originalLogLevel);
    } catch (error) {
      primaryFailed = true;
      throw error;
    } finally {
      if (attempted) {
        try {
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
          if (!primaryFailed) throw restoreError;
        }
      }
    }
  });

  it('keeps dead general settings read-only and shows a fixed notice', async () => {
    await browser.url('http://localhost:1420/settings');
    for (const selector of [
      '#settings-language',
      '#settings-theme',
      '#settings-log-level',
    ]) {
      await expect(await browser.$(selector)).toBeDisplayed();
      expect(await isReadOnly(selector)).toBe(true);
    }
    const notice = await browser.execute(() =>
      Array.from(document.querySelectorAll('p.settings-notice'))
        .map((element) => element.textContent?.trim() || '')
        .find((text) => text.startsWith('言語・テーマ・ログレベル: 未配線')) || ''
    );
    expect(notice.startsWith('言語・テーマ・ログレベル: 未配線')).toBe(true);
    expect(notice.includes('設定画面には現在値を表示します')).toBe(true);
  });
  it('masks secrets and treats an empty API key as a no-op', async () => {
    await browser.url('http://localhost:1420/settings');
    const view = await getConfig();
    const serialized = JSON.stringify(view);
    for (const key of [
      'api_key',
      'client_secret',
      'refresh_token',
      'extra_headers',
      'path',
      'recent_paths',
    ]) {
      expect(serialized.includes('"' + key + '"')).toBe(false);
    }
    const hasKey = view.ai?.openai.has_api_key;
    expect(typeof hasKey).toBe('boolean');
    expect(await (await browser.$('#settings-api-key-state')).getText()).toBe(
      hasKey ? '設定済み' : '未設定'
    );
    expect(await getInputValue('#settings-api-key')).toBe('');
    await (await browser.$('#settings-api-key-save')).click();
    expect(await statusText()).toBe('入力がありません。変更はありません。');
    const after = await getConfig();
    expect(after.revision).toBe(view.revision);
    expect(persistedLogLevel()).toBe(view.general.log_level);
  });
});
