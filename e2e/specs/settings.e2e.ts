import { browser, expect } from '@wdio/globals';

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

    const config = await browser.execute(async () => {
      const tauri = (window as any).__TAURI__;
      return tauri.core.invoke('get_config') as Promise<{ revision: number }>;
    });
    expect(typeof config.revision).toBe('number');
  });
});
