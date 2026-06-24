import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';

const root = fileURLToPath(new URL('../../', import.meta.url));

function read(relativePath) {
  return readFileSync(join(root, relativePath), 'utf8');
}

const wslPackageScript = read('scripts/release/linux/package_linux_host_wsl.ps1');
assert.doesNotMatch(wslPackageScript, /nvm use 22\b/u, 'WSL packaging must not pin Node.js to v22');
assert.match(wslPackageScript, /best_node_version/u, 'WSL packaging must select the highest installed Node.js >=22');
assert.match(wslPackageScript, /STORYLOCK_WSL_NODE_BIN/u, 'WSL packaging must allow explicit Node binary override');

const preflightScript = read('scripts/vercel/preflight.ps1');
assert.match(preflightScript, /yian-windows-host-0\.1\.0-1-prototype-zip\.json/u);
assert.doesNotMatch(preflightScript, /yian-windows-host-0\.1\.0-1-prototype\.json/u);
assert.match(preflightScript, /yian-linux-host-0\.1\.0-1-prototype-deb\.json/u);
assert.match(preflightScript, /\[switch\]\$SkipHttp/u, 'Vercel preflight must support local-only checks before first deploy');
assert.match(preflightScript, /vercel:project-link/u, 'Vercel preflight must check local project binding');
assert.match(preflightScript, /Deployment-level 404 detected/u, 'Vercel preflight must summarize deployment-level 404 failures');

