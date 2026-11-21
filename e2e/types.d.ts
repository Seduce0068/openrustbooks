/// <reference types="webdriverio/async" />
/// <reference types="@wdio/globals" />

declare namespace WebdriverIO {
  interface Config {
    runner?: string;
    specs?: string[];
    maxInstances?: number;
    capabilities?: unknown[];
    services?: unknown[];
    logLevel?: string;
    bail?: number;
    waitforTimeout?: number;
    connectionRetryTimeout?: number;
    connectionRetryCount?: number;
    framework?: string;
    reporters?: string[];
    mochaOpts?: Record<string, unknown>;
  }
  interface Browser {
    url(path: string): Promise<void>;
    waitUntil(fn: () => Promise<boolean>, options?: { timeout?: number }): Promise<void>;
    execute<T, A = undefined>(fn: (arg: A) => T | Promise<T>, arg?: A): Promise<T>;
    getUrl(): Promise<string>;
    getTitle(): Promise<string>;
    getPageSource(): Promise<string>;
    $(selector: string): Promise<WebdriverIO.Element>;
  }
  interface Element {
    click(): Promise<void>;
    getText(): Promise<string>;
    isDisplayed(): Promise<boolean>;
    isEnabled(): Promise<boolean>;
  }
}

declare function describe(name: string, fn: () => void): void;
declare function it(name: string, fn: () => Promise<void> | void): void;
