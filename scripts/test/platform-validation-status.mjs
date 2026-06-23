import { existsSync, readFileSync } from 'node:fs';
import { join, relative } from 'node:path';
import { fileURLToPath } from 'node:url';

const root = fileURLToPath(new URL('../../', import.meta.url));

function pathOf(relativePath) {
  return join(root, relativePath);
}

function readIfExists(relativePath) {
  const absolutePath = pathOf(relativePath);
  return existsSync(absolutePath) ? readFileSync(absolutePath, 'utf8') : '';
}

function has(text, pattern) {
  return pattern.test(text);
}

function statusFrom({ passed = false, blocked = false, pending = false }) {
  if (passed) {
    return 'passed';
  }
  if (blocked) {
    return 'blocked';
  }
  if (pending) {
    return 'pending';
  }
  return 'unknown';
}

function record(relativePath, status, detail) {
  return {
    file: relative(root, pathOf(relativePath)).replaceAll('\\', '/'),
    status,
    detail,
  };
}

const windowsCargoPath = 'src/host/windows-host/Cargo.toml';
const windowsMainPath = 'src/host/windows-host/src/main.rs';
const androidRecordPath = 'docs/test/Android真机验收记录_20260622.md';
const androidLocalReportPath = '.temp/android-device-loop-report.local.md';
const linuxRecordPath = 'docs/test/Linux桌面WSL验收记录_20260622.md';
const linuxSecretReportPath = '.temp/linux-secret-service-wsl-report.local.md';

const windowsCargo = readIfExists(windowsCargoPath);
const windowsMain = readIfExists(windowsMainPath);
const androidRecord = readIfExists(androidRecordPath);
const androidLocalReport = readIfExists(androidLocalReportPath);
const linuxRecord = readIfExists(linuxRecordPath);
const linuxSecretReport = readIfExists(linuxSecretReportPath);

const windowsStatus = statusFrom({
  passed: has(windowsCargo, /default = \["ui-slint"\]/u)
    && !has(windowsCargo, /ui-tray/u)
    && has(windowsMain, /run_desktop_ui_entry\(config\)/u)
    && !has(windowsMain, /run_tray_entry|--tray|mod tray_ui/u),
  pending: existsSync(pathOf(windowsCargoPath)) && existsSync(pathOf(windowsMainPath)),
});

const androidStatus = statusFrom({
  passed: ['AND-01', 'AND-02', 'AND-03', 'AND-04', 'AND-05', 'AND-06', 'AND-07', 'AND-08', 'AND-09'].every((id) =>
    has(androidRecord, new RegExp(`${id}[^\\n|]*(?:\\|[^\\n|]*){2}\\|\\s*通过\\s*\\|`, 'u')),
  ),
  blocked: has(androidRecord, /未发现设备|无连接设备/u) || has(androidLocalReport, /adb devices \| blocked|android host local http \| blocked/u),
  pending: has(androidRecord, /AND-01/u),
});

const linuxSecretStatus = statusFrom({
  passed: has(linuxSecretReport, /"available": true/u) || has(linuxRecord, /LIN-05[^]*\|\s*通过\s*\|/u),
  blocked: has(linuxSecretReport, /SECRET_TOOL=missing|secret-tool is required/u) || has(linuxRecord, /secret-tool.*缺失|未安装 `secret-tool`|阻塞/u),
  pending: has(linuxRecord, /LIN-05/u),
});

const linuxPackageStatus = statusFrom({
  passed: has(linuxRecord, /LIN-01[^]*\|\s*通过\s*\|/u)
    && has(linuxRecord, /LIN-02[^]*\|\s*通过\s*\|/u)
    && has(linuxRecord, /LIN-03[^]*\|\s*通过\s*\|/u)
    && has(linuxRecord, /LIN-04[^]*\|\s*通过\s*\|/u),
  pending: has(linuxRecord, /LIN-01/u),
});

const items = {
  windowsSlintUi: record(
    windowsMainPath,
    windowsStatus,
    windowsStatus === 'passed'
      ? 'Windows default executable is the single Slint UI path.'
      : 'Windows Slint UI single-entry checks are not complete.',
  ),
  androidDevice: record(
    androidRecordPath,
    androidStatus,
    androidStatus === 'passed'
      ? 'Android device checks are marked passed.'
      : `Android device checks are not complete. Latest local report exists: ${existsSync(pathOf(androidLocalReportPath))}.`,
  ),
  linuxPackageAndDesktop: record(
    linuxRecordPath,
    linuxPackageStatus,
    linuxPackageStatus === 'passed'
      ? 'WSL packaging, package contents, desktop material, and WSL script checks are marked passed.'
      : 'Linux package/desktop prechecks are not fully marked passed.',
  ),
  linuxSecretService: record(
    linuxRecordPath,
    linuxSecretStatus,
    linuxSecretStatus === 'passed'
      ? 'Linux Secret Service is marked available.'
      : `Linux Secret Service remains unavailable or unverified. Diagnostic report exists: ${existsSync(pathOf(linuxSecretReportPath))}.`,
  ),
};

const canArchiveManagement = Object.values(items).every((item) => item.status === 'passed');
const nonDeviceItems = {
  windowsSlintUiBuild: {
    status: windowsStatus,
    detail: items.windowsSlintUi.detail,
  },
  androidLocalProbeReport: {
    status: existsSync(pathOf(androidLocalReportPath)) ? 'passed' : 'pending',
    detail: `Android local probe report exists: ${existsSync(pathOf(androidLocalReportPath))}.`,
  },
  linuxPackageAndDesktop: {
    status: linuxPackageStatus,
    detail: items.linuxPackageAndDesktop.detail,
  },
  linuxSecretServiceDiagnosticReport: {
    status: existsSync(pathOf(linuxSecretReportPath)) ? 'passed' : 'pending',
    detail: `Linux Secret Service diagnostic report exists: ${existsSync(pathOf(linuxSecretReportPath))}.`,
  },
};
const nonDeviceReady = Object.values(nonDeviceItems).every((item) => item.status === 'passed');
const status = canArchiveManagement
  ? 'ready_to_archive'
  : nonDeviceReady
    ? 'non_device_ready'
    : 'not_ready';

console.log(JSON.stringify({
  status,
  canArchiveManagement,
  nonDeviceReady,
  nonDeviceItems,
  items,
  requiredBeforeArchive: canArchiveManagement
    ? []
    : Object.entries(items)
        .filter(([, item]) => item.status !== 'passed')
        .map(([key, item]) => ({ key, status: item.status, detail: item.detail })),
  requiredBeforeNonDeviceReady: nonDeviceReady
    ? []
    : Object.entries(nonDeviceItems)
        .filter(([, item]) => item.status !== 'passed')
        .map(([key, item]) => ({ key, status: item.status, detail: item.detail })),
}, null, 2));