const publishScript = read('scripts/vercel/publish_site_release.ps1');
assert.match(publishScript, /Assert-VercelProjectLink/u, 'Vercel publish must guard against deploying to the wrong project');
assert.match(publishScript, /-SkipHttp/u, 'Vercel publish must not block first deploy on stale online HTTP preflight');
assert.match(publishScript, /post-deploy Vercel preflight/u, 'Vercel publish must run HTTP preflight after deployment');
assert.match(publishScript, /deploy", "--yes"/u, 'Vercel publish must use non-interactive deploy mode');
assert.match(publishScript, /Test-VercelCliReady/u, 'Vercel publish must fail fast on auth/network problems');
assert.match(publishScript, /Invoke-VercelDeployWithRetry/u, 'Vercel publish must retry transient deploy failures');
assert.match(publishScript, /openid-configuration/u, 'Vercel publish must explain OIDC/TLS failures');
assert.match(publishScript, /Test-VercelDomainAccess/u, 'Vercel publish must diagnose custom-domain ownership/access');
assert.match(publishScript, /Set-VercelDeploymentAlias/u, 'Vercel publish must support binding the production custom domain to the deployment');

const syncEnvScript = read('scripts/vercel/sync_env_file_to_vercel.ps1');
assert.match(syncEnvScript, /Assert-VercelProjectLink/u, 'Vercel env sync must guard against updating the wrong project');

const vercelProductionWorkflow = read('.github/workflows/vercel-production.yml');
assert.match(vercelProductionWorkflow, /workflow_dispatch/u, 'Vercel production workflow must be manually triggered');
assert.match(vercelProductionWorkflow, /VERCEL_TOKEN/u, 'Vercel production workflow must use token auth');
assert.match(vercelProductionWorkflow, /VERCEL_ORG_ID/u, 'Vercel production workflow must bind org id');
assert.match(vercelProductionWorkflow, /VERCEL_PROJECT_ID/u, 'Vercel production workflow must bind project id');
assert.match(vercelProductionWorkflow, /npm test/u, 'Vercel production workflow must run tests before deploy');
assert.match(vercelProductionWorkflow, /vercel@54\.5\.1 deploy --prod --yes/u, 'Vercel production workflow must pin CLI deploy command');

const publishWslScript = read('scripts/vercel/publish_site_release_wsl.ps1');
assert.match(publishWslScript, /npx --yes vercel@54\.5\.1/u, 'WSL Vercel publish must pin fallback CLI version');
assert.match(publishWslScript, /VERCEL_TOKEN/u, 'WSL Vercel publish must support token auth');
assert.match(publishWslScript, /nvm ls --no-colors/u, 'WSL Vercel publish must load nvm Node >=22');
assert.match(publishWslScript, /STORYLOCK_GATEWAY_PUBLIC_URL/u, 'WSL Vercel publish must post-check production URL');
assert.match(publishWslScript, /VERCEL_BIND_CUSTOM_DOMAIN/u, 'WSL Vercel publish must support explicit custom-domain alias binding');

const vercelEnvExample = read('scripts/vercel/.env.example');
assert.match(vercelEnvExample, /VERCEL_PROJECT_NAME=storylock-gateway/u, 'Vercel env example must point to the gateway project, not the generic repo name');

const linuxPackageTest = read('scripts/test/linux-package-contents.mjs');
assert.match(linuxPackageTest, /STORYLOCK_WSL_DISTRO/u, 'Linux package test must allow WSL distro override');
assert.match(linuxPackageTest, /tarPackagePath/u, 'Linux package test must support tar.gz artifacts');
assert.match(linuxPackageTest, /debPackagePath/u, 'Linux package test must support deb artifacts');
assert.match(linuxPackageTest, /EncodedCommand/u, 'Linux zip package test must preserve Unicode paths on Windows');

const androidReadinessTest = read('scripts/test/android-readiness.mjs');
assert.match(androidReadinessTest, /AndroidKeyStore/u, 'Android readiness must guard AndroidKeyStore usage');
assert.match(androidReadinessTest, /BiometricPrompt/u, 'Android readiness must guard BiometricPrompt usage');
assert.match(androidReadinessTest, /validate_android_question_set\.mjs/u, 'Android readiness must validate the bundled question set');
assert.match(androidReadinessTest, /STORYLOCK_ANDROID_APK_CHECKSUM/u, 'Android readiness must guard APK metadata output');

const androidDeviceLoopScript = read('scripts/android/check_device_loop.ps1');
assert.match(androidDeviceLoopScript, /AndroidHostPort/u, 'Android device loop must allow the device host port to be configured');
assert.match(androidDeviceLoopScript, /LocalForwardPort/u, 'Android device loop must allow the adb forward port to be configured');
assert.match(androidDeviceLoopScript, /adb forward "tcp:\$LocalForwardPort" "tcp:\$AndroidHostPort"/u, 'Android device loop must forward local HTTP host endpoints');
assert.match(androidDeviceLoopScript, /\/permission-summary/u, 'Android device loop must check permission-summary through adb forward');
assert.match(androidDeviceLoopScript, /canonicalAnswer\|acceptedAnswers\|privateKey\|signingKeyBytes/u, 'Android device loop must guard sensitive fields in permission-summary');

const packageJson = JSON.parse(read('package.json'));
assert.match(packageJson.scripts['test:non-device-validation'], /test:windows-host-features/u, 'Non-device validation must check the Windows Slint UI build');
assert.match(packageJson.scripts['test:non-device-validation'], /check_device_loop\.ps1/u, 'Non-device validation must generate the Android device-loop report');
assert.match(packageJson.scripts['test:non-device-validation'], /diagnose:linux-secret-service:wsl/u, 'Non-device validation must generate the Linux Secret Service diagnostic report');
assert.match(packageJson.scripts['test:non-device-validation'], /test:platform-readiness/u, 'Non-device validation must run platform readiness checks');
assert.match(packageJson.scripts['test:non-device-validation'], /status:platform-validation/u, 'Non-device validation must summarize remaining real-device blockers');

const buildScript = read('scripts/vercel/build_yian_web.mjs');
assert.match(buildScript, /metadataNameFor/u);
assert.match(buildScript, /-tar-gz\.json/u);
assert.match(buildScript, /-\$1\.json/u);

const windowsHostCargo = read('src/host/windows-host/Cargo.toml');
assert.match(windowsHostCargo, /default = \["ui-slint"\]/u, 'Windows host default debug builds must start the Slint UI');
assert.doesNotMatch(windowsHostCargo, /ui-tray/u, 'Windows host must keep Slint as the only Windows UI feature');

const windowsHostMain = read('src/host/windows-host/src/main.rs');
assert.match(windowsHostMain, /#\[cfg\(feature = "ui-slint"\)\]\s*fn run_default_entry[\s\S]*?run_desktop_ui_entry\(config\)/u, 'Windows host default entry must start the Slint UI');
assert.match(windowsHostMain, /windows_subsystem = "windows"/u, 'Windows host Slint UI builds must use the Windows GUI subsystem');
assert.doesNotMatch(windowsHostMain, /run_tray_entry|--tray|mod tray_ui/u, 'Windows host runtime must not expose a tray UI path');
assert.match(windowsHostMain, /managementStats/u, 'Yian Host status must expose redacted management statistics');
assert.match(windowsHostMain, /authorizationModes/u, 'Yian Host management statistics must list authorization modes');
assert.match(windowsHostMain, /requiredCells/u, 'Yian Host management statistics must include required grid-cell counts');
assert.match(windowsHostMain, /remoteInterfaces/u, 'Yian Host management statistics must aggregate remote interface access');
assert.match(windowsHostMain, /host_internal_only/u, 'Yian Host status must keep local storage paths internal');
assert.doesNotMatch(windowsHostMain, /host\.storage\?\.path|bank\.path/u, 'Yian Host browser UI must not display Host storage or question-bank paths');
assert.match(windowsHostMain, /story-template\/generate/u, 'Yian Host may expose a story-template candidate generation endpoint');
assert.match(windowsHostMain, /story-template\/candidates/u, 'StoryLock must be able to explicitly pull queued story-template candidates');
assert.match(windowsHostMain, /storylock_explicit_pull_only/u, 'Story template candidates must use a StoryLock pull-only model');
assert.match(windowsHostMain, /hostInvokesStoryLock[\s\S]*false/u, 'Host must not actively invoke StoryLock when generating story templates');
assert.match(windowsHostMain, /configured_direct_access/u, 'LLM keys must be represented only as direct-access configured status');
assert.doesNotMatch(windowsHostMain, /"apiKey"|"secretKey"|"llmApiKey"|"rawLlmKey"/u, 'Host UI/status must not serialize raw LLM key values');

const windowsHostSlint = read('src/host/windows-host/src/slint_ui.rs');
assert.doesNotMatch(
  windowsHostSlint,
  /core-package-dir|core-manifest-path|core-catalog-path|core-author-draft-path|core-temp-draft-path|core-templates-dir|managed-objects|set_managed_objects|Host-readable permission summary|StoryLock Core package path|Question bank path/u,
  'Yian Host UI must not expose StoryLock draft, vault, catalog, template, package, or question-bank paths',
);
assert.doesNotMatch(
  windowsHostSlint,
  /config\.data_dir\.join\("storylock-core"\)|ensure_storylock_core_package\(&core_package_dir\)\?/u,
  'Yian Host startup must not initialize StoryLock storage under the Host data directory',
);
assert.match(windowsHostSlint, /STORYLOCK_CORE_DATA_DIR/u, 'StoryLock UI storage must have its own data directory override');
assert.match(windowsHostSlint, /management-stats/u, 'Slint Host UI must include a management statistics surface');
assert.match(windowsHostSlint, /6 of 9 cells/u, 'Slint Host UI must show single-read grid authorization mode');
assert.match(windowsHostSlint, /22 of 24 cells/u, 'Slint Host UI must show local-only story-edit authorization mode');
assert.match(windowsHostSlint, /StoryLock must explicitly pull/u, 'Slint Host UI must describe story-template pull-only ownership');
assert.match(windowsHostSlint, /configured\/missing/u, 'Slint Host UI must not display raw LLM key values');
assert.match(windowsHostSlint, /SettingsIconButton/u, 'Slint Host UI must expose a settings icon button in the header');
assert.match(windowsHostSlint, /property <string> language: "zh"/u, 'Slint Host UI must keep a language setting state');
assert.match(windowsHostSlint, /model: \["中文", "English"\]/u, 'Slint Host UI must use a language dropdown');
assert.match(windowsHostSlint, /root\.language = value == "中文" \? "zh" : "en"/u, 'Slint Host UI language dropdown must switch between Chinese and English');
assert.match(windowsHostSlint, /SettingsIconButton[\s\S]*active-page = 4/u, 'Slint Host UI settings icon must open the settings page');
assert.match(windowsHostSlint, /connection-test-status/u, 'Slint Host UI must show local and remote connection test feedback on the main page');
assert.match(windowsHostSlint, /test-local-host\(\) -> string/u, 'Slint Host UI must expose a local host connection test callback');
assert.match(windowsHostSlint, /test-remote-connection\(\) -> string/u, 'Slint Host UI must expose a remote connection test callback');
assert.match(windowsHostSlint, /Manual launch from Settings only/u, 'StoryLock launch must be triggered from the settings page');
assert.doesNotMatch(windowsHostSlint, /label: "StoryLock UI";[\s\S]{0,120}selected: root\.active-page/u, 'StoryLock launch must not remain as a main navigation page');
assert.match(windowsHostSlint, /export component StoryLockCoreApp[\s\S]*property <string> language: "zh"/u, 'StoryLock Core UI must keep its own language setting state');
assert.match(windowsHostSlint, /export component StoryLockCoreApp[\s\S]*SettingsIconButton[\s\S]*active-page = 5/u, 'StoryLock Core UI must expose a settings icon button');
assert.match(windowsHostSlint, /export component StoryLockCoreApp[\s\S]*model: \["中文", "English"\]/u, 'StoryLock Core UI must use a language dropdown');
assert.match(windowsHostSlint, /24 个问题/u, 'StoryLock Core UI must include Chinese navigation text');
assert.match(windowsHostSlint, /StoryLock Core 界面/u, 'StoryLock Core settings page must describe its language scope');

const linuxSecretServiceWslScript = read('scripts/linux/check_linux_secret_service_wsl.ps1');
assert.match(linuxSecretServiceWslScript, /check_linux_secret_service_wsl\.sh/u, 'Linux WSL Secret Service wrapper must call the shell diagnostic');
assert.match(linuxSecretServiceWslScript, /linux-secret-service-wsl-report\.local\.md/u, 'Linux WSL Secret Service wrapper must write a local report');

const linuxSecretServiceShellScript = read('scripts/linux/check_linux_secret_service_wsl.sh');
assert.match(linuxSecretServiceShellScript, /NODE_SELECTED/u, 'Linux Secret Service diagnostic must report selected Node.js');
assert.match(linuxSecretServiceShellScript, /SECRET_TOOL/u, 'Linux Secret Service diagnostic must report secret-tool availability');
assert.match(linuxSecretServiceShellScript, /CHECK_SECRET_STORE_EXIT/u, 'Linux Secret Service diagnostic must report SecretStore check exit code');
assert.match(linuxSecretServiceShellScript, /check-secret-store\.mjs/u, 'Linux Secret Service diagnostic must run the shared SecretStore check');

console.log(JSON.stringify({
  status: 'passed',
  checks: [
    'wsl-node-selection',
    'preflight-metadata-names',
    'vercel-project-link-guard',
    'vercel-deployment-404-summary',
    'vercel-preflight-deploy-order',
    'vercel-auth-network-diagnostics',
    'vercel-ci-token-deploy',
    'vercel-wsl-token-deploy',
    'linux-package-format-flexibility',
    'download-metadata-name-collision',
    'windows-slint-ui-entry',
    'android-device-loop-permission-summary',
    'linux-secret-service-wsl-diagnostic',
  ],
}, null, 2));
