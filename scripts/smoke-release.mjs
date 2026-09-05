// Run against the executable that will be packaged, with no frontend server.
// A plain cargo release build must fail; a Tauri production build must pass.
import assert from 'node:assert/strict';
import { spawn, spawnSync } from 'node:child_process';
import { once } from 'node:events';
import { copyFile, mkdir, mkdtemp, rm, writeFile } from 'node:fs/promises';
import { createServer } from 'node:net';
import { tmpdir } from 'node:os';
import { isAbsolute, join, relative, resolve } from 'node:path';
import { setTimeout as delay } from 'node:timers/promises';

assert.equal(process.platform, 'win32', 'The release smoke test requires Windows');
assert.ok(process.argv[2], 'Usage: node scripts/smoke-release.mjs <executable>');
const directory = await mkdtemp(join(tmpdir(), 'steamthrottle-smoke-'));
let app;
let socket;
const policyKeys = [];
const executableName = `steamthrottle-smoke-${process.pid}.exe`;
try {
  const executable = join(directory, executableName);
  await copyFile(resolve(process.argv[2]), executable);
  await mkdir(join(directory, 'SteamThrottle'));
  await writeFile(join(directory, 'SteamThrottle', 'settings.json'), JSON.stringify({
    automationEnabled: false, restoreOnExit: false, startWithWindows: false,
  }));

  const portServer = createServer();
  portServer.listen(0, '127.0.0.1');
  await once(portServer, 'listening');
  const port = portServer.address().port;
  await new Promise((resolve, reject) => portServer.close(error => error ? reject(error) : resolve()));

  // Hosted Windows runners are elevated. WebView2 150+ ignores their
  // environment overrides, but honors machine policies scoped to this exe.
  // https://learn.microsoft.com/en-us/microsoft-edge/webview2/concepts/security
  if (process.env.GITHUB_ACTIONS === 'true') {
    for (const [policy, value] of Object.entries({
      AdditionalBrowserArguments: `--remote-debugging-port=${port}`,
      UserDataFolder: join(directory, 'webview'),
    })) {
      const key = `HKLM\\SOFTWARE\\Policies\\Microsoft\\Edge\\WebView2\\${policy}`;
      const result = spawnSync('reg.exe', [
        'add', key, '/v', executableName, '/t', 'REG_SZ', '/d', value, '/f',
      ], { windowsHide: true, encoding: 'utf8' });
      assert.equal(result.status, 0, `Cannot configure WebView2 test policy: ${result.stderr}`);
      policyKeys.push(key);
    }
  }

  app = spawn(executable, [], {
    cwd: directory,
    windowsHide: true,
    stdio: 'ignore',
    env: {
      ...process.env,
      LOCALAPPDATA: directory,
      WEBVIEW2_USER_DATA_FOLDER: join(directory, 'webview'),
      WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS: `--remote-debugging-port=${port}`,
    },
  });
  let launchError;
  app.on('error', error => { launchError = error; });

  const deadline = Date.now() + 45_000;
  let page;
  while (Date.now() < deadline && !page) {
    if (launchError) throw launchError;
    assert.equal(app.exitCode, null, `Application exited during startup: ${app.exitCode}`);
    try {
      const response = await fetch(`http://127.0.0.1:${port}/json/list`, {
        signal: AbortSignal.timeout(1000),
      });
      page = (await response.json()).find(target => target.type === 'page');
    } catch { /* WebView2 may still be starting. */ }
    if (!page) await delay(200);
  }
  assert.ok(page, 'WebView2 did not start within 45 seconds');
  socket = new WebSocket(page.webSocketDebuggerUrl);
  await Promise.race([
    once(socket, 'open'),
    delay(5000).then(() => { throw new Error('WebView2 debugger connection timed out'); }),
  ]);

  let nextId = 0;
  async function evaluate(expression) {
    const id = ++nextId;
    return new Promise((resolve, reject) => {
      const timer = setTimeout(() => {
        socket.removeEventListener('message', receive);
        reject(new Error('WebView2 evaluation timed out'));
      }, 5000);
      function receive(event) {
        const message = JSON.parse(event.data);
        if (message.id !== id) return;
        clearTimeout(timer);
        socket.removeEventListener('message', receive);
        if (message.error || message.result?.exceptionDetails) {
          reject(new Error(JSON.stringify(message)));
        } else {
          resolve(message.result.result.value);
        }
      }
      socket.addEventListener('message', receive);
      socket.send(JSON.stringify({ id, method: 'Runtime.evaluate', params: {
        expression, returnByValue: true, awaitPromise: true,
      } }));
    });
  }

  let state;
  while (Date.now() < deadline) {
    state = await evaluate(`({
      url: location.href,
      title: document.title,
      heading: document.querySelector('h1')?.textContent,
      ready: document.querySelector('[aria-label="Toggle automation"]')?.disabled === false,
      text: document.body?.innerText.slice(0, 500)
    })`);
    if (state.ready) break;
    if (/chrome-error:|localhost:1420/.test(state.url)) break;
    await delay(200);
  }
  assert.equal(new URL(state.url).hostname, 'tauri.localhost', JSON.stringify(state));
  assert.equal(state.title, 'Steam Throttle', JSON.stringify(state));
  assert.equal(state.heading, 'Overview', JSON.stringify(state));
  assert.equal(state.ready, true, `UI did not receive its backend snapshot: ${JSON.stringify(state)}`);
  console.log(`Release startup passed: ${state.url}; Overview rendered; backend snapshot received.`);
} finally {
  socket?.close();
  if (app?.pid && app.exitCode === null) {
    spawnSync('taskkill', ['/PID', String(app.pid), '/T', '/F'], { windowsHide: true, stdio: 'ignore' });
  }
  for (const key of policyKeys) {
    const result = spawnSync('reg.exe', ['delete', key, '/v', executableName, '/f'], {
      windowsHide: true, encoding: 'utf8',
    });
    assert.equal(result.status, 0, `Cannot remove WebView2 test policy: ${result.stderr}`);
  }
  // Only remove the uniquely created temporary test directory.
  const child = relative(resolve(tmpdir()), resolve(directory));
  assert.ok(child.startsWith('steamthrottle-smoke-') && !child.includes('..') && !isAbsolute(child));
  await rm(directory, { recursive: true, force: true, maxRetries: 10, retryDelay: 300 });
}
